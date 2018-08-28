use std;
use std::io;

use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Truncated,
    Exhausted,
    NotImplemented,
    Invalid,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            Io(ref err) => write!(f, "IO error: {}", err),
            Truncated => write!(f, "Packet was truncated!"),
            Exhausted => write!(f, "Buffer to small!"),
            NotImplemented => write!(f, "Not implemented"),
            Invalid => write!(f, "Invalid Reqtype"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Error {
        Error::Io(value)
    }
}
