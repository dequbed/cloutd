use {Parseable, Emitable, Result, Error};
use super::{NhrpBuffer, FixedHeader};
use super::operation::Operation;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub header: FixedHeader,
    pub operation: Operation,
}

impl NhrpMessage {
    pub fn new(header: FixedHeader, operation: Operation) -> Self {
        NhrpMessage {
            header: header,
            operation: operation,
        }
    }

    pub fn into_parts(self) -> (FixedHeader, Operation) {
        (self.header, self.operation)
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        if self.buffer_len() as usize > buffer.len() {
            Err(Error::Exhausted)
        } else {
            self.emit(buffer);
            Ok(self.buffer_len() as usize)
        }
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<Self> {
        match NhrpBuffer::new_checked(buffer) {
            Ok(buffer) => buffer.parse(),
            Err(e) => Err(e),
        }
    }
}

use super::NhrpOp::*;
impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NhrpMessage> for NhrpBuffer<&'a T> {
    fn parse(&self) -> Result<NhrpMessage> {
        let header = <Self as Parseable<FixedHeader>>::parse(self)?;

        use super::operation::*;
        let operation = match header.optype() {
            ResolutionRequest => {
                let msg: ResolutionRequestMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::ResolutionRequest(msg)
            },
            ResolutionReply => {
                let msg: ResolutionReplyMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::ResolutionReply(msg)
            },
            RegistrationRequest => {
                let msg: RegistrationRequestMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::RegistrationRequest(msg)
            },
            RegistrationReply => {
                let msg: RegistrationReplyMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::RegistrationReply(msg)
            },
            PurgeRequest => {
                let msg: PurgeRequestMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::PurgeRequest(msg)
            },
            PurgeReply => {
                let msg: PurgeReplyMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::PurgeReply(msg)
            },
            _ => unimplemented!(),
        };

        //let extensions = <Self as Parseable<Extensions>>::parse(&self.extensions())?

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
            RegistrationReply(ref msg) => msg.buffer_len(),
            PurgeRequest(ref msg) => msg.buffer_len(),
            PurgeReply(ref msg) => msg.buffer_len(),
        };
        payload_len + self.header.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);

        {
            let payload = &mut buffer[self.header.buffer_len()..self.buffer_len() as usize];

            use nhrp::operation::Operation::*;
            match self.operation {
                ResolutionRequest(ref msg) => msg.emit(payload),
                ResolutionReply(ref msg) => msg.emit(payload),
                RegistrationRequest(ref msg) => msg.emit(payload),
                RegistrationReply(ref msg) => msg.emit(payload),
                PurgeRequest(ref msg) => msg.emit(payload),
                PurgeReply(ref msg) => msg.emit(payload),
            }
        }

        let mut mbuffer = NhrpBuffer::new(buffer);
        mbuffer.set_length(self.buffer_len() as u16);
        mbuffer.set_extoffset(0);
        mbuffer.set_checksum(0);
        let chksum = mbuffer.calculate_checksum();
        mbuffer.set_checksum(chksum);
    }
}
