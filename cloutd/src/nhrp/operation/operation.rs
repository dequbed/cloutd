use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    ResolutionRequest(ResolutionRequestMessage),
    ResolutionReply(ResolutionReplyMessage),
    RegistrationRequest(RegistrationRequestMessage),
    RegistrationReply(RegistrationReplyMessage),
    PurgeRequest(PurgeRequestMessage),
    PurgeReply(PurgeReplyMessage),
}
