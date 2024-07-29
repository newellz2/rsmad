use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, error::Error, ffi::{c_void, CStr, CString, FromBytesUntilNulError}, fmt, mem::MaybeUninit, ptr, rc::{Rc, Weak}, slice};
use crate::{ibmad, ibnetdisc};

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
}

impl fmt::Display for FabricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FabricError::NullPointerError => write!(f, "Null pointer"),
            FabricError::UnknownNodeTypeError => write!(f, "Unknown node type"),
            FabricError::DiscoveryError => write!(f, "Discovery failed"),
            FabricError::PortDiscoveryError => write!(f, "Port discovery failed"),

        }
    }
}

impl Error for FabricError {} 

#[derive(Debug)]
pub struct Fabric {
    pub switches: Vec<Node>,
    pub cas: Vec<Node>,
    pub routers: Vec<Node>,
    pub guids_lids: HashMap<u64, u16>,
    pub lids_guids: HashMap<u16, u64>
}

#[derive(Default, Debug, Clone)]
pub struct Node {
    pub guid: u64,
    pub node_desc: String,
    pub node_type: NodeType,
    pub smalid: u16,
    pub ports: Option<Vec<Port>>,
    pub dev_id: u32,
    pub vendor_id: u32,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub number: i32,
    pub phys_state: u32,
    pub logical_state: u32,
    pub base_lid: u16,
    pub remote_port: Option<Box<Port>>,
    pub remote_node: Option<Box<Node>>,
    pub switch: Option<Weak<Node>>,
}

impl Node {
    pub fn rc() -> Rc<Self>{
        Rc::new(Node::default())
    }
}
pub trait IBNode {
    fn set_node_desc(&mut self, node_desc_bytes: &[u8]) ->  Result<(), FromBytesUntilNulError>;
    fn node_desc(&self) ->  &str;
    fn guid(&self) ->  u64;
    fn node_type(&self) -> NodeType;
}

impl IBNode for Node {
    fn set_node_desc(&mut self, node_desc_bytes: &[u8]) ->  Result<(), FromBytesUntilNulError> {
        let cstr = CStr::from_bytes_until_nul(node_desc_bytes)?;
        self.node_desc = cstr.to_string_lossy().to_string();
        Ok(())
    }

    fn node_desc(&self) ->  &str {
        &self.node_desc
    }

    fn guid(&self) ->  u64 {
        self.guid
    }

    fn node_type(&self) -> NodeType {
        self.node_type.clone()
    }
}


impl Fabric {
    pub fn new() -> Fabric{
        Fabric{
            cas: Vec::new(),
            switches: Vec::new(),
            routers: Vec::new(),
            guids_lids: HashMap::new(),
            lids_guids: HashMap::new(),
        }
    }

    pub fn discover(&mut self, ca_name: &str) -> Result<(), FabricError> {

        let ib_config = Box::new(ibnetdisc::sys::ibnd_config_t{
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
                ib_config_ptr)
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

        return Ok(())
    }

    fn read_port(nd_port: &ibnd_port, read_remote: bool) -> Result<Port, FabricError> {

        let remote_port: Option<Box<Port>> = None;
        let remote_node: Option<Box<Node>> = None;

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

        let mut port = Port {
            number: nd_port.portnum,
            phys_state: phys_state,
            logical_state: logical_state,
            base_lid: nd_port.base_lid,
            remote_port: remote_port,
            remote_node: remote_node,
            switch: None,
        };

        if !nd_port.remoteport.is_null() && read_remote {
            let nd_remote_port = unsafe { &*nd_port.remoteport };

            if let Ok(remote_port) = Fabric::read_port(nd_remote_port, false) {
                port.remote_port = Some(
                    Box::new(remote_port)
                )
            }

            if let Ok(remote_node) = Fabric::read_node(nd_remote_port.node, false) {
                port.remote_node = Some(
                    Box::new(remote_node)
                )
            }
        };
        
        Ok(port)
    }

    fn read_ports(nd_node: &ibnd_node) -> Result<Vec<Port>, FabricError> {

        if nd_node.ports.is_null(){
            return Err(FabricError::NullPointerError);
        }

        let num_ports = nd_node.numports; 
        let port_ptrs: &[*mut ibnetdisc::sys::ibnd_port] = unsafe { slice::from_raw_parts(nd_node.ports, num_ports as usize) };
        let mut ports:Vec<Port> = Vec::new();

        for &port_ptr in port_ptrs { 
            if !port_ptr.is_null() {

                let nd_port = unsafe { &*port_ptr };

                let r = Fabric::read_port(nd_port, true);

                match r {
                    Ok(p) =>{
                        ports.push(p);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }

            }
        }

        Ok(ports)
        
    }

    fn read_node (nd_node_ptr: *mut ibnd_node, read_ports: bool) -> Result<Node, FabricError> {

        if nd_node_ptr.is_null(){
            return Err(FabricError::NullPointerError);
        }

        let nd_node = unsafe { &*nd_node_ptr };
        let mut new_node = Node::default();

        new_node.guid = nd_node.guid;
        
        //NodeDesc
        let unsigned_vec: Vec<u8> = nd_node.nodedesc.iter().map(|&x| x as u8).collect();
        let r = CStr::from_bytes_until_nul(&unsigned_vec) ;
        if let Ok(cstr) = r {
            new_node.node_desc = cstr.to_string_lossy().to_string();
        }

        new_node.dev_id = unsafe {
            mad_get_field( nd_node.info.as_ptr() as *mut c_void, 0, MAD_FIELDS_IB_NODE_DEVID_F)
        };

        new_node.vendor_id = unsafe {
            mad_get_field( nd_node.info.as_ptr() as *mut c_void, 0, MAD_FIELDS_IB_NODE_DEVID_F)
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

        if read_ports {
            if let Ok(ports) = Fabric::read_ports(nd_node) 
            {
                new_node.ports = Some(ports);
            }
        }

        Ok(new_node)
    }

    pub fn add_nodes(&mut self, ibnd_fabric: &Box<ibnd_fabric>) -> Result<(), FabricError>{

        if ibnd_fabric.nodes.is_null() {
            return Err(FabricError::NullPointerError);
        }

        let mut next = ibnd_fabric.nodes;
        while !next.is_null() {
            let node: &ibnd_node = unsafe { &*next };

            let r = Fabric::read_node(next, true);
            if let Ok(new_node) = r {

                match new_node.node_type {
                    NodeType::CA => {
                        self.cas.push(new_node);
                    }
                    NodeType::SWITCH => {
                        self.switches.push(new_node);
                    }
                    NodeType::ROUTER => {
                        self.routers.push(new_node);
                    }
                    _ => {
                        return Err(FabricError::UnknownNodeTypeError);
                    }
                }
            }

            next = node.next;
        }

        self.guids_lids = self.guids_lids();
        self.lids_guids = self.lids_guids();

        Ok(())
    }

    fn guids_lids(&self) -> HashMap<u64, u16>{
        let mut guid_lids: HashMap<u64, u16> = HashMap::new();

        for s in self.switches.iter() {
            guid_lids.insert(s.guid, s.smalid);

            if let Some(ports) = &s.ports {
                for po in ports.iter() {
                    if let (Some(r_port), Some(r_node)) = (&po.remote_port, &po.remote_node) {
                        if !guid_lids.contains_key(&r_node.guid){
                            guid_lids.insert(r_node.guid, r_port.base_lid);
                        }

                    }
                }
            }

        };
        return guid_lids;
    }

    fn lids_guids(&self) -> HashMap<u16, u64>{
        let mut lids_guids: HashMap<u16, u64> = HashMap::new();

        for s in self.switches.iter() {
            lids_guids.insert(s.smalid, s.guid);

            if let Some(ports) = &s.ports {
                for po in ports.iter() {
                    if let (Some(r_port), Some(r_node)) = (&po.remote_port, &po.remote_node) {
                        if r_port.base_lid != 65535 {
                            if !lids_guids.contains_key(&r_port.base_lid){
                                    lids_guids.insert(r_port.base_lid, r_node.guid);
                            }
                        }

                    }
                }
            }

        };
        return lids_guids;
    }
}