use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NhrpMandatory {
    ResolutionRequest(CommonHeader),
    ResolutionReply(CommonHeader),
    RegistrationRequest(CommonHeader),
    RegistrationReply(CommonHeader),
    PurgeRequest(CommonHeader),
    PurgeReply(CommonHeader),
    ErrorIndication(ErrorHeader),
    Other(Vec<u8>),
}
