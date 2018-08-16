use super::{NhrpBuffer, NhrpHeader, NhrpMessage};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub header: NhrpHeader,
    pub message: NhrpMessage,
}
