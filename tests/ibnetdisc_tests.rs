
#[cfg(test)]
mod tests {
    use std::{ffi::{c_void, CStr, CString}, mem::MaybeUninit, ptr, slice};

    #[test]
    fn unsafe_ffi_ibnd_discover_fabric_success() {
        rsmad::umad::umad_init();


        let ib_config = Box::new(rsmad::ibnetdisc::sys::ibnd_config_t{
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
            let fabric_ptr: *mut rsmad::ibnetdisc::sys::ibnd_fabric = rsmad::ibnetdisc::sys::ibnd_discover_fabric(
                ca_name_c_str.as_ptr() as *mut i8, 
                1,
                null_ptr as *mut rsmad::ibnetdisc::sys::portid,
                ib_config_ptr);

            if !fabric_ptr.is_null() {
                fabric = Some(Box::from_raw(fabric_ptr));
            }
        };

        let _ib_config = unsafe { Box::from_raw(ib_config_ptr) };

        let f: Box<rsmad::ibnetdisc::sys::ibnd_fabric> = fabric.unwrap();

        if !f.nodes.is_null(){
        let node = unsafe { &*f.nodes };
        let mut next = node.next;

        while !next.is_null() {

            let node: &rsmad::ibnetdisc::sys::ibnd_node = unsafe { &*next };

            match node.type_ {
                i if i == rsmad::ibmad::sys::MAD_NODE_TYPE_IB_NODE_CA as i32 => {
                    let node_desc = unsafe { CStr::from_ptr(node.nodedesc.as_ptr()) };
                    
                    let dev_id: u32 = unsafe {
                        rsmad::ibmad::sys::mad_get_field( node.info.as_ptr() as *mut c_void, 0, rsmad::ibmad::sys::MAD_FIELDS_IB_NODE_DEVID_F)
                    };

                    let vendor_id: u32 = unsafe {
                        rsmad::ibmad::sys::mad_get_field( node.info.as_ptr() as *mut c_void, 0, rsmad::ibmad::sys::MAD_FIELDS_IB_NODE_DEVID_F)
                    };
                    

                    println!("HCA NodeDesc:{:?}, GUID:{:x}, SMALID:{}, DEVID:{}, VENDORID:{}", node_desc, node.guid, node.smalid, dev_id, vendor_id);

                }
                i if i == rsmad::ibmad::sys::MAD_NODE_TYPE_IB_NODE_SWITCH as i32 => {
                    let num_ports = node.numports; 
                    let port_ptrs: &[*mut rsmad::ibnetdisc::sys::ibnd_port] = unsafe { slice::from_raw_parts(node.ports, num_ports as usize) };
                    for &port_ptr in port_ptrs {  
                        if !port_ptr.is_null() {
                            let port = unsafe { &*port_ptr };

                            if !port.remoteport.is_null() {
                                let remoteport =  unsafe { &*port.remoteport };
                                let remotenode = unsafe { &*remoteport.node };

                                let nodedesc = unsafe { CStr::from_ptr(remotenode.nodedesc.as_ptr()) };

                                println!("Remote Port LID: {} PortNum: {} NodeDesc:{:?} GUID:{:x} TYPE:{} SMLID:{}, BASE_LID:{}", 
                                    port.base_lid, port.portnum, nodedesc, remotenode.guid, remotenode.type_, remotenode.smalid,
                                        remoteport.base_lid);
                            }   
                            
                            unsafe { 
                                let phys_state = rsmad::ibmad::sys::mad_get_field( port.info.as_ptr() as *mut c_void, 0, rsmad::ibmad::sys::MAD_FIELDS_IB_PORT_PHYS_STATE_F);
                                let logical_state = rsmad::ibmad::sys::mad_get_field( port.info.as_ptr() as *mut c_void, 0, rsmad::ibmad::sys::MAD_FIELDS_IB_PORT_STATE_F);
                                println!("PhyState: {}, PortState: {}", phys_state, logical_state);

                            };
                        }
                    }
                }
                _ => {

                }
            }

            next = node.next;
        }
    }
        rsmad::umad::umad_done();
    }

    #[test]
    fn fabric_add_nodes_success() {
        rsmad::umad::umad_init();

        let mut fabric = rsmad::ibnetdisc::Fabric::new();
        let r = fabric.discover("mlx5_0");
        match r {
            Ok(_) =>{
                println!("Discovery completed successfully");
            }
            Err(err) => {
                println!("Discovery failed: {}", err);

            }   
        }

        for s in fabric.switches.iter() {

            println!("Switch: {}", s.node_desc);
            if let Some(ports) = &s.ports {
                for po in ports.iter() {
                    print!("\t[{:0>2}] 0x{:x}", po.number, s.guid);
                    if let (Some(r_port), Some(r_node)) = (&po.remote_port, &po.remote_node) {
                        print!(" - [{:0>2}] LID:{} {} {:?}", r_port.number, r_port.base_lid, r_node.node_desc, r_node.node_type );
                    } else {
                        print!(" - {} {}", po.logical_state, po.phys_state);
                    }
                    print!("\n");
                }
            }

        };

        for c in fabric.cas.iter() {

            print!("CA: 0x{:x} {} {}", c.guid, c.node_desc, c.smalid);
            print!("\n");

        };

        for (r , k) in fabric.guids_lids.into_iter(){
            println!("0x{:x} {}", r, k);
        }

        for (r , k) in fabric.lids_guids.into_iter(){
            println!("0x{:x} {}", r, k);
        }

        rsmad::umad::umad_done();
    }
}