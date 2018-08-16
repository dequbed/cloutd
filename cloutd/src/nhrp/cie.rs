use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientInformationEntry {
    pub code: u8,
    pub prefixlength: u8,
    pub mtu: u16,
    pub holdingtime: u16,
    pub client_addr_tl: u8,
    pub client_saddr_tl: u8,
    pub client_proto_len: u8,
    pub preference: u8,
    pub client_nbma_addr: Vec<u8>,
    pub client_nbma_saddr: Vec<u8>,
    pub client_proto_addr: Vec<u8>,
}
