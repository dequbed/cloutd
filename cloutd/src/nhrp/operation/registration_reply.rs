use {Parseable, Emitable, Result};
use super::*;
use super::cie::buffer::CieBuffer;
use super::cie::message::ClientInformationEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegistrationReplyMessage {
    header: CommonHeader,
    cie: ClientInformationEntry,
}

impl RegistrationReplyMessage {
    pub fn new(header: CommonHeader, cie: ClientInformationEntry) -> Self {
        RegistrationReplyMessage {
            header: header, cie: cie,
        }
    }

    pub fn split(self) -> (CommonHeader, ClientInformationEntry) {
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
