#![allow(dead_code)]
use crate::{Field, Rest, Result, Error};
use super::extension::{ExtensionType, END_OF_EXTENSIONS};

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
        u16::from_be_bytes(data[CUTYPE].try_into().unwrap())
    }

    pub fn extensiontype(&self) -> ExtensionType {
        (self.cutype() & 0x3FFF).into()
    }

    pub fn compulsory(&self) -> bool {
        self.cutype() & 0x8000 == 0x8000
    }

    pub fn payload_length(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from_be_bytes(data[LENGTH].try_into().unwrap())
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
        let curr = u16::from_be_bytes(data[CUTYPE].try_into().unwrap());
        let value: u16 = value.into();
        let value = value & 0x3FFF;

        data[CUTYPE].copy_from_slice(&(value + (curr & 0x8000)).to_be_bytes());
    }

    pub fn set_compulsory(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        let curr = u16::from_be_bytes(data[CUTYPE].try_into().unwrap());
        let value: u16 = if value { 0x8000 } else { 0 };
        data[CUTYPE].copy_from_slice(&(value + (curr & 0x3FFF)).to_be_bytes());
    }

    pub fn set_length(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        data[LENGTH].copy_from_slice(&value.to_be_bytes());
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
                if extbuffer.extensiontype() == END_OF_EXTENSIONS {
                    self.position = self.buffer.as_ref().len();
                }
                Some(Ok(extbuffer))
            },
            Err(_) => {
                // Fuse the iterator, invalid buffers can only happen if stuff goes really wrong.
                self.position = self.buffer.as_ref().len();
                None
            }
        }
    }
}
