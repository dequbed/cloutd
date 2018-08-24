use {Parseable, Emitable, Result};
use super::*;
use super::cie::buffer::CieIterator;
use super::cie::message::ClientInformationEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PurgeMessage {
    header: CommonHeader,
    cie: Vec<ClientInformationEntry>,
}

impl PurgeMessage {
    pub fn header(&self) -> &CommonHeader {
        &self.header
    }

    pub fn cie(&self) -> &Vec<ClientInformationEntry> {
        &self.cie
    }

    pub fn into_parts(self) -> (CommonHeader, Vec<ClientInformationEntry>) {
        (self.header, self.cie)
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<PurgeMessage> for OperationBuffer<&'a T> {
    fn parse(&self) -> Result<PurgeMessage> {
        let header = <Self as Parseable<CommonHeader>>::parse(self)?;
        let cies = CieIterator::new(self.payload());
        let mut ciev = Vec::new();
        for cie in cies {
            match cie {
                Ok(cie) => ciev.push(cie.parse()?),
                Err(e) => return Err(e),
            }
        }

        Ok(PurgeMessage {
            header: header,
            cie: ciev,
        })
    }
}

impl Emitable for PurgeMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len() + self.cie.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        let buffer = &mut buffer[self.header.buffer_len()..];
        self.cie.emit(buffer);
    }
}
