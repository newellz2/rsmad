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
}

