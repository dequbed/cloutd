use {Field, Index, Rest, Result, Error};
use byteorder::{ByteOrder, NativeEndian};

const AFN: Field = 0..2;
const PROTOTYPE: Field = 2..4;
const SNAP: Field = 4..9;
const HOPCOUNT: Index = 9;
const PACKETSIZE: Field = 10..12;
const CHECKSUM: Field = 12..14;
const EXTOFFSET: Field = 14..16;
const VERSION: Index = 16;
const OPTYPE: Index = 17;
const SHTL: Index = 18;
const SSTL: Index = 19;
const PAYLOAD: Rest = 20..;

pub const FIXED_HEADER_LEN: usize = PAYLOAD.start;

pub struct NhrpBuffer<T> {
    buffer: T
}

impl<T: AsRef<[u8]>> NhrpBuffer<T> {
    pub fn new(buffer: T) -> NhrpBuffer<T> {
        NhrpBuffer { buffer: buffer }
    }

    fn check_buffer_length(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < PACKETSIZE.end || len < self.length() as usize {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }
}
