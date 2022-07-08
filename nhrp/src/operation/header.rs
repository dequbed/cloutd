use super::*;
use crate::{Parseable, Emitable, Result, Error};

use std::net::IpAddr::{self, *};
use std::net::Ipv4Addr;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AddrTL {
    NSAP(u8),
    E164(u8),
}
impl From<u8> for AddrTL {
    fn from(value: u8) -> AddrTL {
        assert!(value < 64);
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
            E164(v) => (v & 0b00111111) | 64,
            NSAP(v) => (v & 0b00111111),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommonHeader {
    pub flags: u16,
    pub request_id: u32,
    pub src_nbma_addr: IpAddr,
    pub src_proto_addr: IpAddr,
    pub dst_proto_addr: IpAddr,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorHeader {
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<CommonHeader> for OperationBuffer<&'a T> {
    fn parse(&self) -> Result<CommonHeader> {
        Ok(CommonHeader {
            flags: self.flags(),
            request_id: self.request_id(),
            src_nbma_addr: parse_ip(self.src_nbma_addr())?,
            src_proto_addr: parse_ip(self.src_proto_addr())?,
            dst_proto_addr: parse_ip(self.dst_proto_addr())?,
        })
    }
}

fn parse_ip(a: &[u8]) -> Result<IpAddr> {
        match a.len() {
            4 => {
                let addr = Ipv4Addr::new(a[0], a[1], a[2], a[3]);
                Ok(IpAddr::V4(addr))
            },
            16 => {
                let mut addr: [u16; 8] = [0;8];
                for i in 0..8 {
                    let start = i*2;
                    let end = start + 2;
                    addr[i] = u16::from_be_bytes(a[start..end].try_into().unwrap())
                }
                Ok(IpAddr::V6(addr.into()))
            },
            _ => Err(Error::NotImplemented),
        }
}

fn iplen(i: &IpAddr) -> usize {
    match *i {
        V4(_) => 4,
        V6(_) => 16
    }
}
fn write_ip(buf: &mut [u8], addr: IpAddr) {
    match addr {
        V4(a) => {
            let a = a.octets();
            buf.copy_from_slice(&a);
        }
        V6(a) => {
            let a = a.octets();
            buf.copy_from_slice(&a);
        }
    };
}

impl Emitable for CommonHeader {
    fn buffer_len(&self) -> usize {
        10 + iplen(&self.src_nbma_addr)
           + iplen(&self.src_proto_addr)
           + iplen(&self.dst_proto_addr)
    }

    fn emit(&self, buffer: &mut [u8]) {
        use self::AddrTL::*;
        let mut buffer = OperationBuffer::new(buffer);
        buffer.set_src_nbma_addr_tl(NSAP(iplen(&self.src_nbma_addr) as u8));
        buffer.set_src_nbma_saddr_tl(NSAP(0));
        buffer.set_src_proto_addr_len(iplen(&self.src_proto_addr) as u8);
        buffer.set_dst_proto_addr_len(iplen(&self.dst_proto_addr) as u8);
        buffer.set_flags(self.flags);
        buffer.set_request_id(self.request_id);
        write_ip(buffer.src_nbma_addr_mut(), self.src_nbma_addr);
        write_ip(buffer.src_proto_addr_mut(), self.src_proto_addr);
        write_ip(buffer.dst_proto_addr_mut(), self.dst_proto_addr);
    }
}
