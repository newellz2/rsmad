use crate::ibmad::sys::{
    mad_get_field, 
    MAD_FIELDS_IB_NODE_DEVID_F, 
    MAD_FIELDS_IB_NODE_VENDORID_F, 
    MAD_NODE_TYPE_IB_NODE_CA, 
    MAD_NODE_TYPE_IB_NODE_ROUTER,
    MAD_NODE_TYPE_IB_NODE_SWITCH
};
use std::{cell::RefCell, ffi::{c_void, CStr}, rc::Rc, slice};
use super::{fabric::{Fabric, FabricError}, port::Port, sys::{self, ibnd_node}};

#[repr(i32)]
#[derive(Debug, Default, Copy, Clone)]
pub enum NodeType {
    #[default]
    UNKNOWN = 0,
    CA = 1,
    SWITCH = 2,
    ROUTER = 3,
}

#[derive(Default, Debug, Clone)]
pub struct Node {
    pub guid: u64,
    pub lid: u16,
    pub node_desc: String,
    pub node_type: NodeType,
    pub smalid: u16,
    pub ports: Option<Vec<Rc<RefCell<Port>>>>,
    pub dev_id: u32,
    pub vendor_id: u32,
}


impl Node {
    pub fn rc() -> Rc<Self> {
        Rc::new(Node::default())
    }

    //Get the ports associated with the node
    //Set the parent
    pub fn get_nd_ports(
        nd_node: &ibnd_node,
        fabric: &mut Fabric,
    ) -> Result<Vec<Rc<RefCell<Port>>>, FabricError> {
        if nd_node.ports.is_null() {
            return Err(FabricError::NullPointerError);
        }

        let num_ports = nd_node.numports;
        let port_ptrs: &[*mut sys::ibnd_port] =
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
    pub fn from_nd_node(nd_node_ptr: *mut ibnd_node) -> Result<Node, FabricError> {
        if nd_node_ptr.is_null() {
            return Err(FabricError::NullPointerError);
        }

        let nd_node = unsafe { &*nd_node_ptr };
        let mut new_node = Node::default();

        new_node.guid = nd_node.guid;

        //NodeDesc
        let nodedesc_vec: Vec<u8> = nd_node.nodedesc.iter().map(|&x| x as u8).collect();
        let r = CStr::from_bytes_until_nul(&nodedesc_vec);
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