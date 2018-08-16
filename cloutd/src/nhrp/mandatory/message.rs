use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NhrpMessage {
    ResolutionRequest(ResolutionRequestMessage),
}
