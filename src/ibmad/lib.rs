use std::{ffi::{c_void, CStr, CString}, mem::MaybeUninit};

use thiserror::Error;


use crate::ibmad::{self};

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
    #[error("Unable to construct DR MAD path.")]
    DRMADPathError,
}

#[derive(Clone, Debug)]
pub struct IBMadPort {
    pub port: *mut ibmad::sys::ibmad_port,
}

unsafe impl Send for IBMadPort {}
unsafe impl Sync for IBMadPort {}

#[derive(Debug, Default)]
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
        let mut node_info = NodeInfo::default();

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
    let port: *mut  ibmad::sys::ibmad_port  = unsafe { ibmad::sys::mad_rpc_open_port(dev_name_ptr, 1, mgmt_classes.as_ptr() as *mut i32, num_classes) };

    Ok(IBMadPort{
        port
    })

}

pub fn mad_rpc_close_port(ibmad_port: &mut IBMadPort) -> Result<(), IBMadError> {

    unsafe { ibmad::sys::mad_rpc_close_port(ibmad_port.port) };
    Ok(())

}

pub fn send_dr_node_info_mad(port: &IBMadPort, path: &str, timeout: u32) -> Result<NodeInfo, IBSmpError> {

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

    let drpath:Box<ib_dr_path_t> = unsafe { Box::from_raw(drpath_ptr) };

    if r > 0 {
        portid.drpath = *drpath;        
    } else {
        return Err(IBSmpError::DRMADPathError);
    }
    
    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_INFO, 0, timeout, port.port) };

    let _portid = unsafe { Box::from_raw(portid_ptr) };

    if r.is_null() {
        return Err(IBSmpError::SendMADError);
    }

    let ni = NodeInfo::from_mad_fields(&mut data);

    Ok(ni)

}

pub fn send_lid_node_info_mad(port:&IBMadPort, lid: i32, timeout: u32) -> Result<NodeInfo, IBSmpError> {

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

    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  

    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_INFO, 0, timeout, port.port) };

    let _portid = unsafe { Box::from_raw(portid_ptr) };

    if r.is_null() {
        return Err(IBSmpError::SendMADError);
    }

    let ni = NodeInfo::from_mad_fields(&mut data);

    Ok(ni)

}

pub fn send_dr_node_desc_mad(port: &IBMadPort, path: &str, timeout: u32) -> Result<String, IBSmpError> {

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

    let drpath:Box<ib_dr_path_t> = unsafe { Box::from_raw(drpath_ptr) };

    if r > 0 {
        portid.drpath = *drpath;        
    } else {
        return Err(IBSmpError::DRMADPathError);
    }
    
    let mut data: [u8; IB_SMP_DATA_SIZE as usize] = [0; IB_SMP_DATA_SIZE as usize];
    let data_ptr = data.as_mut_ptr() as *mut c_void;  
    let portid_ptr = Box::into_raw(portid);

    let r: *mut u8 = unsafe { smp_query_via(data_ptr, portid_ptr, ibmad::sys::SMI_ATTR_ID_IB_ATTR_NODE_DESC  , 0, timeout, port.port) };

    let _portid = unsafe { Box::from_raw(portid_ptr) };

    if r.is_null() {
        return Err(IBSmpError::SendMADError);
    }

    let null_terminator_index = data
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(data.len());

    let node_desc_c_str = unsafe { CStr::from_bytes_with_nul_unchecked(&data[..null_terminator_index + 1]) };

    Ok(
        node_desc_c_str.to_string_lossy().to_string()
    )
}

pub fn perfquery(port:&IBMadPort, lid: i32, portnum: i32, pkey: u32, timeout: u32) -> Result<ibmad::perf::ExtPerfCounters, IBSmpError>{
    let portid = Box::new(ib_portid_t{
        lid: lid,
        drpath: unsafe { MaybeUninit::<ib_dr_path_t>::zeroed().assume_init() },
        grh_present: 0,
        gid: unsafe { MaybeUninit::<[u8; 16]>::zeroed().assume_init() },
        qp: 0,
        qkey: 0,
        sl: 0,
        pkey_idx: pkey,
    });
    let mut data: [u8; 1024 as usize] = [0; 1024];
    let portid_ptr = Box::into_raw(portid);
    let data_ptr = data.as_mut_ptr();
    let r: *mut u8 = unsafe { 
        pma_query_via(data_ptr as *mut c_void, portid_ptr, portnum, timeout, GSI_ATTR_ID_IB_GSI_PORT_COUNTERS_EXT, port.port)
    };
    let _portid = unsafe { Box::from_raw(portid_ptr) };
    if r.is_null() {
        return Err(IBSmpError::SendMADError);
    }
    let perf_counter = ibmad::perf::ExtPerfCounters::from_mad_fields(&mut data);
    Ok(perf_counter)
}

//smp_set_via
pub fn set_node_desc(port:&IBMadPort, lid: i32, timeout: u32) {
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

    let device_name_c_str = CString::new("switch-spine").unwrap();
    let portid_ptr: *mut portid = Box::into_raw(portid);

    //unsafe { smp_mkey_set(port.port, 0x0) };

    let r: *mut u8 = unsafe { 
        smp_set_via(device_name_c_str.as_bytes_with_nul().as_ptr() as *mut c_void, portid_ptr, SMI_ATTR_ID_IB_ATTR_NODE_DESC, 0, timeout , port.port)
    };

    let _portid = unsafe { Box::from_raw(portid_ptr) };
    if r.is_null() {
        println!("ERROR");
    }
}

