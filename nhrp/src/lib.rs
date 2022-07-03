/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use core::ops::{Range, RangeFrom};

/// Represent a multi-bytes field with a fixed size in a packet
pub(crate) type Field = Range<usize>;
/// Represent a field that starts at a given index in a packet
pub(crate) type Rest = RangeFrom<usize>;
/// Represent a field of exactly one byte in a packet
pub(crate) type Index = usize;

// pub mod socket;
// pub use self::socket::*;
// pub mod codec;
// pub use codec::*;
// pub mod framed;
// pub use framed::*;
pub mod buffer;
pub use self::buffer::*;
pub mod header;
pub use self::header::*;
pub mod message;
pub use self::message::*;
pub mod operation;
pub use self::operation::*;

pub mod extensions;
pub use self::extensions::*;

pub enum Error {
    Truncated,
    Exhausted,
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, Error>;
pub trait Parseable<T> {
    fn parse(&self) -> Result<T>;
}

/// A type that implements `Emitable` can be serialized.
pub trait Emitable {
    /// Return the length of the serialized data.
    fn buffer_len(&self) -> usize;

    /// Serialize this types and write the serialized data into the given buffer.
    ///
    /// # Panic
    ///
    /// This method panic if the buffer is not big enough. You **must** make sure the buffer is big
    /// enough before calling this method. You can use
    /// [`buffer_len()`](trait.Emitable.html#method.buffer_len) to check how big the storage needs
    /// to be.
    fn emit(&self, buffer: &mut [u8]);
}

impl<T: Emitable> Emitable for Option<T> {
    fn buffer_len(&self) -> usize {
        match *self {
            Some(ref v) => v.buffer_len(),
            None => 0,
        }
    }

    fn emit(&self, buffer: &mut [u8]) {
        if let Some(ref v) = *self {
            v.emit(buffer)
        }
    }
}

impl<T: Emitable> Emitable for Vec<T> {
    fn buffer_len(&self) -> usize {
        self.iter().fold(0usize, |sum, e| sum + e.buffer_len())
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.iter().fold(buffer, |buf, e| {
            e.emit(buf);
            &mut buf[e.buffer_len()..]
        });
    }
}
