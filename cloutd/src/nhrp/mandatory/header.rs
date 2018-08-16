use super::*;
use {Parseable, Emitable, Result};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AddrTL {
    NSAP(u8),
    E164(u8),
}
impl From<u8> for AddrTL {
    fn from(value: u8) -> AddrTL {
        use self::AddrTL::*;
        if value & 64 == 64 {
            E164(value ^ 64) // Unset the 6th bit so the contained u8 is the length
        } else {
            NSAP(value)
        }
    }
}
impl From<AddrTL> for u8 {
    // FIXME: Technically only valid for values <64
    fn from(value: AddrTL) -> u8 {
        use self::AddrTL::*;
        match value {
            E164(v) => v | 64,
            NSAP(v) => v
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NhrpMandatory {
    ResolutionRequest(CommonHeader),
    ResolutionReply(CommonHeader),
    RegistrationRequest(CommonHeader),
    RegistrationReply(CommonHeader),
    PurgeRequest(CommonHeader),
    PurgeReply(CommonHeader),
    ErrorIndication(ErrorHeader),
    Other(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommonHeader {
    flags: u16,
    request_id: u32,
    src_nbma_addr: Vec<u8>,
    src_nbma_saddr: Vec<u8>,
    src_proto_addr: Vec<u8>,
    dst_proto_addr: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorHeader {
    shtl: AddrTL,
    sstl: AddrTL,

}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<CommonHeader> for MandatoryHeaderBuffer<&'a T> {
    fn parse(&self) -> Result<CommonHeader> {
        Ok(CommonHeader {
            flags: self.flags(),
            request_id: self.request_id(),
            src_nbma_addr: self.src_nbma_addr().to_vec(),
            src_nbma_saddr: self.src_nbma_saddr().to_vec(),
            src_proto_addr: self.src_proto_addr().to_vec(),
            dst_proto_addr: self.dst_proto_addr().to_vec(),
        })
    }
}

impl Emitable for CommonHeader {
    fn buffer_len(&self) -> usize {
        8 + self.src_nbma_addr.len()
          + self.src_nbma_saddr.len()
          + self.src_proto_addr.len()
          + self.dst_proto_addr.len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        use self::AddrTL::*;
        let mut buffer = MandatoryHeaderBuffer::new(buffer);
        buffer.set_src_nbma_addr_tl(NSAP(self.src_nbma_addr.len() as u8));
        buffer.set_src_nbma_saddr_tl(NSAP(self.src_nbma_saddr.len() as u8));
        buffer.set_src_proto_addr_len(self.src_proto_addr.len() as u8);
        buffer.set_dst_proto_addr_len(self.dst_proto_addr.len() as u8);
        buffer.set_flags(self.flags);
        buffer.set_request_id(self.request_id);
        buffer.src_nbma_addr_mut().copy_from_slice(&self.src_nbma_addr);
        buffer.src_nbma_saddr_mut().copy_from_slice(&self.src_nbma_saddr);
        buffer.src_proto_addr_mut().copy_from_slice(&self.src_proto_addr);
        buffer.dst_proto_addr_mut().copy_from_slice(&self.dst_proto_addr);
    }
}