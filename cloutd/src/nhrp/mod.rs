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
pub mod recv_nhrp;
pub mod send_nhrp;
pub mod frame;
pub mod protocol;
pub mod codec;
pub mod buffer;

pub use self::socket::*;
pub use self::codec::NhrpCodec;
pub use self::frame::NhrpFramed;
pub use self::recv_nhrp::*;
