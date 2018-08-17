#![allow(dead_code)]
use super::header::*;
use {Field, Index, Rest, Result, Error};
use byteorder::{ByteOrder, BigEndian};


const AFN: Field = 0..2;
const PROTYPE: Field = 2..4;
const SNAP: Field = 4..9;
const HOPCOUNT: Index = 9;
const PKTSIZE: Field = 10..12;
const CHECKSUM: Field = 12..14;
const EXTOFFSET: Field = 14..16;
const VERSION: Index = 16;
const OPTYPE: Index = 17;
const PAYLOAD: Rest = 18..;

pub const FIXED_HEADER_LEN: usize = PAYLOAD.start;

pub struct NhrpBuffer<T> {
    buffer: T
}

impl<T: AsRef<[u8]>> NhrpBuffer<T> {
    pub fn new(buffer: T) -> NhrpBuffer<T> {
        NhrpBuffer { buffer: buffer }
    }

    pub fn new_checked(buffer: T) -> Result<NhrpBuffer<T>> {
        let packet = Self::new(buffer);
        packet.check_buffer_length()?;
        Ok(packet)
    }

    fn check_buffer_length(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < PKTSIZE.end || len < self.length() as usize {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn payload_length(&self) -> usize {
        let total_length = self.length() as usize;
        let payload_offset = PAYLOAD.start;
        total_length - payload_offset
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn afn(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[AFN])
    }

    pub fn protype(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[PROTYPE])
    }

    pub fn prosnap(&self) -> [u8; 5] {
        let data = self.buffer.as_ref();
        let mut s = [0; 5];
        s.copy_from_slice(&data[SNAP]);
        s
    }

    pub fn hopcount(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[HOPCOUNT]
    }

    pub fn length(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[PKTSIZE])
    }

    pub fn checksum(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[CHECKSUM])
    }

    pub fn extoffset(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[EXTOFFSET])
    }

    pub fn version(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[VERSION]
    }

    pub fn optype(&self) -> NhrpOp {
        let data = self.buffer.as_ref();
        data[OPTYPE].into()
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> NhrpBuffer<&'a T> {
    pub fn payload(&self) -> &'a [u8] {
        let range = PAYLOAD.start..self.extoffset() as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }

    pub fn extensions(&self) -> &'a [u8] {
        let range = (self.extoffset() as usize)..;
        let data = self.buffer.as_ref();
        &data[range]
    }
}

impl<'a, T: AsRef<[u8]> + AsMut<[u8]> + ?Sized> NhrpBuffer<&'a mut T> {
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let range = PAYLOAD.start..self.extoffset() as usize;
        let data = self.buffer.as_mut();
        &mut data[range]
    }

    pub fn extensions_mut(&mut self) -> &mut [u8] {
        let range = (self.extoffset() as usize)..;
        let data = self.buffer.as_mut();
        &mut data[range]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> NhrpBuffer<T> {
    pub fn set_afn(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[AFN], value)
    }

    pub fn set_protype(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[PROTYPE], value)
    }

    pub fn set_prosnap(&mut self, value: [u8; 5]) {
        let data = self.buffer.as_mut();
        data[SNAP].copy_from_slice(&value);
    }

    pub fn set_hopcount(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[HOPCOUNT] = value
    }

    pub fn set_length(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[PKTSIZE], value)
    }

    pub fn set_checksum(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[CHECKSUM], value)
    }

    pub fn set_extoffset(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[EXTOFFSET], value)
    }

    pub fn set_version(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[VERSION] = value
    }

    pub fn set_optype(&mut self, value: NhrpOp) {
        let data = self.buffer.as_mut();
        data[OPTYPE] = value.into()
    }
}
