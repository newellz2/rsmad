pub enum MadAttrId {
    ClassPortInfo = 0x1,
    Notice = 0x2,
    InformInfo = 0x3,
}

#[allow(non_camel_case_types)]
pub enum MadFields {
    IBNoField = 0,
    IBGIDPrefix_F = 1,

    //NodeInfo
	IBNodeBaseVer = 75,
	IBNodeClassVer_F = 76,
	IBNodeType_F = 77,
	IBNodeNPorts_F = 78,
	IBNodeSytemGuid_F = 79,
	IBNodeGuid_F = 80,
	IBNodePortGuid_F = 81,
	IBNodePartitionCap_F = 82,
	IBNodeDevid_F = 83,
	IBNodeRevision_F = 84,
	IBNodeLocalPort_F = 85,
	IBNodeVendorid_F = 86,

	//Extended Counter
	IBPcExtXmtBytes_F = 196,
	IBPcExtRcvBytes_F = 197,
	IBPcExtXmtPkts_F = 198,
	IBPcExtRcvPkts_F = 199,
	IBPcExtXmtUPkts_F = 200,
	IBPcExtRcvUPkts_F = 201,
	IBPcExtXmtMPkts_F = 202,
	IBPcExtRcvMPkts_F = 203,

	IBPcXmtDiscLast_F = 266,
	IBPcRcvLocalPhyErr_F = 267,
	IBPcRcvMalformedPktErr_F = 268,
	IBPcRcvBufOvrErr_F = 269,
	IBPcRcvDLIDMapErr_F = 270,
	IBPcRcvVLMapErr_F = 271,
	IBPcRcvLoopingErr_F = 272,

	IBPcExtErrSym_F = 644, //Symbol Error
	IBPcExtLinkRecovers_F = 645, //Recovers Error
	IBPcExtLinkDowned_F = 646, //Downed Error
	IBPcExtErrRcv_F = 647, //Receive Errors
	IBPcExtErrPhysRcv_F = 648, //Physical Receive Errors
	IBPcExtErrSwitchRel_F = 649, //Switch Relay Errors
	IBPcExtXmtDiscards_F = 650, //Discards
	IBPcExtErrXmtConstr_F = 651, //Xmit Constraint Errors
	IBPcExtErrRcvConstr_F = 652, //Rcv Constraint Errors
	IBPcExtErrLocalInteg_F = 653, //Local Integrity Errors
	IBPcExtErrExcessOvr_F = 654, //ExcessOverrun
	IBPcExtVL15Dropped = 655, //VL15Dropped
	IBPcExtXmitWait_F = 656, //XmitWaits
	IBPcExtQP1Drop_F = 657, //QP1 Drops
}

