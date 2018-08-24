use {Parseable, Emitable, Result};
use super::*;
use super::cie::buffer::CieBuffer;
use super::cie::message::ClientInformationEntry;

use std::net::IpAddr;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RegistrationCode {
    Success,
    Prohibited,
    InsufficientResources,
    AlreadyRegistered,
    Unknown(u8)
}
impl From<u8> for RegistrationCode {
    fn from(value: u8) -> RegistrationCode {
        use RegistrationCode::*;
        match value {
            0 => Success,
            4 => Prohibited,
            5 => InsufficientResources,
            14 => AlreadyRegistered,
            _ => Unknown(value),
        }
    }
}
impl From<RegistrationCode> for u8 {
    fn from(value: RegistrationCode) -> u8 {
        use RegistrationCode::*;
        match value {
            Success => 0,
            Prohibited => 4,
            InsufficientResources => 5,
            AlreadyRegistered => 14,
            Unknown(v) => v
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegistrationReplyMessage {
    header: CommonHeader,
    cie: ClientInformationEntry,
}

impl RegistrationReplyMessage {
    pub fn new(request_id: u32,
               code: RegistrationCode,
               mut cie: ClientInformationEntry,
               src_nbma_addr: IpAddr,
               src_proto_addr: IpAddr,
               dst_proto_addr: IpAddr,
               unique: bool
    ) -> Self {
        let header = CommonHeader {
            flags: if unique { 0x8000 } else { 0 },
            request_id: request_id,
            src_nbma_addr: src_nbma_addr,
            src_proto_addr: src_proto_addr,
            dst_proto_addr: dst_proto_addr,
        };

        cie.code = code.into();

        RegistrationReplyMessage {
            header: header, cie: cie,
        }
    }

    pub fn into_parts(self) -> (CommonHeader, ClientInformationEntry) {
        (self.header, self.cie)
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<RegistrationReplyMessage> for OperationBuffer<&'a T> {
    fn parse(&self) -> Result<RegistrationReplyMessage> {
        let header = <Self as Parseable<CommonHeader>>::parse(self)?;
        let cie = CieBuffer::new_checked(self.payload())?.parse()?;

        Ok(RegistrationReplyMessage {
            header: header,
            cie: cie,
        })
    }
}

impl Emitable for RegistrationReplyMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len() + self.cie.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        let buffer = &mut buffer[self.header.buffer_len()..];
        self.cie.emit(buffer);
    }
}
