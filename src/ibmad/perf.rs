use std::ffi::c_void;
use super::enums::MadFields;
use crate::ibmad;

const PERF_COUNTERS_FIELDS: [i32; 21] = 
[
    MadFields::IBPcExtXmtBytes_F as i32,
    MadFields::IBPcExtRcvBytes_F as i32,
    MadFields::IBPcExtXmtPkts_F as i32,
    MadFields::IBPcExtRcvPkts_F as i32,
    MadFields::IBPcExtXmtUPkts_F as i32,
    MadFields::IBPcExtRcvUPkts_F as i32,
    MadFields::IBPcExtXmtMPkts_F as i32,
    MadFields::IBPcExtRcvMPkts_F as i32,
    MadFields::IBPcExtErrSym_F as i32,
    MadFields::IBPcExtLinkRecovers_F as i32,
    MadFields::IBPcExtLinkDowned_F as i32,
    MadFields::IBPcExtErrRcv_F as i32,
    MadFields::IBPcExtErrPhysRcv_F as i32,
    MadFields::IBPcExtErrSwitchRel_F as i32,
    MadFields::IBPcExtXmtDiscards_F as i32,
    MadFields::IBPcExtErrXmtConstr_F as i32,
    MadFields::IBPcExtErrRcvConstr_F as i32,
	MadFields::IBPcExtErrExcessOvr_F as i32,
	MadFields::IBPcExtVL15Dropped as i32,
	MadFields::IBPcExtXmitWait_F as i32,
	MadFields::IBPcExtQP1Drop_F as i32,
];

#[derive(Debug, Default)]
pub struct ExtPerfCounters {
    xmt_bytes: u64,
    rcv_bytes: u64,
    xmt_pkts: u64,
    rcv_pkts: u64,
    xmt_upkts: u64,
    rcv_upkts: u64,
    xmt_mpkts: u64,
    rcv_mpkts: u64,
    symbol_errors: u64,
    link_recovers: u64,
    link_downed: u64,
    rcv_errors: u64,
    phys_rcv_errors: u64,
    switch_relay_errors: u64,
    xmt_discards: u64,
    xmt_constraint_errors: u64,
    rcv_contrainst_errors: u64,
    excess_overruns_errors: u64,
    vl15dropped: u64,
    xmit_wait: u64,
    qp1_drop: u64,

}

impl ExtPerfCounters {
    pub fn from_mad_fields(data: &mut [u8]) -> Self {
        let mut ext_perf = ExtPerfCounters::default();

        for (index, &field) in PERF_COUNTERS_FIELDS.iter().enumerate() {
            let mut val: [u8; 8] = [0; 8];
            let val_ptr = val.as_mut_ptr() as *mut c_void;

            unsafe {
                ibmad::sys::mad_decode_field(data.as_mut_ptr(), field.try_into().unwrap(), val_ptr);
            }

            let result = u64::from_le_bytes(val);
            *ext_perf.field_by_index(index) = result;
        }

        ext_perf
    }

    fn field_by_index(&mut self, index: usize) -> &mut u64 {
        match index {
            0 => &mut self.xmt_bytes,
            1 => &mut self.rcv_bytes,
            2 => &mut self.xmt_pkts,
            3 => &mut self.rcv_pkts,
            4 => &mut self.xmt_upkts,
            5 => &mut self.rcv_upkts,
            6 => &mut self.xmt_mpkts,
            7 => &mut self.rcv_mpkts,
            8 => &mut self.symbol_errors,
            9 => &mut self.link_recovers,
            10 => &mut self.link_downed,
            11 => &mut self.rcv_errors,
            12 => &mut self.phys_rcv_errors,
            13 => &mut self.switch_relay_errors,
            14 => &mut self.xmt_discards,
            15 => &mut self.xmt_constraint_errors,
            16 => &mut self.rcv_contrainst_errors,
            17 => &mut self.excess_overruns_errors,
            18 => &mut self.vl15dropped,
            19 => &mut self.xmit_wait,
            20 => &mut self.qp1_drop,
            _ => panic!("Invalid field index for ExtPerfCounters"),
        }
    }
}
