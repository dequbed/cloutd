use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolutionRequestMessage {
    header: CommonHeader,
    cie: Option<ClientInformationEntry>,
}
