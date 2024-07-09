use std::{ffi::{c_void, CString, CStr}, mem::MaybeUninit};
use thiserror::Error;

use crate::ibmad;

use ibmad::sys::*;
use ibmad::enums::*;

#[derive(Error, Debug)]
pub enum IBMadError {
    #[error("Unable to open port.")]
    OpenPortError,
}

#[derive(Error, Debug)]
pub enum IBSmpError {
    #[error("Unable to send MAD.")]
    SendMADError,
}


pub struct IBMadPort {
    port: *mut ibmad::sys::ibmad_port,
}

#[derive(Debug)]
pub struct NodeInfo {
    pub base_ver: u64,
    pub class_vers: u64,
    pub node_type: u64,
    pub num_ports: u64,
    pub system_guid: u64,
    pub guid: u64,
    pub port_guid: u64,
    pub part_cap: u64,
    pub dev_id: u64,
    pub revision: u64,
    pub local_port: u64,
    pub vendor_id: u64,
}

impl NodeInfo {
    pub fn from_mad_fields(data: &mut [u8]) -> Self {
        let mut node_info = NodeInfo {
            base_ver: 0,
            class_vers: 0,
            node_type: 0,
            num_ports: 0,
            system_guid: 0,
            guid: 0,
            port_guid: 0,
            part_cap: 0,
            dev_id: 0,
            revision: 0,
            local_port: 0,
            vendor_id: 0,
        };

        for i in (MadFields::IBNodeBaseVer as i32)..=(MadFields::IBNodeVendorid_F as i32) {
            let mut val: [u8; 8] = [0; 8];
            let val_ptr = val.as_mut_ptr() as *mut c_void;

            unsafe {
                ibmad::sys::mad_decode_field(data.as_mut_ptr(), i.try_into().unwrap(), val_ptr as *mut c_void);
            }

            let result = u64::from_le_bytes(val); //Little Endian
     
            if i == MadFields::IBNodeBaseVer as i32 {
                node_info.base_ver = result;
            }

            if i == MadFields::IBNodeClassVer_F as i32 {
                node_info.class_vers = result;
            }

            if i == MadFields::IBNodeType_F as i32 {
                node_info.node_type = result;
            }

            if i == MadFields::IBNodeNPorts_F as i32 {
                node_info.num_ports = result;
            }

            if i == MadFields::IBNodeSytemGuid_F as i32 {
                node_info.system_guid = result;
            }

            if i == MadFields::IBNodeGuid_F as i32 {
                node_info.guid = result;
            }

            if i == MadFields::IBNodePortGuid_F as i32 {
                node_info.port_guid = result;
            }

            if i == MadFields::IBNodePartitionCap_F as i32 {
                node_info.part_cap = result;
            }

            if i == MadFields::IBNodeDevid_F as i32 {
                node_info.dev_id = result;
            }

            if i == MadFields::IBNodeRevision_F as i32 {
                node_info.revision = result;
            }

            if i == MadFields::IBNodeLocalPort_F as i32 {
                node_info.local_port = result;
            }

            if i == MadFields::IBNodeVendorid_F as i32 {
                node_info.vendor_id = result;
            }
        }

        node_info
    }
}


pub fn mad_rpc_open_port(device_name: &str, mgmt_classes: &[u32]) -> Result<IBMadPort, IBMadError> {

    let device_name_c_str = CString::new(device_name).unwrap();
    let num_classes = mgmt_classes.len() as i32;
    let dev_name_ptr: *mut i8 = device_name_c_str.as_ptr() as *mut i8;    
    let port:*mut  ibmad::sys::ibmad_port  = unsafe { ibmad::sys::mad_rpc_open_port(dev_name_ptr, 1, mgmt_classes.as_ptr() as *mut i32, num_classes) };

    Ok(IBMadPort{
        port
    })

}

pub fn send_dr_node_info_mad(port: IBMadPort, path: &str, timeout: u32) -> Result<NodeInfo, IBSmpError> {

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
        qp: 0,
        qkey: 0,
        sl: 0,
        pkey_idx: 0,
    });

    let drpath_ptr = Box::into_raw(drpath);

    let routepath =  CString::new(path).unwrap();

    let r = unsafe { str2drpath(drpath_ptr, routepath.as_ptr() as *mut i8, 0xffff, 0xffff) };

    if r > 0 {
        let drpath:Box<ib_dr_path_t> = unsafe { Box::from_raw(drpath_ptr) };
        portid.drpath = *drpath;
    }
    
    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_INFO, 0, timeout, port.port) };

    if r.is_null() {
        return Err(IBSmpError::SendMADError);
    }

    let ni = NodeInfo::from_mad_fields(&mut data);

    Ok(ni)

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
    }
    
    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_DESC  , 0, timeout, port.port) };

    let portid: Box<ib_portid_t> = unsafe { Box::from_raw(portid_ptr) };

    //NodeDesc
    //let mut node_desc_bytes = [0u8; 64+1];
    //let node_desc_ptr = data.as_mut_ptr() as *mut c_void;  

    let null_terminator_index = data
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(data.len());

    let node_desc_c_str = unsafe { CStr::from_bytes_with_nul_unchecked(&data[..null_terminator_index + 1]) };

    // Convert to Rust String and handle potential UTF-8 issues (although unlikely for ASCII)
    let node_desc_string = node_desc_c_str.to_string_lossy();

    println!("smp_query_via: r={:?} buf={:?} qp={} sl={} node_desc={:?}", r, data, portid.qp, portid.sl, node_desc_string);

}


pub fn perf_query(port: IBMadPort, lid: i32, portnum: i32, timeout: u32) {

    let portid = Box::new(ib_portid_t{
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
