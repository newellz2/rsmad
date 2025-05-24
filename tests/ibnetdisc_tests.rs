#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell, ffi::{c_void, CStr, CString}, mem::{MaybeUninit}, ptr, rc::Rc, slice
    };

    #[test]
    fn unsafe_ffi_ibnd_discover_fabric_success() {
        rsmad::umad::umad_init();

        let ib_config = Box::new(rsmad::ibnetdisc::sys::ibnd_config_t {
            max_smps: unsafe { MaybeUninit::<u32>::zeroed().assume_init() },
            show_progress: 0,
            max_hops: unsafe { MaybeUninit::<u32>::zeroed().assume_init() },
            debug: 0,
            timeout_ms: 100,
            retries: 3,
            flags: rsmad::ibnetdisc::sys::IBND_CONFIG_MLX_EPI,
            mkey: 0,
            pad: unsafe { MaybeUninit::<[u8; 44]>::zeroed().assume_init() },
        });

        let ib_config_ptr = Box::into_raw(ib_config);
        let mut fabric: Option<Box<rsmad::ibnetdisc::sys::ibnd_fabric>> = None;
        let ca_name_c_str = CString::new("mlx5_0").unwrap();
        let null_ptr: *const u8 = ptr::null();

        unsafe {
            let fabric_ptr: *mut rsmad::ibnetdisc::sys::ibnd_fabric =
                rsmad::ibnetdisc::sys::ibnd_discover_fabric(
                    ca_name_c_str.as_ptr() as *mut i8,
                    1,
                    null_ptr as *mut rsmad::ibnetdisc::sys::portid,
                    ib_config_ptr,
                );

            if !fabric_ptr.is_null() {
                fabric = Some(Box::from_raw(fabric_ptr));
            }
        };

        let _ib_config = unsafe { Box::from_raw(ib_config_ptr) };

        let f: Box<rsmad::ibnetdisc::sys::ibnd_fabric> = fabric.unwrap();

        if !f.nodes.is_null() {
            let node = unsafe { &*f.nodes };
            let mut next = node.next;

            while !next.is_null() {
                let node: &rsmad::ibnetdisc::sys::ibnd_node = unsafe { &*next };

                match node.type_ {
                    i if i == rsmad::ibmad::sys::MAD_NODE_TYPE_IB_NODE_CA as i32 => {
                        let node_desc = unsafe { CStr::from_ptr(node.nodedesc.as_ptr()) };

                        let dev_id: u32 = unsafe {
                            rsmad::ibmad::sys::mad_get_field(
                                node.info.as_ptr() as *mut c_void,
                                0,
                                rsmad::ibmad::sys::MAD_FIELDS_IB_NODE_DEVID_F,
                            )
                        };

                        let vendor_id: u32 = unsafe {
                            rsmad::ibmad::sys::mad_get_field(
                                node.info.as_ptr() as *mut c_void,
                                0,
                                rsmad::ibmad::sys::MAD_FIELDS_IB_NODE_DEVID_F,
                            )
                        };

                        println!(
                            "HCA NodeDesc:{:?}, GUID:{:x}, SMALID:{}, DEVID:{}, VENDORID:{}",
                            node_desc, node.guid, node.smalid, dev_id, vendor_id
                        );
                    }
                    i if i == rsmad::ibmad::sys::MAD_NODE_TYPE_IB_NODE_SWITCH as i32 => {
                        let num_ports = node.numports;
                        let port_ptrs: &[*mut rsmad::ibnetdisc::sys::ibnd_port] =
                            unsafe { slice::from_raw_parts(node.ports, num_ports as usize) };
                        for &port_ptr in port_ptrs {
                            if !port_ptr.is_null() {
                                let port = unsafe { &*port_ptr };

                                if !port.remoteport.is_null() {
                                    let remoteport = unsafe { &*port.remoteport };
                                    let remotenode = unsafe { &*remoteport.node };

                                    let nodedesc =
                                        unsafe { CStr::from_ptr(remotenode.nodedesc.as_ptr()) };

                                    println!("Remote Port LID: {} PortNum: {} NodeDesc:{:?} GUID:{:x} TYPE:{} SMLID:{}, BASE_LID:{}", 
                                    port.base_lid, port.portnum, nodedesc, remotenode.guid, remotenode.type_, remotenode.smalid,
                                        remoteport.base_lid);
                                }

                                unsafe {
                                    let phys_state = rsmad::ibmad::sys::mad_get_field(
                                        port.info.as_ptr() as *mut c_void,
                                        0,
                                        rsmad::ibmad::sys::MAD_FIELDS_IB_PORT_PHYS_STATE_F,
                                    );
                                    let logical_state = rsmad::ibmad::sys::mad_get_field(
                                        port.info.as_ptr() as *mut c_void,
                                        0,
                                        rsmad::ibmad::sys::MAD_FIELDS_IB_PORT_STATE_F,
                                    );
                                    println!(
                                        "PhyState: {}, PortState: {}",
                                        phys_state, logical_state
                                    );
                                };
                            }
                        }
                    }
                    _ => {}
                }

                next = node.next;
            }
        }
        rsmad::umad::umad_done();
    }

    #[test]
    fn fabric_add_nodes_success() {
        rsmad::umad::umad_init();

        let port_number = 1;
        let timeout = 100;
        let retries = 3;
        let mkey = 0;
        let max_smps = 0;
        let max_hops = 0;
        let debug = 1;

        let mut fabric = rsmad::ibnetdisc::fabric::Fabric::new("mlx5_0");
        let r = fabric.discover(port_number,timeout,retries,mkey,max_smps,max_hops,debug);
        match r {
            Ok(_) => {
                println!("Discovery completed successfully");
            }
            Err(err) => {
                println!("Discovery failed: {}", err);
            }
        }

        //18188380844304618496
        //18188380844304618560
        let sw = fabric.nodes.get(&18188380844304618560);

        if let Some(switch_rc) = sw {
            let switch_ref = RefCell::borrow_mut(switch_rc);
            println!("{:?}", switch_ref.node_desc);
            if let Some(port_ref) = &switch_ref.ports {
                for p in port_ref {
                    let local_port: std::cell::Ref<rsmad::ibnetdisc::port::Port> = RefCell::borrow(p);
                    println!("LocalPort: {:?}", local_port);

                    if let Some(rprp) = &local_port.remote_port {
                        let r: Option<Rc<RefCell<rsmad::ibnetdisc::port::Port>>> = rprp.upgrade();
                        if let Some(rp) = r {
                            let rp_ref = RefCell::borrow(&rp);
                            println!("  Remote Port: {:?}", rp_ref)
                        }
                    }
                    if let Some(remote_node) = &local_port.remote_node {
                        let r = remote_node.upgrade();
                        if let Some(rp) = r {
                            let rp_ref = RefCell::borrow(&rp);
                            println!("  Remote Node: {:?}: {:?}", rp_ref.node_desc, rp_ref.guid);
                        } else {
                        }
                    }
                }
            }
        }

        println!("Port Count: {:?}", fabric.ports.len());
        println!("Node Count: {:?}", fabric.nodes.len());

        for (_i, rc_node) in fabric.nodes.iter() {
            let node = RefCell::borrow(&rc_node);
            println!("Node: {}, LID: {}, TYPE: {:?}", node.node_desc, node.lid, node.node_type);

            if let Some(ports) = &node.ports {
                for rc_port in ports {
                    let port = RefCell::borrow(&rc_port);

                    print!("\t[{:0>2}] 0x{:x}", port.number, node.guid);
                    if let (Some(weak_remote_port), Some(weak_remote_node)) =
                        (&port.remote_port, &port.remote_node)
                    {
                        if let (Some(remote_port), Some(remote_node)) =
                            (weak_remote_port.upgrade(), weak_remote_node.upgrade())
                        {
                            let rp = RefCell::borrow(&remote_port);
                            let rn = RefCell::borrow(&remote_node);

                            print!(
                                " - [{:0>2}] LID:{} {} {:?}",
                                rp.number, rn.lid, rn.node_desc, rn.node_type
                            );
                        } else {
                            print!(" - (Remote port or node not available)");
                        }
                    } else {
                        print!(" - {} {}", port.logical_state, port.phys_state);
                    }
                    println!("");
                }
            }
        }

        rsmad::umad::umad_done();
    }

    #[test]
    fn fabric_ports_perfquery_success() {
        rsmad::umad::umad_init();

        let mut fabric = rsmad::ibnetdisc::fabric::Fabric::new("mlx5_0");
        let r = fabric.discover(1,1000,3,0,0,0,0);
        match r {
            Ok(_) => {
                println!("Discovery completed successfully");
            }
            Err(err) => {
                println!("Discovery failed: {}", err);
            }
        }

        //LEAF02 Port 28
        if let Some(l2_rc) = fabric.nodes.get(&18188380844304618560) {
            let l2_rc = RefCell::borrow(&l2_rc);

            if let Some(ports) = &l2_rc.ports {

                for i in 0..ports.len() {
                    let pctr_result = fabric.get_port_perfcounter((18188380844304618560, i.try_into().unwrap()));
                    if let Ok(mut pctr) = pctr_result {
                        pctr.set_wait(100);
                        let r = pctr.by_ref().take(2);
                        for (m , p) in r.enumerate() {
                            println!("{} {:?} {:?}",
                            i,
                            p.counters.get("rcv_pkts"),
                            p.counters.get("xmt_pkts")
                        );
                        }
                    }
                }
            }
        }

        rsmad::umad::umad_done();
    }


}
