use {Parseable, Emitable, Result, Error};
use super::*;
use super::cie::buffer::{CieBuffer, CieIterator};
use super::cie::message::ClientInformationEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolutionReplyMessage {
    header: CommonHeader,
    cie: Vec<ClientInformationEntry>,
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
