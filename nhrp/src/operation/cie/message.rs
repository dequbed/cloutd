use crate::cie::buffer::CieBuffer;

use std::net::IpAddr::{self, *};
use std::net::Ipv4Addr;

use crate::{Parseable, Emitable, Result, Error};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientInformationEntry {
    pub code: u8,
    pub prefix_len: u8,
    pub mtu: u16,
    pub holding_time: u16,
    pub preference: u8,
    pub client_nbma_addr: Option<IpAddr>,
    pub client_proto_addr: Option<IpAddr>,
}
impl ClientInformationEntry {
    #[allow(dead_code)]
    pub fn new(code: u8, prefix_len: u8, mtu: u16, holding_time: u16, preference: u8, client_nbma_addr: Option<IpAddr>, client_proto_addr: Option<IpAddr>) -> Self {
        ClientInformationEntry {
            code: code,
            prefix_len: prefix_len,
            mtu: mtu,
            holding_time: holding_time,
            preference: preference,
            client_nbma_addr: client_nbma_addr,
            client_proto_addr: client_proto_addr,
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<ClientInformationEntry> for CieBuffer<&'a T> {
    fn parse(&self) -> Result<ClientInformationEntry> {
        let client_nbma_addr = match self.cli_nbma_addr_tl() {
            0 => Ok(None),
            4 => {
                let a = self.cli_nbma_addr();
                let addr = Ipv4Addr::new(a[0], a[1], a[2], a[3]);
                Ok(Some(IpAddr::V4(addr)))
            },
            16 => {
                let a = self.cli_nbma_addr();
                let addr: [u8; 16] = a.try_into().unwrap();
                Ok(Some(IpAddr::V6(addr.into())))
            },
            _ => {
                Err(Error::NotImplemented)
            }
        }?;
        let client_proto_addr = match self.cli_proto_addr_len() {
            0 => Ok(None),
            4 => {
                let a = self.cli_proto_addr();
                let addr = Ipv4Addr::new(a[0], a[1], a[2], a[3]);
                Ok(Some(IpAddr::V4(addr)))
            },
            16 => {
                let a = self.cli_proto_addr();
                let mut addr: [u16; 8] = [0;8];
                Ok(Some(IpAddr::V6(addr.into())))
            },
            _ => {
                Err(Error::NotImplemented)
            }
        }?;
        Ok(ClientInformationEntry {
            code: self.code(),
            prefix_len: self.prefix_len(),
            mtu: self.mtu(),
            holding_time: self.holding_time(),
            preference: self.preference(),
            client_nbma_addr: client_nbma_addr,
            client_proto_addr: client_proto_addr,
        })
    }
}

impl Emitable for ClientInformationEntry {
    fn buffer_len(&self) -> usize {
        let cnal = match self.client_nbma_addr { None => 0, Some(V4(_)) => 4, Some(V6(_)) => 16 };
        let cpal = match self.client_proto_addr { None => 0, Some(V4(_)) => 4, Some(V6(_)) => 16 };
        12 + cnal + cpal
    }

    fn emit(&self, buffer: &mut [u8]) {
        let mut buffer = CieBuffer::new(buffer);
        buffer.set_code(self.code);
        buffer.set_prefix_len(self.prefix_len);
        buffer.set_mtu(self.mtu);
        buffer.set_holding_time(self.holding_time);
        buffer.set_preference(self.preference);
        buffer.set_cli_nbma_saddr_tl(0);

        match self.client_nbma_addr {
            None => buffer.set_cli_nbma_addr_tl(0),
            Some(V4(a)) => {
                let a = a.octets();
                buffer.set_cli_nbma_addr_tl(a.len() as u8);
                buffer.cli_nbma_addr_mut().copy_from_slice(&a);
            }
            Some(V6(a)) => {
                let a = a.octets();
                buffer.set_cli_nbma_addr_tl(a.len() as u8);
                buffer.cli_nbma_addr_mut().copy_from_slice(&a);
            }
        };
        match self.client_proto_addr {
            None => buffer.set_cli_proto_addr_len(0),
            Some(V4(a)) => {
                let a = a.octets();
                buffer.set_cli_proto_addr_len(a.len() as u8);
                buffer.cli_proto_addr_mut().copy_from_slice(&a);
            },
            Some(V6(a)) => {
                let a = a.octets();
                buffer.set_cli_proto_addr_len(a.len() as u8);
                buffer.cli_proto_addr_mut().copy_from_slice(&a);
            }
        };
    }
}
