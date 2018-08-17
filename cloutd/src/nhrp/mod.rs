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

pub mod socket;
pub use self::socket::*;
pub mod codec;
pub use codec::*;
pub mod buffer;
pub use self::buffer::*;
pub mod header;
pub use self::header::*;
pub mod message;
pub use self::message::*;
pub mod operation;
pub use self::operation::*;
