use std::io;
use thiserror::Error;
use miette::Diagnostic;

#[derive(Debug, Error, Diagnostic)]
#[error("cloutd encountered an error")]
pub struct CloutdError {
    #[source]
    #[diagnostic_source]
    source: Error,

    #[related]
    others: Vec<CloutdError>,
}
impl CloutdError {
    pub fn new(source: Error, others: Vec<CloutdError>) -> Self {
        Self { source, others }
    }

    pub fn from(source: impl Into<Error>) -> Self {
        Self { source: source.into(), others: Vec::new() }
    }

    pub fn and(&mut self, other: CloutdError) -> &mut Self {
        self.others.push(other);
        self
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("an IO error occured")]
    Io(#[from] io::Error),
}
impl Into<CloutdError> for Error {
    fn into(self) -> CloutdError {
        CloutdError::from(self)
    }
}