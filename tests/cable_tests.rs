
#[cfg(test)]
mod tests {


    #[test]
    fn get_cable_info_success() {
        rsmad::umad::umad_init();

        let ca_name = "mlx5_0";
        let lid = 2;
        let portnum = 54;

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                let r = rsmad::umad::cable::get_cable_info(&port, lid, portnum, 0, 0, 20);

                if let Ok(ci) = r {
                    //Needs to support DDM
                    let data = ci.data;
                    let part = data[2].to_le_bytes();
                    println!("Cable Data: {:?}", data);
                    println!("Cable Data Bytes 8-15: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
                     part[0], part[1], part[2], part[3]);

                    let status = rsmad::umad::cable::get_cable_module_status(&ci);
                    println!("Cable Module Status: int={} bin={:b}", status, status);

                    let temp = rsmad::umad::cable::get_cable_temperature(&ci);
                    println!("Cable Temp: {:?}", temp);

                    let vcc = rsmad::umad::cable::get_cable_voltage(&ci);
                    println!("Cable Voltage: {:?}", vcc);
                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                //166-181 Serial Number
                let r = rsmad::umad::cable::get_cable_info(&port, lid, portnum, 0, 166, 20);
                if let Ok(ci) = r {
                    //Needs to support DDM
                    let serial = rsmad::umad::cable::get_serial_number(&ci);
                    println!("Cable Serial: {}", serial);
                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                //129-144 Vendor
                let r = rsmad::umad::cable::get_cable_info(&port, lid, portnum, 0, 129, 20);
                if let Ok(ci) = r {
                    let serial = rsmad::umad::cable::get_vendor_name(&ci);
                    println!("Cable Vendor: {}", serial);
                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                //Page 11h Byte 128 DPStatus 1->2->4
                let r = rsmad::umad::cable::get_cable_info(&port, lid, 45, 17, 128, 20);
                if let Ok(ci) = r {
                    let data = ci.data;
                    let part = data[0].to_le_bytes();
                    println!("Cable Data Page 10h 160o: {:?}", data);
                    println!("Cable Data Bytes 8-15: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
                     part[0], part[1], part[2], part[3]);
                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }

        rsmad::umad::umad_done();
    }

    #[test]
    fn get_cable_flags_high_temp_warn_success() {
        let page: u8 = 0;
        let offset: u16 = 0;
        let flags: u32 = (0x00040000 as u32).to_be();

        let cable_info = rsmad::umad::cable::CableInfo {
            i2c_device_address: rsmad::umad::cable::QSFP_SFP_DEVICE_ADDRESS,
            page_number: page.to_be(),
            device_address: offset.to_be(),
            res1: 0,
            size: (48 as u16).to_be(),
            res2: 0,
            data:  [
                0x0,
                0x0,
                flags,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
            ]
        };
    

        let cable_flags = rsmad::umad::cable::get_cable_flags(&cable_info);

        println!("Cable Data Bytes 8-11: '{:b}'",  cable_info.data[2].to_le());

        println!("Cable Flags: {:?}", cable_flags);
        assert_eq!(cable_flags.temp_mon_high_warn, true);

    }

    #[test]
    fn get_cable_flags_high_temp_alarm_success() {
        let page: u8 = 0;
        let offset: u16 = 0;
        let flags: u32 = (0x00010000 as u32 ).to_be();

        let cable_info = rsmad::umad::cable::CableInfo {
            i2c_device_address: rsmad::umad::cable::QSFP_SFP_DEVICE_ADDRESS,
            page_number: page.to_be(),
            device_address: offset.to_be(),
            res1: 0,
            size: (48 as u16).to_be(),
            res2: 0,
            data:  [
                0x0,
                0x0,
                flags,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
            ]
        };
    

        let cable_flags = rsmad::umad::cable::get_cable_flags(&cable_info);

        println!("Cable Flags: {:?}", cable_flags);
        assert_eq!(cable_flags.temp_mon_high_alarm, true);

    }

    #[test]
    fn get_cable_flags_all_success() {
        let page: u8 = 0;
        let offset: u16 = 0;
        let flags: u32 = (0xffffffff as u32 ).to_be(); //All Warnings and flags = 1

        let cable_info = rsmad::umad::cable::CableInfo {
            i2c_device_address: rsmad::umad::cable::QSFP_SFP_DEVICE_ADDRESS,
            page_number: page.to_be(),
            device_address: offset.to_be(),
            res1: 0,
            size: (48 as u16).to_be(),
            res2: 0,
            data:  [
                0x0,
                0x0,
                flags,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
            ]
        };
    

        let cable_flags = rsmad::umad::cable::get_cable_flags(&cable_info);

        println!("Cable Flags: {:?}", cable_flags);

        assert_eq!(cable_flags.datapath_firmware_error, true);
        assert_eq!(cable_flags.module_state_changed, true);

    }

    #[test]
    fn get_cable_flags_shift_test_success() {
        let page: u8 = 0;
        let offset: u16 = 0;

        for i in (0..11).rev() {
            let flags: u32 = (0x00 | 0x01 as u32).wrapping_shl(i+8).to_be();

            println!("BE Data: {:?}", flags.to_be_bytes());
            println!("LE Data: {:?}", flags.to_le_bytes());

            let cable_info = rsmad::umad::cable::CableInfo {
                i2c_device_address: rsmad::umad::cable::QSFP_SFP_DEVICE_ADDRESS,
                page_number: page.to_be(),
                device_address: offset.to_be(),
                res1: 0,
                size: (48 as u16).to_be(),
                res2: 0,
                data:  [
                    0x0,
                    0x0,
                    flags,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                ]
            };
        

            let cable_flags = rsmad::umad::cable::get_cable_flags(&cable_info);

            println!("Shift Cable Flags: {:?}", cable_flags);
        }
    }

    #[test]
    fn get_cable_info_page0_success() {
        rsmad::umad::umad_init();

        let ca_name = "mlx5_0";
        let lid = 2;
        let portnum = 54;

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                let r = rsmad::umad::cable::get_cable_info(&port, lid, portnum, 0, 0, 20);

                if let Ok(ci) = r {
                    //Needs to support DDM
                    let data = ci.data;
                    let part = data[2].to_le_bytes();
                    println!("Cable Data: {:?}", data);
                    println!("Cable Data Bytes 8-15: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
                     part[0], part[1], part[2], part[3]);

                    let status = rsmad::umad::cable::get_cable_module_status(&ci);
                    println!("Cable Module Status: int={} bin={:b}", status, status);

                    let temp = rsmad::umad::cable::get_cable_temperature(&ci);
                    println!("Cable Temp: {:?}", temp);

                    let vcc = rsmad::umad::cable::get_cable_voltage(&ci);
                    println!("Cable Voltage: {:?}", vcc);

                    let flags = rsmad::umad::cable::get_cable_flags(&ci);
                    println!("Cable Flags: {:?}", flags);
                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }
    }


    #[test]
    fn get_cable_info_page17_128_176_success() {
        rsmad::umad::umad_init();

        let ca_name = "mlx5_0";
        let lid = 2;
        let portnum = 54;

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                let r = rsmad::umad::cable::get_cable_info(&port, lid, portnum, 17, 128, 20);

                if let Ok(ci) = r {
                    //Needs to support DDM
                    let data = ci.data;
                    let part = data[1].to_le_bytes();
                    println!("Cable Data: {:?}", data);
                    println!("Cable Data Bytes 0-3: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
                     part[0], part[1], part[2], part[3]);

                    let part = data[7].to_le_bytes();
                    println!("Cable Data Bytes 160-163: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
                      part[0], part[1], part[2], part[3]);

                    let lane_spec_flags = rsmad::umad::cable::get_lane_specific_flags(&ci);
                    println!("{:?}", lane_spec_flags);
                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }
    }


    #[test]
    fn get_cable_info_page20_192_240_success() {
        rsmad::umad::umad_init();

        let ca_name = "mlx5_0";
        let lid = 2;
        let portnum = 54;

        match rsmad::umad::umad_open_port(ca_name, 1){
            Ok(port) => {
                let r = rsmad::umad::cable::get_cable_info(&port, lid, portnum, 20, 192, 20);

                if let Ok(ci) = r {
                    //Needs to support DDM
                    let data = ci.data;
                    println!("Page 20 Cable Data: {:?}", data);

                }
                let _ = rsmad::umad::umad_close_port(port);
            }
            Err(_err) =>{
                assert!(false, "Failed to open HCA: {:?}", ca_name);

            }
        }
    }
}