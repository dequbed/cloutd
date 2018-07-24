use std::io;
use std::error::Error;

use bytes::{BufMut, BytesMut};
use tokio_codec::{Decoder, Encoder};

use super::protocol::{NhrpPacket, parse};

pub struct NhrpCodec;

impl Decoder for NhrpCodec {
    type Item = NhrpPacket;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        match parse(buf) {
            Ok((_rem, p)) => {
                Ok(Some(p))
            }
            Err(ref e) if e.is_incomplete() => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.description()))
        }
    }
}

impl Encoder for NhrpCodec {
    type Item = NhrpPacket;
    type Error = io::Error;

    fn encode(&mut self, _item: Self::Item, _buf: &mut BytesMut) -> io::Result<()> {
        Ok(())
    }
}
