use crate::{ibmad, ibnetdisc};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    error::Error,
    ffi::{c_void, CStr, CString, FromBytesUntilNulError},
    fmt,
    mem::MaybeUninit,
    ptr,
    rc::{Rc, Weak},
    slice,
};

use ibnetdisc::sys::*;

#[repr(i32)]
#[derive(Debug, Default, Copy, Clone)]
pub enum NodeType {
    #[default]
    UNKNOWN = 0,
    CA = 1,
    SWITCH = 2,
    ROUTER = 3,
}

#[derive(Debug)]
pub enum FabricError {
    NullPointerError,
    UnknownNodeTypeError,
    DiscoveryError,
    PortDiscoveryError,
    PortAlreadyDiscoveredError,
    OriginSameAsRemotePortError,
}

impl fmt::Display for FabricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FabricError::NullPointerError => write!(f, "Null pointer"),
            FabricError::UnknownNodeTypeError => write!(f, "Unknown node type"),
            FabricError::DiscoveryError => write!(f, "Discovery failed"),
            FabricError::PortDiscoveryError => write!(f, "Port discovery failed"),
            FabricError::PortAlreadyDiscoveredError => write!(f, "Port already discovered"),
            FabricError::OriginSameAsRemotePortError => {
                write!(f, "The origin port is the same as the remote port")
            }
        }
    }
}

impl Error for FabricError {}

#[derive(Debug)]
pub struct Fabric {
    pub switches: HashMap<u64, Node>,
    pub routers: HashMap<u64, Node>,
    pub ports: HashMap<(u64, i32), Rc<RefCell<Port>>>,
    pub nodes: HashMap<u64, Rc<RefCell<Node>>>,
}

#[derive(Default, Debug, Clone)]
pub struct Node {
    pub guid: u64,
    pub node_desc: String,
    pub node_type: NodeType,
    pub smalid: u16,
    pub ports: Option<Vec<Rc<RefCell<Port>>>>,
    pub dev_id: u32,
    pub vendor_id: u32,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub guid: u64,
    pub number: i32,
    pub phys_state: u32,
    pub logical_state: u32,
    pub base_lid: u16,
    pub remote_port: Option<Weak<RefCell<Port>>>,
    pub remote_node: Option<Weak<RefCell<Node>>>,
    pub parent: Option<Weak<RefCell<Node>>>,
}

impl Node {
    pub fn rc() -> Rc<Self> {
        Rc::new(Node::default())
    }

    //Get the ports associated with the node
    //Set the parent
    fn get_nd_ports(
        nd_node: &ibnd_node,
        fabric: &mut Fabric,
    ) -> Result<Vec<Rc<RefCell<Port>>>, FabricError> {
        if nd_node.ports.is_null() {
            return Err(FabricError::NullPointerError);
        }

        let num_ports = nd_node.numports;
        let port_ptrs: &[*mut ibnetdisc::sys::ibnd_port] =
            unsafe { slice::from_raw_parts(nd_node.ports, num_ports as usize) };
        let mut ports: Vec<Rc<RefCell<Port>>> = Vec::new();

        for &port_ptr in port_ptrs {
            if !port_ptr.is_null() {
                let nd_port = unsafe { &*port_ptr };
                let r = Port::from_nd_port(nd_port, true, fabric);

                match r {
                    Ok(p) => {
                        ports.push(p);
                    }
                    Err(err) => match err {
                        FabricError::PortAlreadyDiscoveredError => {}
                        _ => {
                            return Err(err);
                        }
                    },
                }
            }
        }

        Ok(ports)
    }

    //From Netdiscover Node (ibnd_node)
    fn from_nd_node(nd_node_ptr: *mut ibnd_node) -> Result<Node, FabricError> {
        if nd_node_ptr.is_null() {
            return Err(FabricError::NullPointerError);
        }

        let nd_node = unsafe { &*nd_node_ptr };
        let mut new_node = Node::default();

        new_node.guid = nd_node.guid;

        //NodeDesc
        let unsigned_vec: Vec<u8> = nd_node.nodedesc.iter().map(|&x| x as u8).collect();
        let r = CStr::from_bytes_until_nul(&unsigned_vec);
        if let Ok(cstr) = r {
            new_node.node_desc = cstr.to_string_lossy().to_string();
        }

        new_node.dev_id = unsafe {
            mad_get_field(
                nd_node.info.as_ptr() as *mut c_void,
                0,
                MAD_FIELDS_IB_NODE_DEVID_F,
            )
        };

        new_node.vendor_id = unsafe {
            mad_get_field(
                nd_node.info.as_ptr() as *mut c_void,
                0,
                MAD_FIELDS_IB_NODE_VENDORID_F,
            )
        };

        new_node.smalid = nd_node.smalid;

        match nd_node.type_ {
            i if i == MAD_NODE_TYPE_IB_NODE_CA as i32 => {
                new_node.node_type = NodeType::CA;
            }
            i if i == MAD_NODE_TYPE_IB_NODE_SWITCH as i32 => {
                new_node.node_type = NodeType::SWITCH;
            }
            i if i == MAD_NODE_TYPE_IB_NODE_ROUTER as i32 => {
                new_node.node_type = NodeType::ROUTER;
            }
            _ => {
                new_node.node_type = NodeType::UNKNOWN;
            }
        }

        Ok(new_node)
    }
}

impl Port {
    //From Netdiscover Port
    pub fn from_nd_port(
        nd_port: &ibnd_port,
        fetch_remote_port: bool,
        fabric: &mut Fabric,
    ) -> Result<Rc<RefCell<Port>>, FabricError> {
        let phys_state = unsafe {
            ibmad::sys::mad_get_field(
                nd_port.info.as_ptr() as *mut c_void,
                0,
                ibmad::sys::MAD_FIELDS_IB_PORT_PHYS_STATE_F,
            )
        };

        let logical_state = unsafe {
            ibmad::sys::mad_get_field(
                nd_port.info.as_ptr() as *mut c_void,
                0,
                ibmad::sys::MAD_FIELDS_IB_PORT_STATE_F,
            )
        };

        let port = Port {
            guid: nd_port.guid,
            number: nd_port.portnum,
            phys_state: phys_state,
            logical_state: logical_state,
            base_lid: nd_port.base_lid,
            remote_port: None,
            remote_node: None,
            parent: None,
        };

        let rc_port = Rc::new(RefCell::new(port));
        let mut remote_port: Option<Weak<RefCell<Port>>> = None;
        let mut remote_node: Option<Weak<RefCell<Node>>> = None;

        if !nd_port.remoteport.is_null() && fetch_remote_port == true {
            let nd_remote_port = unsafe { &*nd_port.remoteport };

            // Remote Port: Check if already in map
            let remote_rc_port = if let Some(existing_port) = fabric
                .ports
                .get(&(nd_remote_port.guid, nd_remote_port.portnum))
            {
                // Use the existing port
                existing_port.clone()
            } else {
                // Create a new port and insert it
                let new_port = Port::from_nd_port(nd_remote_port, false, fabric)?;
                fabric.ports.insert(
                    (nd_remote_port.guid, nd_remote_port.portnum),
                    new_port.clone(),
                );
                new_port
            };

            // Remote Node: Check if already in map
            let remote_rc_node = if let Some(existing_node) = fabric.nodes.get(&nd_remote_port.guid)
            {
                // Use the existing node
                existing_node.clone()
            } else {
                // Create a new node and insert it
                let new_node = Node::from_nd_node(nd_remote_port.node)?;
                let rc_node = Rc::new(RefCell::new(new_node));
                fabric.nodes.insert(nd_remote_port.guid, rc_node.clone());
                rc_node
            };

            remote_port = Some(Rc::downgrade(&remote_rc_port));
            remote_node = Some(Rc::downgrade(&remote_rc_node));
        }

        {
            let mut port_ref = RefCell::borrow_mut(&rc_port);
            port_ref.remote_node = remote_node;
            port_ref.remote_port = remote_port;
        }

        Ok(rc_port)
    }
}
pub trait IBNode {
    fn set_node_desc(&mut self, node_desc_bytes: &[u8]) -> Result<(), FromBytesUntilNulError>;
    fn node_desc(&self) -> &str;
    fn guid(&self) -> u64;
    fn node_type(&self) -> NodeType;
}

impl IBNode for Node {
    fn set_node_desc(&mut self, node_desc_bytes: &[u8]) -> Result<(), FromBytesUntilNulError> {
        let cstr = CStr::from_bytes_until_nul(node_desc_bytes)?;
        self.node_desc = cstr.to_string_lossy().to_string();
        Ok(())
    }

    fn node_desc(&self) -> &str {
        &self.node_desc
    }

    fn guid(&self) -> u64 {
        self.guid
    }

    fn node_type(&self) -> NodeType {
        self.node_type.clone()
    }
}

impl Fabric {
    pub fn new() -> Fabric {
        Fabric {
            nodes: HashMap::new(),
            switches: HashMap::new(),
            routers: HashMap::new(),
            ports: HashMap::new(),
        }
    }

    pub fn discover(&mut self, ca_name: &str) -> Result<(), FabricError> {
        let ib_config = Box::new(ibnetdisc::sys::ibnd_config_t {
            max_smps: unsafe { MaybeUninit::<u32>::zeroed().assume_init() },
            show_progress: 0,
            max_hops: unsafe { MaybeUninit::<u32>::zeroed().assume_init() },
            debug: 0,
            timeout_ms: 100,
            retries: 3,
            flags: ibnetdisc::sys::IBND_CONFIG_MLX_EPI,
            mkey: 0,
            pad: unsafe { MaybeUninit::<[u8; 44]>::zeroed().assume_init() },
        });

        let ib_config_ptr = Box::into_raw(ib_config);
        let ca_name_c_str = CString::new(ca_name.to_string()).unwrap();
        let null_ptr: *const u8 = ptr::null();

        let nd_fabric_ptr: *mut ibnetdisc::sys::ibnd_fabric = unsafe {
            ibnetdisc::sys::ibnd_discover_fabric(
                ca_name_c_str.as_ptr() as *mut i8,
                1,
                null_ptr as *mut ibnetdisc::sys::portid,
                ib_config_ptr,
            )
        };

        if nd_fabric_ptr.is_null() {
            return Err(FabricError::DiscoveryError);
        }

        let nd_fabric = unsafe { Box::from_raw(nd_fabric_ptr) };
        let _ib_config = unsafe { Box::from_raw(ib_config_ptr) };

        let r = self.add_nodes(&nd_fabric);
        if let Err(_) = r {
            return Err(FabricError::DiscoveryError);
        }

        unsafe { ibnd_destroy_fabric(Box::into_raw(nd_fabric)) };

        return Ok(());
    }

    pub fn add_nodes(&mut self, ibnd_fabric: &Box<ibnd_fabric>) -> Result<(), FabricError> {
        if ibnd_fabric.nodes.is_null() {
            return Err(FabricError::NullPointerError);
        }

        //Ports can have multiple refs
        let mut next = ibnd_fabric.nodes;

        while !next.is_null() {
            let node: &ibnd_node = unsafe { &*next };

            //Create a Node
            let r = Node::from_nd_node(next);

            //Create the Vec for the Node's ports
            let mut rc_ports: Vec<Rc<RefCell<Port>>> = Vec::new();

            if let Ok(n) = r {
                let guid = n.guid.clone();
                // Check if the node already exists in self.nodes
                let node_rc = if let Some(existing_node) = self.nodes.get(&guid) {
                    existing_node.clone()
                } else {
                    // Create a new node and insert it
                    let rc_node = Rc::new(RefCell::new(n));
                    self.nodes.insert(guid, rc_node.clone());
                    rc_node
                };

                //Fetch the node's ports
                let r = Node::get_nd_ports(node, self);
                let mut rc_ref: RefMut<Node> = RefCell::borrow_mut(&node_rc);

                if let Ok(mut ports) = r {
                    //Remove each Port from the Vec returned and add it's parent.
                    for rc_port in ports.drain(..) {
                        let mut port = RefCell::borrow_mut(&rc_port);

                        // Update or insert into self.ports
                        if let std::collections::hash_map::Entry::Occupied(mut e) =
                            self.ports.entry((port.guid, port.number))
                        {
                            // Update existing port
                            *e.get_mut() = rc_port.clone();
                        } else {
                            self.ports.insert((port.guid, port.number), rc_port.clone());
                        }

                        //Set the remote port's remote port to this node's port.
                        if let Some(remote_port_rc) = &port.remote_port {
                            let rp_res = &remote_port_rc.upgrade();
                            if let Some(rp_cell) = rp_res {
                                let mut remote_port: RefMut<Port> = RefCell::borrow_mut(rp_cell);
                                remote_port.remote_port = Some(Rc::downgrade(&rc_port));
                            }
                        }

                        rc_ports.push(rc_port.clone());

                        //Set the parent.
                        port.parent = Some(Rc::downgrade(&node_rc));
                    }
                    rc_ref.ports = Some(rc_ports);
                }
            }
            next = node.next;
        }

        Ok(())
    }
}
