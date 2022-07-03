use crate::{Parseable, Emitable, Result};
use super::*;
use super::cie::buffer::CieIterator;
use super::cie::message::ClientInformationEntry;

use std::net::IpAddr;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ResolutionCode {
    Success,
    Prohibited,
    InsufficientResources,
    NoBindingExists,
    BindingNotUnique,
    Unknown(u8)
}
impl From<u8> for ResolutionCode {
    fn from(value: u8) -> ResolutionCode {
        use ResolutionCode::*;
        match value {
            0 => Success,
            4 => Prohibited,
            5 => InsufficientResources,
            12 => NoBindingExists,
            13 => BindingNotUnique,
            _ => Unknown(value)
        }
    }
}
impl From<ResolutionCode> for u8 {
    fn from(value: ResolutionCode) -> u8 {
        use ResolutionCode::*;
        match value {
            Success => 0,
            Prohibited => 4,
            InsufficientResources => 5,
            NoBindingExists => 12,
            BindingNotUnique => 13,
            Unknown(v) => v
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolutionReplyMessage {
    header: CommonHeader,
    cie: Vec<ClientInformationEntry>,
}

impl ResolutionReplyMessage {
    pub fn new(request_id: u32,
               code: ResolutionCode,
               src_n_a: IpAddr,
               src_p_a: IpAddr,
               dst_n_a: Option<IpAddr>,
               dst_p_a: IpAddr,
               requester_router: bool,
               authorative: bool,
               unique: bool,
               src_stable: bool,
               dst_stable: bool,
               holding_time: u16,
               prefix_len: u8
    ) -> Self {
        let header = CommonHeader {
            flags: ((requester_router as u16)<<15) | ((authorative as u16)<<14) | ((dst_stable as u16)<<13) | ((unique as u16)<<12) | ((src_stable as u16)<<11),
            request_id: request_id,
            src_nbma_addr: src_n_a,
            src_proto_addr: src_p_a,
            dst_proto_addr: dst_p_a,
        };

        let cie = match dst_n_a {
            Some(dst) => ClientInformationEntry {
                code: code.into(),
                client_nbma_addr: Some(dst),
                client_proto_addr: Some(dst_p_a),
                holding_time: holding_time,
                mtu: 0,
                preference: 0,
                prefix_len: prefix_len
            },
            None => ClientInformationEntry {
                code: code.into(),
                holding_time: 0,
                mtu: 0,
                preference: 0,
                prefix_len: 0,
                client_nbma_addr: None,
                client_proto_addr: None
            }
        };

        ResolutionReplyMessage {
            header: header, cie: vec![cie],
        }
    }

    #[allow(dead_code)]
    pub fn into_parts(self) -> (CommonHeader, Vec<ClientInformationEntry>) {
        (self.header, self.cie)
    }
}
               

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<ResolutionReplyMessage> for OperationBuffer<&'a T> {
    fn parse(&self) -> Result<ResolutionReplyMessage> {
        let header = <Self as Parseable<CommonHeader>>::parse(self)?;
        let cies = CieIterator::new(self.payload());
        let mut ciev = Vec::new();
        for cie in cies {
            match cie {
                Ok(cie) => ciev.push(cie.parse()?),
                Err(e) => return Err(e),
            }
        }

        Ok(ResolutionReplyMessage {
            header: header,
            cie: ciev,
        })
    }
}

impl Emitable for ResolutionReplyMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len() + self.cie.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        let buffer = &mut buffer[self.header.buffer_len()..];
        self.cie.emit(buffer);
    }
}
