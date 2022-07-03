use crate::{Parseable, Emitable, Result};
use super::*;
use super::cie::buffer::CieIterator;
use super::cie::message::ClientInformationEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegistrationRequestMessage {
    header: CommonHeader,
    cie: Vec<ClientInformationEntry>,
}

impl RegistrationRequestMessage {
    #![allow(dead_code)]

    pub fn new(header: CommonHeader, cie: Vec<ClientInformationEntry>) -> Self {
        RegistrationRequestMessage {
            header: header, cie: cie,
        }
    }

    pub fn header(&self) -> &CommonHeader {
        &self.header
    }

    pub fn into_parts(self) -> (CommonHeader, Vec<ClientInformationEntry>) {
        (self.header, self.cie)
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<RegistrationRequestMessage> for OperationBuffer<&'a T> {
    fn parse(&self) -> Result<RegistrationRequestMessage> {
        let header = <Self as Parseable<CommonHeader>>::parse(self)?;
        let cies = CieIterator::new(self.payload());
        let mut ciev = Vec::new();
        for cie in cies {
            match cie {
                Ok(cie) => ciev.push(cie.parse()?),
                Err(e) => return Err(e),
            }
        }

        Ok(RegistrationRequestMessage {
            header: header,
            cie: ciev,
        })
    }
}

impl Emitable for RegistrationRequestMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len() + self.cie.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        let buffer = &mut buffer[self.header.buffer_len()..];
        self.cie.emit(buffer);
    }
}
