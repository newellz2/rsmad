use std::{ffi::{CStr, CString}, mem::MaybeUninit};
use thiserror::Error;
use byteorder::{LittleEndian, BigEndian, WriteBytesExt};

use crate::umad::{self};

use umad::sys::*;

pub fn format_u64_little_endian(value: u64) -> String {
    let mut buffer = Vec::new();
    buffer.write_u64::<LittleEndian>(value).unwrap();
    let formatted_bytes: Vec<String> = buffer.iter()
        .map(|byte| format!("{:02x}", byte)) 
        .collect();
    formatted_bytes.join("") 
}

pub fn format_u64_big_endian(value: u64) -> String {
    let mut buffer = Vec::new();
    buffer.write_u64::<BigEndian>(value).unwrap();
    let formatted_bytes: Vec<String> = buffer.iter()
        .map(|byte| format!("{:02x}", byte)) 
        .collect();
    formatted_bytes.join("") 
}

#[derive(Error, Debug)]
pub enum UmadError {
    #[error("Unable to retrieve device list.")]
    DeviceListError,
    #[error("Unable to retrieve CA information.")]
    GetCaError,
    #[error("CString creation failed.")]
    CStringError(#[from] std::ffi::NulError),
    #[error("Invalid C string encountered.")]
    InvalidCString,
    #[error("Unable to open CA port.")]
    OpenCaPortError,
    #[error("Unable to close CA port.")]
    CloseCaPortError,
    #[error("Unable to register agent.")]
    RegisterMadAgentError,
    #[error("Unable to unregister agent.")]
    UnregisterMadAgentError,
    #[error("Unable to send MAD.")]
    SendFailure,
    #[error("Unable to receive MAD.")]
    RecvFailure
}


#[derive(Clone, Debug)]
pub struct UmadPort {
    pub port_id: std::os::raw::c_int,
    pub hca: String,
}

pub struct UmadCa {
    pub umad_ca: umad_ca_t,  // Assuming umad_ca_t is your C struct
}

impl UmadCa {
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> { 
        let c_str = unsafe { CStr::from_ptr(self.umad_ca.ca_name.as_ptr()) };
        c_str.to_str() 
    }

    pub fn fw_ver(&self) -> Result<&str, std::str::Utf8Error> { 
        let c_str = unsafe { CStr::from_ptr(self.umad_ca.fw_ver.as_ptr()) };
        c_str.to_str() 
    }

    pub fn node_guid(&self) -> u64 {
        self.umad_ca.node_guid
    }
}

pub fn umad_init() -> i32 {
    unsafe { 
        let result = umad::sys::umad_init();
        return result;
    };
}

pub fn umad_done() -> i32 {
    unsafe { 
        let result  = umad::sys::umad_done();
        return result;
    };
}

pub fn umad_list_devices() -> Result<Vec<String>, UmadError> {
    let mut ca_names = Vec::new();

    let device_list_ptr = unsafe { umad_get_ca_device_list() };
    if device_list_ptr.is_null() {
        return Err(UmadError::DeviceListError); 
    }

    let mut device_ptr = device_list_ptr;
    while !device_ptr.is_null() {
        // Safely convert C string to Rust String
        let device_name = unsafe { 
            CStr::from_ptr((*device_ptr).ca_name)
                .to_string_lossy()
                .to_string()
        };
        ca_names.push(device_name);

        device_ptr = unsafe { (*device_ptr).next };
    }

    unsafe { umad_free_ca_device_list(device_list_ptr) };

    Ok(ca_names)
}

pub fn umad_get_ca(ca_name: &str) -> Result<UmadCa, UmadError> {
    let ca_name_c_str: CString = CString::new(ca_name)?;
    let ca = Box::new(umad_ca_t {
        ca_name: [0; 20], 
        node_guid: 0, 
        node_type: 0,
        numports: 0,
        fw_ver: [0; 20],
        ca_type: [0; 40],
        hw_ver: [0; 20],
        system_guid: 0,
        ports: unsafe { MaybeUninit::<[*mut umad_port; 10]>::zeroed().assume_init() }
    });

    let ca_ptr = Box::into_raw(ca);

    let r = unsafe { umad::sys::umad_get_ca(ca_name_c_str.as_ptr(), ca_ptr) };

    if r != 0 {
        return Err(UmadError::GetCaError);
    }

    let ca:Box<umad_ca_t> = unsafe { Box::from_raw(ca_ptr) };


    Ok(UmadCa {
        umad_ca: *ca,
    })
}

pub fn umad_open_port(ca_name: &str, portnum: i32) -> Result<UmadPort, UmadError>{
    let ca_name_c_str: CString = CString::new(ca_name)?;

    let port_id = unsafe { umad::sys::umad_open_port(ca_name_c_str.as_ptr(), portnum.into())};

    if port_id < 0 {
        return Err(UmadError::OpenCaPortError);
    }

    let umad_port = UmadPort{
        hca: ca_name.to_owned(),
        port_id: port_id,
    };

    Ok(umad_port)
}

pub fn umad_close_port(umad_port: UmadPort) -> Result<bool, UmadError>{

    let result = unsafe { umad::sys::umad_close_port(umad_port.port_id)};

    if result < 0 {
        return Err(UmadError::CloseCaPortError);
    }

    Ok(true)
}
