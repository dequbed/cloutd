use {Parseable, Emitable, Result, Error};
use super::*;
use super::cie::buffer::CieBuffer;
use super::cie::message::ClientInformationEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolutionRequestMessage {
    header: CommonHeader,
    cie: Option<ClientInformationEntry>,
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<ResolutionRequestMessage> for OperationBuffer<&'a T> {
    fn parse(&self) -> Result<ResolutionRequestMessage> {
        let header = <Self as Parseable<CommonHeader>>::parse(self)?;
        let cie = match CieBuffer::new_checked(self.payload()) {
            Ok(buffer) => Some(buffer.parse()?),
            Err(Error::Truncated) => None,
            Err(e) => return Err(e),
        };

        Ok(ResolutionRequestMessage {
            header: header,
            cie: cie,
        })
    }
}

impl Emitable for ResolutionRequestMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len() + self.cie.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        match self.cie {
            Some(_) => {
                let endoffset = self.header.buffer_len() + self.cie.buffer_len();
                let buffer = &mut buffer[self.header.buffer_len()..endoffset];
                self.cie.emit(buffer);
            },
            None => ()
        }
    }
}
