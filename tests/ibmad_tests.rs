
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn ibmad_send_drmad_success() {
        rsmad::umad::umad_init();

        let ca_names = rsmad::umad::umad_list_devices().unwrap();

        let ca_name = ca_names.first().unwrap();

        let ca = rsmad::umad::umad_get_ca(&ca_name).unwrap();
        println!("Name: {}", ca.name().unwrap());
        println!("Firmware: {}", ca.fw_ver().unwrap());
        println!("NodeGuid: 0x{}", rsmad::umad::format_u64_little_endian(ca.node_guid()));

        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name).unwrap();

        rsmad::ibmad::send_dr_nodeinfo_mad(port, "0,1,1,1,45", 200);

        rsmad::umad::umad_done();
    }

    #[test]
    fn ibmad_send_drmad_node_desc_success() {
        rsmad::umad::umad_init();

        let ca_names = rsmad::umad::umad_list_devices().unwrap();

        let ca_name = ca_names.first().unwrap();

        let ca = rsmad::umad::umad_get_ca(&ca_name).unwrap();
        println!("Name: {}", ca.name().unwrap());
        println!("Firmware: {}", ca.fw_ver().unwrap());
        println!("NodeGuid: 0x{}", rsmad::umad::format_u64_little_endian(ca.node_guid()));

        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name).unwrap();

        rsmad::ibmad::send_dr_node_desc_mad(port, "0,1,1,1,45", 200);

        rsmad::umad::umad_done();
    }

    #[test]
    fn ib_mad_pma_query_via_success() {
        rsmad::umad::umad_init();

        let ca_names = rsmad::umad::umad_list_devices().unwrap();

        let ca_name = ca_names.first().unwrap();

        let port = rsmad::ibmad::mad_rpc_open_port(&ca_name).unwrap();

        rsmad::ibmad::perf_query(port, 132, 5, 3000);

        rsmad::umad::umad_done();
    }

    #[test]
    fn ib_mad_pma_query_via_success_multithreaded() {
        rsmad::umad::umad_init();
    
        let ca_names = rsmad::umad::umad_list_devices().unwrap();
        let ca_name = ca_names.first().unwrap().clone();
    
        let mut handles = vec![];
        for _ in 0..100 {

            let ca_name_clone = ca_name.clone();
    
            let handle = std::thread::spawn(move || {
                let port = rsmad::ibmad::mad_rpc_open_port(&ca_name_clone).unwrap();
                rsmad::ibmad::perf_query(port, 132, 5, 3000);
            });
    
            handles.push(handle);
        }
    
        for handle in handles {
            handle.join().unwrap();
        }
    
        rsmad::umad::umad_done();
    }
}