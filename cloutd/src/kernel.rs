use std::io;
use thiserror::Error;
use miette::Diagnostic;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("Opening rt-netlink connection failed")]
    #[diagnostic(code("rtnl::conn::open"))]
    ConnectionError(#[source] io::Error),
}