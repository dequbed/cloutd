#![allow(dead_code)]
use {Field, Rest, Result, Error};
use byteorder::{ByteOrder, BigEndian};

use super::extension::{ExtensionType, EndOfExtensionsType};

const CUTYPE: Field = 0..2;
const LENGTH: Field = 2..4;
const PAYLOAD: Rest = 4..;

pub const EXTENSION_HEADER_LEN: usize = PAYLOAD.start;

pub struct ExtensionBuffer<T> {
    buffer: T,
}

impl<T: AsRef<[u8]>> ExtensionBuffer<T> {
    pub fn new(buffer: T) -> ExtensionBuffer<T> {
        ExtensionBuffer { buffer: buffer }
    }

    pub fn new_checked(buffer: T) -> Result<ExtensionBuffer<T>> {
        let ext = Self::new(buffer);
        ext.check_buffer_length()?;
        Ok(ext)
    }

    fn check_buffer_length(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < LENGTH.end || len < self.length() as usize {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    fn cutype(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[CUTYPE])
    }

    pub fn extensiontype(&self) -> ExtensionType {
        (self.cutype() & 0x3FFF).into()
    }

    pub fn compulsory(&self) -> bool {
        self.cutype() & 0x8000 == 0x8000
    }

    pub fn payload_length(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[LENGTH])
    }
    pub fn length(&self) -> u16 {
        self.payload_length() + 4
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> ExtensionBuffer<&'a T> {
    pub fn payload(&self) -> &'a [u8] {
        let range = PAYLOAD.start..(PAYLOAD.start + self.length() as usize);
        let data = self.buffer.as_ref();
        &data[range]
    }
}

impl<'a, T: AsRef<[u8]> + AsMut<[u8]> + ?Sized> ExtensionBuffer<&'a mut T> {
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let range = PAYLOAD.start..(PAYLOAD.start + self.length() as usize);
        let data = self.buffer.as_mut();
        &mut data[range]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ExtensionBuffer<T> {
    pub fn set_extensiontype(&mut self, value: ExtensionType) {
        let data = self.buffer.as_mut();
        let curr = BigEndian::read_u16(&data[CUTYPE]);
        let value: u16 = value.into();
        let value = value & 0x3FFF;
        BigEndian::write_u16(&mut data[CUTYPE], value + (curr & 0x8000))
    }

    pub fn set_compulsory(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        let curr = BigEndian::read_u16(&data[CUTYPE]);
        let value: u16 = if value { 0x8000 } else { 0 };
        BigEndian::write_u16(&mut data[CUTYPE], value + (curr & 0x3FFF))
    }

    pub fn set_length(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[LENGTH], value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionIterator<T> {
    position: usize,
    buffer: T,
}

impl<T> ExtensionIterator<T> {
    pub fn new(buffer: T) -> Self {
        ExtensionIterator {
            position: 0,
            buffer: buffer,
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized + 'a> Iterator for ExtensionIterator<&'a T> {
    type Item = Result<ExtensionBuffer<&'a [u8]>>;

    fn next(&mut self) -> Option<Self::Item> {
        // We already exhausted the buffer or got an invalid/empty one.
        if self.position >= self.buffer.as_ref().len() {
            return None
        }

        match ExtensionBuffer::new_checked(&self.buffer.as_ref()[self.position..]) {
            Ok(extbuffer) => {
                self.position += extbuffer.length() as usize;
                // Fuse buffer on EoE
                if extbuffer.extensiontype() == EndOfExtensionsType {
                    self.position = self.buffer.as_ref().len();
                }
                Some(Ok(extbuffer))
            },
            Err(e) => {
                // Fuse the iterator, invalid buffers can only happen if stuff goes really wrong.
                self.position = self.buffer.as_ref().len();
                None
            }
        }
    }
}
