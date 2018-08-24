use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    ResolutionRequest(ResolutionRequestMessage),
    ResolutionReply(ResolutionReplyMessage),
    RegistrationRequest(RegistrationRequestMessage),
    RegistrationReply(RegistrationReplyMessage),
    PurgeRequest(PurgeMessage),
    PurgeReply(PurgeMessage),
}

use NhrpOp;
impl Operation {
    #[allow(dead_code)]
    pub fn optype(&self) -> NhrpOp {
        use Operation::*;
        use NhrpOp as N;
        match *self {
            ResolutionRequest(_) => N::ResolutionRequest,
            ResolutionReply(_) => N::ResolutionReply,
            RegistrationRequest(_) => N::RegistrationRequest,
            RegistrationReply(_) => N::RegistrationReply,
            PurgeRequest(_) => N::PurgeRequest,
            PurgeReply(_) => N::PurgeReply,
        }
    }
}
