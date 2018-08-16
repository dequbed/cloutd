use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NhrpMandatory {
    ResolutionRequest(ResolutionRequestMessage),
}
