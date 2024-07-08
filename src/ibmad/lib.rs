
use std::{ffi::{c_void, CString, CStr}, mem::MaybeUninit};
use thiserror::Error;

use crate::{ibmad, umad};

use ibmad::sys::*;

#[derive(Error, Debug)]
pub enum IBMadError {
    #[error("Unable to open port.")]
    OpenPortError,
}

pub struct IBMadPort {
    port: *mut ibmad::sys::ibmad_port,
}

pub fn mad_rpc_open_port(device_name: &str) -> Result<IBMadPort, IBMadError> {

    let device_name_c_str = CString::new(device_name).unwrap();

    let mgmt_classes =[ MAD_CLASSES_IB_SMI_DIRECT_CLASS, MAD_CLASSES_IB_SMI_CLASS, MAD_CLASSES_IB_SA_CLASS, MAD_CLASSES_IB_PERFORMANCE_CLASS ];
    let num_classes = mgmt_classes.len() as i32;

    let dev_name_ptr: *mut i8 = device_name_c_str.as_ptr() as *mut i8;
    
    let port:*mut  ibmad::sys::ibmad_port  = unsafe { ibmad::sys::mad_rpc_open_port(dev_name_ptr, 1, mgmt_classes.as_ptr() as *mut i32, num_classes) };

    Ok(IBMadPort{
        port
    })

}

pub fn send_dr_nodeinfo_mad(port: IBMadPort, path: &str, timeout: u32) {

    let drpath = Box::new(ib_dr_path_t{
        cnt: 0,
        p: unsafe { MaybeUninit::<[u8; 64]>::zeroed().assume_init() },
        drslid: 0xffff,
        drdlid: 0xffff,
    });

    let mut portid = Box::new(ib_portid_t{
        lid: 0,
        drpath: *drpath,
        grh_present: 0,
        gid: unsafe { MaybeUninit::<[u8; 16]>::zeroed().assume_init() },
        qp: 100,
        qkey: 0,
        sl: 100,
        pkey_idx: 0,
    });

    let drpath_ptr = Box::into_raw(drpath);

    let routepath =  CString::new(path).unwrap();

    let r = unsafe { str2drpath(drpath_ptr, routepath.as_ptr() as *mut i8, 0xffff, 0xffff) };

    if r > 0 {
        let drpath:Box<ib_dr_path_t> = unsafe { Box::from_raw(drpath_ptr) };
        portid.drpath = *drpath;
        println!("str2drpath success: r={} cnt={} path={:?}", r, drpath.cnt, drpath.p);
    }
    
    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_INFO, 0, timeout, port.port) };

    let portid: Box<ib_portid_t> = unsafe { Box::from_raw(portid_ptr) };

    //NodeType
    let mut node_type =  Box::new(0 as i32);
    let node_type_ptr = Box::into_raw(node_type);

    unsafe { ibmad::sys::mad_decode_field( data.as_mut_ptr() , ibmad::sys::MAD_FIELDS_IB_NODE_TYPE_F, node_type_ptr as *mut c_void) };

    node_type = unsafe { Box::from_raw(node_type_ptr) };

    //NodeGUID
    let mut node_guid = 0u64;

    unsafe { ibmad::sys::mad_decode_field( data.as_mut_ptr() , ibmad::sys::MAD_FIELDS_IB_NODE_GUID_F, &mut node_guid as *mut u64 as *mut c_void) };

    println!("smp_query_via: r={:?} buf={:?} qp={} sl={} node_type={} node_guid={}", r, data, portid.qp, portid.sl, node_type, umad::format_u64_big_endian(node_guid));

}


pub fn send_dr_node_desc_mad(port: IBMadPort, path: &str, timeout: u32) {

    let drpath = Box::new(ib_dr_path_t{
        cnt: 0,
        p: unsafe { MaybeUninit::<[u8; 64]>::zeroed().assume_init() },
        drslid: 0xffff,
        drdlid: 0xffff,
    });

    let mut portid = Box::new(ib_portid_t{
        lid: 0,
        drpath: *drpath,
        grh_present: 0,
        gid: unsafe { MaybeUninit::<[u8; 16]>::zeroed().assume_init() },
        qp: 100,
        qkey: 0,
        sl: 100,
        pkey_idx: 0,
    });

    let drpath_ptr = Box::into_raw(drpath);

    let routepath =  CString::new(path).unwrap();

    let r = unsafe { str2drpath(drpath_ptr, routepath.as_ptr() as *mut i8, 0xffff, 0xffff) };

    if r > 0 {
        let drpath:Box<ib_dr_path_t> = unsafe { Box::from_raw(drpath_ptr) };
        portid.drpath = *drpath;
        println!("str2drpath success: r={} cnt={} path={:?}", r, drpath.cnt, drpath.p);
    }
    
    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_DESC  , 0, timeout, port.port) };

    let portid: Box<ib_portid_t> = unsafe { Box::from_raw(portid_ptr) };

    //NodeDesc
    let mut node_desc_bytes = [0u8; 64+1];
    let node_desc_ptr = data.as_mut_ptr() as *mut c_void;  

    //unsafe { ibmad::sys::mad_decode_field( data.as_mut_ptr() , 0x11  , node_desc_ptr) };
    let null_terminator_index = data
    .iter()
    .position(|&byte| byte == 0)
    .unwrap_or(data.len()); // Default to the end if no null found

    // Convert to CStr (only up to the null terminator)
    let node_desc_c_str = unsafe { CStr::from_bytes_with_nul_unchecked(&data[..null_terminator_index + 1]) };


    // Convert to CStr (only up to the null terminator)
    let node_desc_data_c_str = unsafe { CStr::from_bytes_with_nul_unchecked(&data[0..64]) };
    let node_desc_bytes_c_str = unsafe { CStr::from_bytes_with_nul_unchecked(&node_desc_bytes[0..64]) };


    // Convert to Rust String and handle potential UTF-8 issues (although unlikely for ASCII)
    let node_desc_string = node_desc_c_str.to_string_lossy();


    println!("smp_query_via: r={:?} buf={:?} qp={} sl={} node_desc={:?}", r, data, portid.qp, portid.sl, node_desc_string);

}


pub fn perf_query(port: IBMadPort, lid: i32, portnum: i32, timeout: u32) {

    let mut portid = Box::new(ib_portid_t{
        lid: lid,
        drpath: unsafe { MaybeUninit::<ib_dr_path_t>::zeroed().assume_init() },
        grh_present: 0,
        gid: unsafe { MaybeUninit::<[u8; 16]>::zeroed().assume_init() },
        qp: 0,
        qkey: 0,
        sl: 0,
        pkey_idx: 0,
    });

    let portid_ptr = Box::into_raw(portid);
    
    let mut data: [u8; 1024 as usize] = [0; 1024];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let r: *mut u8 = unsafe { 
        pma_query_via(data_ptr as *mut c_void, portid_ptr, portnum, timeout, GSI_ATTR_ID_IB_GSI_PORT_COUNTERS_EXT, port.port)
    };

    //LinkDowned
    let mut link_down_ctr =  Box::new(0 as i32);
    let link_down_ctr_ptr = Box::into_raw(link_down_ctr);
    unsafe { ibmad::sys::mad_decode_field( data.as_mut_ptr() , ibmad::sys::MAD_FIELDS_IB_PC_EXT_LINK_DOWNED_F, link_down_ctr_ptr as *mut c_void) };
    link_down_ctr = unsafe { Box::from_raw(link_down_ctr_ptr) };

    //XmitBytes
    let mut xmit_bytes_ctr =  Box::new(0 as i64);
    let xmit_bytes_ctr_ptr = Box::into_raw(xmit_bytes_ctr);
    unsafe { ibmad::sys::mad_decode_field( data.as_mut_ptr() , ibmad::sys::MAD_FIELDS_IB_PC_EXT_XMT_BYTES_F, xmit_bytes_ctr_ptr as *mut c_void) };
    xmit_bytes_ctr = unsafe { Box::from_raw(xmit_bytes_ctr_ptr) };

    println!("perfquery: {:?} {:?} {:?} {:?}", data, &r, link_down_ctr, xmit_bytes_ctr);

}
