include!(concat!(env!("OUT_DIR"), "/nhrp.rs"));

#[packet]
pub struct NHRPPacket {
    header: NHRPHeader,
    mandatorypart: NHRPMandatoryPart,
    cies: Vec<NHRPCIE>,
}

#[packet]
pub struct NHRPHeader {
    addressfamily: u16be,
    protocoltype: u16be,
    protocolsnap: u40be,
    hopcount: u8,
    packetsize: u16be,
    checksum: u16be,
    extensionoffset: u16be,
    opversion: u8,
    optype: u8,
}

#[packet]
pub struct NHRPMandatoryPart {
    /* shtl and sstl are moved to the mandatory part instead of the header
     * because a length_fn gets passed &self. This way we can use shtl and sstl
     * to determine the length of source_nbma_addr and source_nbma_subaddr.
     */
    shtl: u8,
    sstl: u8,
    source_proto_len: u8,
    dest_proto_len: u8,
    flags: u16be,
    request_id: u32be,

    #[length_fn = "shtl_to_len"]
    source_nbma_addr: Vec<u8>,

    #[length_fn = "sstl_to_len"]
    source_nbma_subaddr: Vec<u8>,

    #[length = "source_proto_len"]
    source_proto_addr: Vec<u8>,

    #[length = "dest_proto_len"]
    dest_proto_addr: Vec<u8>,
}

#[packet]
pub struct NHRPCIE {
    code: u8,
    prefix_len: u8,
    _: u16be,
    mtu: u16be,
    holding_time: u16be,
    client_addr_typelen: u8,
    client_subaddr_typelen: u8,
    client_proto_len: u8,
    preference: u8,

    #[length_fn = "client_addr_typelen_to_len"]
    client_nbma_addr: Vec<u8>,

    #[length_fn = "client_subaddr_typelen_to_len"]
    client_nbma_subaddr: Vec<u8>,

    #[length = "client_proto_len"]
    client_proto_addr: Vec<u8>,
}

fn shtl_to_len(pkt: &NHRPMandatoryPart) -> usize {
}
