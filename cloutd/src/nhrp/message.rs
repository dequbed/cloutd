use {Parseable, Emitable, Result, Error};
use super::{NhrpBuffer, NhrpHeader, NhrpMandatory, MandatoryHeaderBuffer};
use super::mandatory::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub header: NhrpHeader,
    pub message: NhrpMandatory,
}

impl NhrpMessage {
    pub fn into_parts(self) -> (NhrpHeader, NhrpMandatory) {
        (self.header, self.message)
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        if self.header.length() as usize > buffer.len() {
            Err(Error::Exhausted)
        } else {
            self.emit(buffer);
            Ok(self.header.length() as usize)
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NhrpMessage> for NhrpBuffer<&'a T> {
    fn parse(&self) -> Result<NhrpMessage> {
        let header = <Self as Parseable<NhrpHeader>>::parse(self)?;

        use super::NhrpOp::*;
        let message = match header.optype() {
            ResolutionRequest => {
                let msg: ResolutionRequestMessage
                    = MandatoryHeaderBuffer::new(&self.payload()).parse()?;
                NhrpMandatory::ResolutionRequest(msg)
            },
            ResolutionReply => {
                let msg: ResolutionReplyMessage
                    = MandatoryHeaderBuffer::new(&self.payload()).parse()?;
                NhrpMandatory::ResolutionReply(msg)
            },
            RegistrationRequest => {
                let msg: RegistrationRequestMessage
                    = MandatoryHeaderBuffer::new(&self.payload()).parse()?;
                NhrpMandatory::RegistrationRequest(msg)
            },
            _ => unimplemented!(),
        };

        Ok(NhrpMessage {
            header: header,
            message: message,
        })
    }
}

impl Emitable for NhrpMessage {
    fn buffer_len(&self) -> usize {
        use nhrp::mandatory::NhrpMandatory::*;
        let payload_len = match self.message {
            ResolutionRequest(ref msg) => msg.buffer_len(),
            ResolutionReply(ref msg) => msg.buffer_len(),
            RegistrationRequest(ref msg) => msg.buffer_len(),
        };
        payload_len + self.header.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        let buffer = &mut buffer[self.header.buffer_len()..self.header.length() as usize];

        use nhrp::mandatory::NhrpMandatory::*;
        match self.message {
            ResolutionRequest(ref msg) => msg.emit(buffer),
            ResolutionReply(ref msg) => msg.emit(buffer),
            RegistrationRequest(ref msg) => msg.emit(buffer),
        }
    }
}
