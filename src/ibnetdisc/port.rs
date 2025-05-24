use std::{cell::RefCell, ffi::c_void, rc::{Rc, Weak}, thread, time::Duration};
use crate::ibmad::{self};
use super::{fabric::{Fabric, FabricError}, node::Node, sys::ibnd_port};

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


pub struct PortPerfcounter {
    pub port: Weak<RefCell<Port>>,
    pub ib_port: Weak<ibmad::IBMadPort>,
    pub msecs_wait: u64,
}

impl PortPerfcounter {
    pub fn set_wait(&mut self, msecs: u64) {
        self.msecs_wait = msecs;
    }
}

impl Iterator for PortPerfcounter {
    type Item = ibmad::perf::ExtPerfCounters; 

    fn next(&mut self) -> Option<Self::Item> {

        if let (Some(port_rc), Some(ib_port)) = (self.port.upgrade(), self.ib_port.upgrade()) {
            let port_ref = RefCell::borrow(&port_rc);
            thread::sleep(Duration::from_millis(self.msecs_wait)); 
            if let Ok(perfctrs) = ibmad::perfquery(&ib_port, port_ref.base_lid.into(), port_ref.number, 0,  200){
                return Some(perfctrs);
            }

        }
        
        None

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

    pub fn swap_port(port: &Port) -> Result<Rc<RefCell<Port>>, FabricError> {
            if let Some(remote_port_rc) = &port.remote_port {
                if let Some(remote_port_rc) = remote_port_rc.upgrade(){
                    let remote_port_ref = RefCell::borrow(&remote_port_rc);
                    let new_port: Rc<RefCell<Port>> = Rc::new(RefCell::new(Port {
                        guid: remote_port_ref.guid,
                        number: remote_port_ref.number,
                        phys_state: remote_port_ref.phys_state,
                        logical_state: remote_port_ref.logical_state,
                        base_lid: remote_port_ref.base_lid,
                        remote_port: None,
                        remote_node: None,
                        parent: None,
                    }));
                return Ok(new_port);
            }
            }

        Err(FabricError::NoPortError)
    }
}