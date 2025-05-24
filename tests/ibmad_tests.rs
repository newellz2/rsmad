
#[cfg(test)]
mod tests {

    //cargo test --package rsmad --test ibmad_tests -- tests::ibmad_send_drmad_success --show-output
    #[test]
    fn ibmad_send_dr_mad_success() {
        rsmad::umad::umad_init();

        let mgmt_classes = [ rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_DIRECT_CLASS, 
                             rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_CLASS, 
                             rsmad::ibmad::sys::MAD_CLASSES_IB_SA_CLASS
                            ];

        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap();
        let ca = rsmad::umad::umad_get_ca(&ca_name).unwrap();

        println!("HCA Name: {}", ca.name().unwrap());
        println!("HCA Firmware: {}", ca.fw_ver().unwrap());
        println!("HCA NodeGuid: 0x{}", rsmad::umad::format_u64_little_endian(ca.node_guid()));

        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name, &mgmt_classes).unwrap();
        let ni = rsmad::ibmad::send_dr_node_info_mad(&port, "0,1,1,1,45", 200).unwrap();
        println!("NodeInfo GUID: 0x{:x}", ni.guid);
        println!("{:?}", ni);

        rsmad::umad::umad_done();
    }

    #[test]
    fn ibmad_send_dr_mad_node_desc_success() {
        rsmad::umad::umad_init();

        let mgmt_classes = [ 
                            rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_DIRECT_CLASS, 
                            rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_CLASS, 
                            rsmad::ibmad::sys::MAD_CLASSES_IB_SA_CLASS,
                            ];

        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap();
        let ca = rsmad::umad::umad_get_ca(&ca_name).unwrap();

        println!("Name: {}", ca.name().unwrap());
        println!("Firmware: {}", ca.fw_ver().unwrap());
        println!("NodeGuid: 0x{}", rsmad::umad::format_u64_little_endian(ca.node_guid()));
        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name, &mgmt_classes).unwrap();
        let r = rsmad::ibmad::send_dr_node_desc_mad(&port, "0,1,1,1,45", 200);
        println!("NodeDesc: {:?}", r.unwrap());
        rsmad::umad::umad_done();
    }

    #[test]
    fn ibmad_send_lid_node_info_mad_success() {
        rsmad::umad::umad_init();

        let mgmt_classes = [ 
            rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_DIRECT_CLASS, 
            rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_CLASS, 
            rsmad::ibmad::sys::MAD_CLASSES_IB_SA_CLASS, 
            ];

        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap();
        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name, &mgmt_classes).unwrap();
        let r = rsmad::ibmad::send_lid_node_info_mad(&port, 132, 3000);
        println!("LID-Routed NodeInfo: {:?}", r.unwrap());
        rsmad::umad::umad_done();
    }

    #[test]
    fn ibmad_set_node_desc_success() {
        rsmad::umad::umad_init();

        let mgmt_classes = [ 
            rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_DIRECT_CLASS, 
            rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_CLASS, 
            rsmad::ibmad::sys::MAD_CLASSES_IB_SA_CLASS, 
            ];

        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap();
        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name, &mgmt_classes).unwrap();
        let _r = rsmad::ibmad::set_node_desc(&port, 2, 3000);
        rsmad::umad::umad_done();
    }

    #[test]
    fn ib_mad_pma_query_via_success() {
        rsmad::umad::umad_init();

        let mgmt_classes = [ 
                            rsmad::ibmad::sys::MAD_CLASSES_IB_SA_CLASS, 
                            rsmad::ibmad::sys::MAD_CLASSES_IB_PERFORMANCE_CLASS
                           ];

        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap();
        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name, &mgmt_classes).unwrap();
        let r = rsmad::ibmad::perfquery(&port, 2, 45, 0, 3000);
        println!("Extended Perf Counters: {:?}", r.unwrap());
        rsmad::umad::umad_done();
    }

    #[test]
    fn ib_mad_pma_query_via_success_multithreaded() {
        rsmad::umad::umad_init();

        let mgmt_classes = [ rsmad::ibmad::sys::MAD_CLASSES_IB_SA_CLASS, 
                                       rsmad::ibmad::sys::MAD_CLASSES_IB_PERFORMANCE_CLASS,
                                       rsmad::ibmad::sys::MAD_CLASSES_IB_SMI_CLASS, 
                                       ];

        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap().clone();
    
        let mut handles = vec![];
        for _ in 0..1 {

            let ca_name_clone = ca_name.clone();
            let mgmt_classes_clone = mgmt_classes.clone();
            let handle = std::thread::spawn(move || {
                let port = rsmad::ibmad::mad_rpc_open_port(&ca_name_clone, &mgmt_classes_clone).unwrap();
                let r = rsmad::ibmad::perfquery(&port, 520, 3, 0, 3000);
                println!("Extended Perf Counters: {:?}", r.unwrap())
            });
    
            handles.push(handle);
        }
    
        for handle in handles {
            handle.join().unwrap();
        }
        rsmad::umad::umad_done();

    }

}