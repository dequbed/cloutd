use super::*;
use super::cie::message::ClientInformationEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolutionRequestMessage {
    header: CommonHeader,
    cie: Option<ClientInformationEntry>,
}
