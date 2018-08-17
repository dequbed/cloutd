use {Parseable, Emitable, Result, Error};
use super::{NhrpBuffer, FixedHeader};
use super::operation::Operation;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub header: FixedHeader,
    pub operation: Operation,
}

impl NhrpMessage {
    pub fn into_parts(self) -> (FixedHeader, Operation) {
        (self.header, self.operation)
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        if self.header.length() as usize > buffer.len() {
            Err(Error::Exhausted)
        } else {
            self.emit(buffer);
            Ok(self.header.length() as usize)
        }
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<Self> {
        match NhrpBuffer::new_checked(buffer) {
            Ok(buffer) => buffer.parse(),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NhrpMessage> for NhrpBuffer<&'a T> {
    fn parse(&self) -> Result<NhrpMessage> {
        let header = <Self as Parseable<FixedHeader>>::parse(self)?;

        use super::NhrpOp::*;
        let operation = match header.optype() {
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
            operation: operation,
        })
    }
}

impl Emitable for NhrpMessage {
    fn buffer_len(&self) -> usize {
        use nhrp::operation::Operation::*;
        let payload_len = match self.operation {
            ResolutionRequest(ref msg) => msg.buffer_len(),
            ResolutionReply(ref msg) => msg.buffer_len(),
            RegistrationRequest(ref msg) => msg.buffer_len(),
        };
        payload_len + self.header.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        let buffer = &mut buffer[self.header.buffer_len()..self.header.length() as usize];

        use nhrp::operation::Operation::*;
        match self.operation {
            ResolutionRequest(ref msg) => msg.emit(buffer),
            ResolutionReply(ref msg) => msg.emit(buffer),
            RegistrationRequest(ref msg) => msg.emit(buffer),
        }
    }
}
