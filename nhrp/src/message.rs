use crate::{Error, Parseable, Emitable, Result};
use super::{NhrpBuffer, FixedHeader};
use super::extensions::{Extension, ExtensionIterator};
use super::operation::Operation;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub header: FixedHeader,
    pub operation: Operation,
    pub extensions: Vec<Extension>
}

impl NhrpMessage {
    pub fn new(header: FixedHeader, operation: Operation, extensions: Vec<Extension>) -> Self {
        NhrpMessage {
            header: header,
            operation: operation,
            extensions: extensions,
        }
    }

    pub fn into_parts(self) -> (FixedHeader, Operation, Vec<Extension>) {
        (self.header, self.operation, self.extensions)
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) -> crate::Result<usize> {
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
    fn parse(&self) -> crate::Result<NhrpMessage> {
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
                let msg: PurgeMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::PurgeRequest(msg)
            },
            PurgeReply => {
                let msg: PurgeMessage
                    = OperationBuffer::new(&self.payload()).parse()?;
                Operation::PurgeReply(msg)
            },
            _ => unimplemented!(),
        };

        let extensioni = ExtensionIterator::new(self.extensions());
        let mut extensions = Vec::new();
        for e in extensioni {
            match e {
                // FIXME: Gracefully handle extensions we don't recognice but aren't compulsory
                Ok(e) => extensions.push(e.parse()?),
                Err(e) => return Err(e),
            }
        }

        Ok(NhrpMessage::new(header, operation, extensions))
    }
}

impl Emitable for NhrpMessage {
    fn buffer_len(&self) -> usize {
        use crate::operation::Operation::*;
        let payload_len = match self.operation {
            ResolutionRequest(ref msg) => msg.buffer_len(),
            ResolutionReply(ref msg) => msg.buffer_len(),
            RegistrationRequest(ref msg) => msg.buffer_len(),
            RegistrationReply(ref msg) => msg.buffer_len(),
            PurgeRequest(ref msg) => msg.buffer_len(),
            PurgeReply(ref msg) => msg.buffer_len(),
        };
        payload_len + self.extensions.iter().fold(0, |s,e| s + e.length()) + self.header.buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);

        let eoff = self.extensions.iter().fold(0, |s,e| s + e.length());
        let end = if eoff == 0 { self.buffer_len() } else { eoff };
        {
            let payload = &mut buffer[self.header.buffer_len()..end];

            use crate::operation::Operation::*;
            match self.operation {
                ResolutionRequest(ref msg) => msg.emit(payload),
                ResolutionReply(ref msg) => msg.emit(payload),
                RegistrationRequest(ref msg) => msg.emit(payload),
                RegistrationReply(ref msg) => msg.emit(payload),
                PurgeRequest(ref msg) => msg.emit(payload),
                PurgeReply(ref msg) => msg.emit(payload),
            }
        }

        {
            let buffer = &mut buffer[eoff..self.buffer_len()];
            self.extensions.emit(buffer);
        }

        let mut mbuffer = NhrpBuffer::new(buffer);
        mbuffer.set_length(self.buffer_len() as u16);
        mbuffer.set_extoffset(eoff as u16);
        mbuffer.set_checksum(0);
        let chksum = mbuffer.calculate_checksum();
        mbuffer.set_checksum(chksum);
    }
}
