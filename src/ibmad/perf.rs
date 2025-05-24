use std::ops::Sub;
use super::enums::MadFields;
use crate::ibmad;
use std::{collections::HashMap, ffi::c_void};

const PERF_COUNTERS_FIELDS: [(i32, &str); 28] = [
    (MadFields::IBPcExtXmtBytes_F as i32, "xmt_bytes"),
    (MadFields::IBPcExtRcvBytes_F as i32, "rcv_bytes"),
    (MadFields::IBPcExtXmtPkts_F as i32,  "xmt_pkts"),
    (MadFields::IBPcExtRcvPkts_F as i32,  "rcv_pkts"),
    (MadFields::IBPcExtXmtUPkts_F as i32, "xmt_upkts"),
    (MadFields::IBPcExtRcvUPkts_F as i32, "rcv_upkts"),
    (MadFields::IBPcExtXmtMPkts_F as i32, "xmt_mpkts"),
    (MadFields::IBPcExtRcvMPkts_F as i32, "rcv_mpkts"),
    (MadFields::IBPcExtErrSym_F as i32,   "symbol_errors"),
    (MadFields::IBPcExtLinkRecovers_F as i32, "link_recovers"),
    (MadFields::IBPcExtLinkDowned_F as i32, "link_downed"),
    (MadFields::IBPcExtErrRcv_F as i32, "rcv_errors"),
    (MadFields::IBPcExtErrPhysRcv_F as i32, "phys_rcv_errors"),
    (MadFields::IBPcExtErrSwitchRel_F as i32, "switch_rel_errors"),
    (MadFields::IBPcExtXmtDiscards_F as i32, "xmt_discards"),
    (MadFields::IBPcXmtDiscLast_F as i32, "xmt_discard_last"),
    (MadFields::IBPcRcvLocalPhyErr_F as i32,"rcv_local_phy_errors"),
    (MadFields::IBPcRcvMalformedPktErr_F as i32,"rcv_malformed_pkt_errors"),
    (MadFields::IBPcRcvBufOvrErr_F as i32,"rcv_buffer_overrun_errors"),
    (MadFields::IBPcRcvDLIDMapErr_F as i32, "rcv_dlid_map_errors"),
    (MadFields::IBPcRcvVLMapErr_F as i32, "rcv_vl_map_errors"),
    (MadFields::IBPcRcvLoopingErr_F as i32, "rcv_looping_errors"),
    (MadFields::IBPcExtErrXmtConstr_F as i32,"xmt_constraint_errors"),
    (MadFields::IBPcExtErrRcvConstr_F as i32,"rcv_constraint_errors"),
    (MadFields::IBPcExtErrExcessOvr_F as i32,"excess_overrun_errors"),
    (MadFields::IBPcExtVL15Dropped as i32, "vl15dropped"),
    (MadFields::IBPcExtXmitWait_F as i32, "xmit_waits"),
    (MadFields::IBPcExtQP1Drop_F as i32, "qp1_drops"),
];

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ExtPerfCounters {
    pub counters: HashMap<String, u64>,
}

impl ExtPerfCounters {
    pub fn from_mad_fields(data: &mut [u8]) -> Self {
        let mut ext_perf = ExtPerfCounters::default();

        for field in PERF_COUNTERS_FIELDS.iter() {
            let mut val: [u8; 8] = [0; 8];
            let val_ptr = val.as_mut_ptr();

            unsafe {
                ibmad::sys::mad_decode_field(
                    data.as_mut_ptr(),
                    field.0.try_into().unwrap(),
                    val_ptr as *mut c_void,
                );
            }

            let result = u64::from_le_bytes(val);
            ext_perf.counters.insert(field.1.to_string(), result);
        }

        ext_perf
    }

    pub fn delta(&self, other: &ExtPerfCounters) -> HashMap<String, u64> {
        let mut delta_map = HashMap::new();

        for (name, &value) in &self.counters {
            let delta = if let Some(&other_value) = other.counters.get(name) {
                value.saturating_sub(other_value) // Use saturating_sub to prevent underflow
            } else {
                value // If the counter isn't in 'other', the delta is the full value
            };

            // Include only non-zero deltas
            if delta != 0 {
                delta_map.insert(name.clone(), delta);
            }
        }

        delta_map // Return the HashMap with deltas
    }

    #[allow(dead_code)]
    fn display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, value) in self.delta(&ExtPerfCounters::default()) {
            writeln!(f, "{}: {}", name, value)?;
        }
        Ok(())
    }
}

impl Sub for ExtPerfCounters {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let mut output = ExtPerfCounters {
            counters: HashMap::new(),
        };

        // Iterate over self.counters, checking for corresponding values in other.counters
        for (name, &value) in &self.counters {
            let delta = if let Some(&other_value) = other.counters.get(name) {
                value.saturating_sub(other_value)
            } else {
                value 
            };

            output.counters.insert(name.clone(), delta);
            
        }

        output 
    }
}