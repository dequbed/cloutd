use super::{NhrpBuffer, NhrpHeader, NhrpMandatory, ClientInformationEntry};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NhrpMessage {
    pub fixed: NhrpHeader,
    pub mandatory: NhrpMandatory,
    pub cies: Vec<ClientInformationEntry>,
}
