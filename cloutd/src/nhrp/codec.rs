use {Emitable, Error};
use std::marker::PhantomData;
use bytes::{BufMut, BytesMut};
use tokio_codec::{Decoder, Encoder};
use super::{NhrpBuffer, NhrpMessage,};

pub struct NhrpCodec<T> {
    phantom: PhantomData<T>,
}

impl<T> NhrpCodec<T> {
    pub fn new() -> Self {
        NhrpCodec {
            phantom: PhantomData,
        }
    }
}

impl<T> Decoder for NhrpCodec<NhrpBuffer<T>>
    where T: AsRef<[u8]> + From<BytesMut>,
{
    type Item = NhrpBuffer<T>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = match NhrpBuffer::new_checked(src.as_ref()) {
            Ok(buf) => buf.length() as usize,
            Err(Error::Truncated) => return Ok(None),
            Err(e) => return Err(e),
        };
        let buf = src.split_to(len);
        Ok(Some(NhrpBuffer::new(T::from(buf))))
    }
}

impl<T: AsRef<[u8]>> Encoder for NhrpCodec<NhrpBuffer<T>> {
    type Item = NhrpBuffer<T>;
    type Error = Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.extend_from_slice(msg.into_inner().as_ref());
        Ok(())
    }
}

impl Decoder for NhrpCodec<NhrpMessage> {
    type Item = NhrpMessage;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = match NhrpBuffer::new_checked(src.as_ref()) {
            Ok(buf) => buf.length() as usize,
            Err(Error::Truncated) => return Ok(None),
            Err(e) => return Err(e),
        };
        let buf = src.split_to(len);
        Ok(Some(NhrpMessage::from_bytes(&buf)?))
    }
}

impl Encoder for NhrpCodec<NhrpMessage> {
    type Item = NhrpMessage;
    type Error = Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let msg_len = msg.buffer_len();
        buf.reserve(msg_len);
        unsafe {
            let size = msg.to_bytes(&mut buf.bytes_mut()[..])?;
            buf.advance_mut(size);
        };

        Ok(())
    }
}
