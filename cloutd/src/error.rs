use std;
use std::io;

use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Truncated,
    Exhausted,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            Io(ref err) => write!(f, "IO error: {}", err),
            Truncated => write!(f, "Packet was truncated!"),
            Exhausted => write!(f, "Buffer to small!"),
        }
    }
}
