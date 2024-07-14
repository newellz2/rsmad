
#[cfg(test)]
mod tests {

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
}