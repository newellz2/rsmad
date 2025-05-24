
#[cfg(test)]
mod tests {
    use rsmad::umad::{umad_close_port, umad_open_port};


    #[test]
    fn umad_init_done_success() {
        rsmad::umad::umad_init();

        rsmad::umad::umad_done();
        
    }

    #[test]
    fn umad_list_devices_success() {
        rsmad::umad::umad_init();

        let ca_names = rsmad::umad::umad_list_devices().unwrap();

        ca_names.iter().for_each(|c|{
            println!("Device: {}", c);
        });

        rsmad::umad::umad_done();
    }

    #[test]
    fn umad_get_ca_success() {
        rsmad::umad::umad_init();

        let ca_names = rsmad::umad::umad_list_devices().unwrap();

        ca_names.iter().for_each(|c|{
            let ca = rsmad::umad::umad_get_ca(&c).unwrap();
            println!("Name: {}", ca.name().unwrap());
            println!("Firmware: {}", ca.fw_ver().unwrap());
            println!("NodeGuid: 0x{}", rsmad::umad::format_u64_little_endian(ca.node_guid()));
        });

        rsmad::umad::umad_done();
    }

    #[test]
    fn umad_open_port_success() {
        rsmad::umad::umad_init();

        let ca_names = rsmad::umad::umad_list_devices().unwrap();

        ca_names.iter().for_each(|c|{
            let ca = rsmad::umad::umad_get_ca(&c).unwrap();
            let ca_name = ca.name().unwrap();
            let ca_fw = ca.fw_ver().unwrap();
            let guid = rsmad::umad::format_u64_little_endian(ca.node_guid());
            println!("Name: {}", ca_name);
            println!("Firmware: {}", ca_fw);
            println!("NodeGuid: 0x{}", guid);
            let result =  umad_open_port(ca_name, 1);
            assert!(result.is_ok(), "Failed to open port");

            if let Ok(port) = result {
                let _ = umad_close_port(port);
            }
        });

        rsmad::umad::umad_done();
    }

    #[test]
    fn umad_open_port_failed_success() {
        rsmad::umad::umad_init();

        let ca_name = "mlx99_0";

        match umad_open_port(ca_name, 1) {
            Ok(port) => {
                assert!(false, "Opened invalid HCA port: {:?}", port)

            }
            Err(err) => {
                assert!(matches!(err, rsmad::umad::UmadError::OpenCaPortError), "Failed to open port: {:?}", err)
            }
        }

        rsmad::umad::umad_done();

    }
}