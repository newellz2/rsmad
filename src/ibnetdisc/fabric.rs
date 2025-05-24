use std::{cell::{RefCell, RefMut}, collections::{hash_map::Entry, HashMap}, error::Error, ffi::CString, fmt, mem::MaybeUninit, ptr, rc::{Rc, Weak}};

use crate::ibmad;

use super::{node::{Node, NodeType}, port::{Port, PortPerfcounter}, sys};

#[derive(Debug)]
pub enum FabricError {
    NullPointerError,
    UnknownNodeTypeError,
    DiscoveryError,
    PortDiscoveryError,
    PortAlreadyDiscoveredError,
    PortNotFound,
    OriginSameAsRemotePortError,
    NoPortError,
}

impl fmt::Display for FabricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FabricError::NullPointerError => write!(f, "Null pointer"),
            FabricError::UnknownNodeTypeError => write!(f, "Unknown node type"),
            FabricError::DiscoveryError => write!(f, "Discovery failed"),
            FabricError::PortDiscoveryError => write!(f, "Port discovery failed"),
            FabricError::PortAlreadyDiscoveredError => write!(f, "Port already discovered"),
            FabricError::PortNotFound => write!(f, "Port not found"),
            FabricError::OriginSameAsRemotePortError => {
                write!(f, "The origin port is the same as the remote port")
            },
            FabricError::NoPortError => write!(f, "No port"),
        }
    }
}

impl Error for FabricError {}

#[derive(Debug)]
pub struct Fabric {
    pub ports: HashMap<(u64, i32), Rc<RefCell<Port>>>,
    pub nodes: HashMap<u64, Rc<RefCell<Node>>>,
    pub adapters: HashMap<u64, Weak<RefCell<Node>>>,
    pub switches: HashMap<u64, Weak<RefCell<Node>>>,
    pub hca_name: String,
    pub ib_port: Rc<ibmad::IBMadPort>,
}


impl Fabric {
    pub fn new(hca_name: &str) -> Fabric {

        let mgmt_classes = [ 
                            ibmad::sys::MAD_CLASSES_IB_SMI_DIRECT_CLASS, 
                            ibmad::sys::MAD_CLASSES_IB_SMI_CLASS, 
                            ibmad::sys::MAD_CLASSES_IB_SA_CLASS,
                            ibmad::sys::MAD_CLASSES_IB_PERFORMANCE_CLASS,
                            ];

        let port = ibmad::mad_rpc_open_port(hca_name, &mgmt_classes).unwrap();

        Fabric {
            nodes: HashMap::new(),
            ports: HashMap::new(),
            adapters: HashMap::new(),
            switches: HashMap::new(),
            hca_name: hca_name.to_string(),
            ib_port: port.into(),
        }
    }

    pub fn discover(&mut self, port_number: i32, timeout: u32, retries: u32, mkey: u64, max_smps: u32, max_hops: u32, debug: u32) -> Result<(), FabricError> {
        let ib_config = Box::new(sys::ibnd_config_t {
            max_smps: max_smps,
            show_progress: 0,
            max_hops: max_hops,
            debug: debug,
            timeout_ms: timeout,
            retries: retries,
            flags: sys::IBND_CONFIG_MLX_EPI,
            mkey: mkey,
            pad: unsafe { MaybeUninit::<[u8; 44]>::zeroed().assume_init() },
        });

        let ib_config_ptr = Box::into_raw(ib_config);
        let ca_name_c_str = CString::new(self.hca_name.as_str()).unwrap();
        let null_ptr: *const u8 = ptr::null();

        let nd_fabric_ptr: *mut sys::ibnd_fabric = unsafe {
            sys::ibnd_discover_fabric(
                ca_name_c_str.as_ptr() as *mut i8,
                port_number,
                null_ptr as *mut sys::portid,
                ib_config_ptr,
            )
        };

        let _ib_config = unsafe { Box::from_raw(ib_config_ptr) };

        if nd_fabric_ptr.is_null() {
            return Err(FabricError::DiscoveryError);
        }

        let nd_fabric: Box<_> = unsafe { Box::from_raw(nd_fabric_ptr) };

        let r = self.add_nodes(&nd_fabric);
        if let Err(_) = r {
            return Err(FabricError::DiscoveryError);
        }

        let nd_fabric_ptr = Box::into_raw(nd_fabric);
        unsafe { sys::ibnd_destroy_fabric(nd_fabric_ptr) };

        //let nd_fabric: Box<_> = unsafe { Box::from_raw(nd_fabric_ptr) };

        return Ok(());
    }

    pub fn add_nodes(&mut self, ibnd_fabric: &Box<sys::ibnd_fabric>) -> Result<(), FabricError> {
        if ibnd_fabric.nodes.is_null() {
            return Err(FabricError::NullPointerError);
        }

        let mut next = ibnd_fabric.switches;

        while !next.is_null() {
            let node: &sys::ibnd_node = unsafe { &*next };

            let r = Node::from_nd_node(next);
            let mut rc_ports: Vec<Rc<RefCell<Port>>> = Vec::new();

            if let Ok(n) = r {
                let guid = n.guid.clone();
                let node_rc = if let Some(existing_node) = self.nodes.get(&guid) {
                    existing_node.clone()
                } else {
                    let rc_node = Rc::new(RefCell::new(n));
                    
                    self.nodes.insert(guid, rc_node.clone());

                    match rc_node.borrow().node_type {
                        NodeType::CA => {
                            self.adapters.insert(guid, Rc::downgrade(&rc_node));
                        }
                        NodeType::SWITCH => {
                            self.switches.insert(guid, Rc::downgrade(&rc_node));
                        }
                        _ => {}
                    };

                    rc_node
                };

                //Fetch the node's ports
                let r = Node::get_nd_ports(node, self);

                let mut rc_ref: RefMut<Node> = RefCell::borrow_mut(&node_rc);

                if let Ok(mut ports) = r {

                    //Remove each Port from the Vec returned and add it's parent.
                    for port_rc in ports.drain(..) {
                        let mut port = RefCell::borrow_mut(&port_rc);

                        // Update or insert into self.ports
                        if let Entry::Occupied(mut e) =
                            self.ports.entry((port.guid, port.number))
                        {
                            *e.get_mut() = port_rc.clone();
                        } else {
                            self.ports.insert((port.guid, port.number), port_rc.clone());
                        }

                        //Set the remote port's remote port to this node's port.
                        //Set a CA's LID and Port
                        if let (Some(remote_node_weak), Some(remote_port_weak)) = (&port.remote_node, &port.remote_port) {
                            let remote_node_opt = &remote_node_weak.upgrade();
                            let remote_port_opt = &remote_port_weak.upgrade();

                            if let (Some(remote_node_cell), Some(remote_port_cell)) = (remote_node_opt, remote_port_opt) {
                                let mut remote_port: RefMut<Port> = RefCell::borrow_mut(remote_port_cell);
                                let mut remote_node  = RefCell::borrow_mut(remote_node_cell);

                                remote_node.lid = remote_port.base_lid;
                                remote_port.remote_port = Some(Rc::downgrade(&port_rc));

                                match remote_node.node_type {
                                    NodeType::CA => {
                                            remote_port.remote_node = Some(Rc::downgrade(&node_rc));
                                            remote_node.ports = Some(vec![
                                                remote_port_cell.clone()
                                            ]);
                                        
                                    }
                                    _ =>{ }
                                }
                            }
                        }

                        rc_ports.push(port_rc.clone());
                        port.parent = Some(Rc::downgrade(&node_rc));
                    }
                    rc_ref.ports = Some(rc_ports);
                }
            }
            next = node.next;
        }

        Ok(())
    }

    pub fn get_port_perfcounter(&self, port_info: (u64, i32)) -> Result<PortPerfcounter, FabricError> {

        if let Some(port) = self.ports.get(&port_info) {

            let port_perf = PortPerfcounter{
                port: Rc::downgrade(&port.clone()),
                ib_port: Rc::downgrade(&self.ib_port),
                msecs_wait: 0,
            };

            return Ok(port_perf);

        }

        Err(FabricError::PortNotFound)

    }
}
