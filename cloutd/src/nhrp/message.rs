use super::{NhrpBuffer, NhrpHeader, NhrpMandatory};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub header: NhrpHeader,
    pub message: NhrpMandatory,
}
