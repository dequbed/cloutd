use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NhrpMandatory {
    ResolutionRequest(ResolutionRequestMessage),
    ResolutionReply(ResolutionReplyMessage),
    RegistrationRequest(RegistrationRequestMessage),
    /*
     *RegistrationReply(RegistrationReplyMessage),
     *PurgeRequest(PurgeRequestMessage),
     *PurgeReply(PurgeReplyMessage),
     *ErrorIndication(ErrorMessage),
     */
}
