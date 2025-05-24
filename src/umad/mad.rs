#[allow(non_camel_case_types)]
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct ib_mad {
	pub base_version: u8,
	pub mgmt_class: u8,
	pub class_version: u8,
	pub method: u8,
	pub status: u16,
	pub hop_ptr: u8,
	pub hop_cnt: u8,
	pub tid: u64,
	pub attr_id: u16,
	pub resv: u16,
	pub attr_mod: u32,
	pub reserved: u64,
	pub reserved2: [u8; 32],
	pub data:  [u8; 256]
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ib_mad_addr {
    pub qpn: u32,
    pub qkey: u32,
    pub lid: u16,
    pub sl: u8,
    pub path_bits: u8,
    pub grh_present: u8,
    pub gid_index: u8,
    pub hop_limit: u8,
    pub traffic_class: u8,
    pub gid: [u8; 16],
    pub flow_label: u32,
    pub pkey_index: u16,
    pub reserved: [u8; 6],
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ib_user_mad {
    pub agent_id: u32,
    pub status: u32,
    pub timeout_ms: u32,
    pub retries: u32,
    pub length: u32,
    pub addr: ib_mad_addr,
    pub data: [u8; 320],
}

impl ib_user_mad {
    pub fn new() -> ib_user_mad {
        ib_user_mad {
            agent_id: 0,
            status: 0,
            timeout_ms: 0,
            retries: 0,
            length: 0,
            addr: ib_mad_addr{
                qpn: 0,
                qkey: 0,
                lid: 0,
                sl: 0,
                path_bits: 0,
                grh_present: 0,
                gid_index: 0,
                hop_limit: 0,
                traffic_class: 0,
                gid: unsafe { std::mem::zeroed() },
                flow_label: 0,
                pkey_index: 0,
                reserved: unsafe { std::mem::zeroed()  },
            },
            data: unsafe { std::mem::zeroed() },
        }
    }

    pub fn as_c_void_ptr(&self) -> *mut std::ffi::c_void {
        self as *const ib_user_mad as *mut std::ffi::c_void
    }
}