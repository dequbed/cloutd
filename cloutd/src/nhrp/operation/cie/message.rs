use nhrp::cie::buffer::CieBuffer;

use {Parseable, Emitable, Result};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientInformationEntry {
    pub code: u8,
    pub prefix_len: u8,
    pub mtu: u16,
    pub holding_time: u16,
    pub preference: u8,
    pub client_nbma_addr: Vec<u8>,
    pub client_nbma_saddr: Vec<u8>,
    pub client_proto_addr: Vec<u8>,
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<ClientInformationEntry> for CieBuffer<&'a T> {
    fn parse(&self) -> Result<ClientInformationEntry> {
        Ok(ClientInformationEntry {
            code: self.code(),
            prefix_len: self.prefix_len(),
            mtu: self.mtu(),
            holding_time: self.holding_time(),
            preference: self.preference(),
            client_nbma_addr: self.cli_nbma_addr().to_vec(),
            client_nbma_saddr: self.cli_nbma_saddr().to_vec(),
            client_proto_addr: self.cli_proto_addr().to_vec(),
        })
    }
}

impl Emitable for ClientInformationEntry {
    fn buffer_len(&self) -> usize {
        12 + self.client_nbma_addr.len() + self.client_nbma_saddr.len() + self.client_proto_addr.len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        let mut buffer = CieBuffer::new(buffer);
        buffer.set_code(self.code);
        buffer.set_prefix_len(self.prefix_len);
        buffer.set_mtu(self.mtu);
        buffer.set_holding_time(self.holding_time);
        buffer.set_preference(self.preference);
        buffer.set_cli_nbma_addr_tl(self.client_nbma_addr.len() as u8);
        buffer.set_cli_nbma_saddr_tl(self.client_nbma_saddr.len() as u8);
        buffer.set_cli_proto_addr_len(self.client_proto_addr.len() as u8);
        buffer.cli_nbma_addr_mut().copy_from_slice(&self.client_nbma_addr);
        buffer.cli_nbma_saddr_mut().copy_from_slice(&self.client_nbma_saddr);
        buffer.cli_proto_addr_mut().copy_from_slice(&self.client_proto_addr);
    }
}
