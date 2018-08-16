#![allow(dead_code)]
use super::header::*;
use {Field, Index, Rest, Result, Error};
use byteorder::{ByteOrder, BigEndian};

const SHTL: Index = 0;
const SSTL: Index = 1;
const SRC_PROTO_LEN: Index = 2;
const DST_PROTO_LEN: Index = 3;
const FLAGS: Field = 4..6;
const REQUEST_ID: Field = 6..10;
const ADDRS: Rest = 10..;

pub struct MandatoryHeaderBuffer<T> {
    buffer: T,
}

impl<T: AsRef<[u8]>> MandatoryHeaderBuffer<T> {
    pub fn new(buffer: T) -> MandatoryHeaderBuffer<T> {
        MandatoryHeaderBuffer { buffer: buffer }
    }

    pub fn new_checked(buffer: T) -> Result<MandatoryHeaderBuffer<T>> {
        let packet = Self::new(buffer);
        packet.check_buffer_length()?;
        Ok(packet)
    }

    fn check_buffer_length(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < DST_PROTO_LEN || len < self.length() as usize {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn length(&self) -> u32 {
        let shtl: u8 = self.src_nbma_addr_tl().into();
        let sstl: u8 = self.src_nbma_saddr_tl().into();

        8 + shtl as u32
          + sstl as u32
          + self.src_proto_addr_len() as u32
          + self.dst_proto_addr_len() as u32
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn src_nbma_addr_tl(&self) -> AddrTL {
        let data = self.buffer.as_ref();
        data[SHTL].into()
    }
    pub fn src_nbma_addr_offset(&self) -> usize {
        ADDRS.start
    }

    pub fn src_nbma_saddr_tl(&self) -> AddrTL {
        let data = self.buffer.as_ref();
        data[SSTL].into()
    }
    pub fn src_nbma_saddr_offset(&self) -> usize {
        let shtl: u8 = self.src_nbma_addr_tl().into();
        self.src_nbma_addr_offset() + shtl as usize
    }

    pub fn src_proto_addr_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[SRC_PROTO_LEN]
    }
    pub fn src_proto_addr_offset(&self) -> usize {
        let sstl: u8 = self.src_nbma_saddr_tl().into();
        self.src_nbma_saddr_offset() + sstl as usize
    }

    pub fn dst_proto_addr_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[DST_PROTO_LEN]
    }
    pub fn dst_proto_addr_offset(&self) -> usize {
        self.src_proto_addr_offset() + self.src_proto_addr_len() as usize
    }

    pub fn flags(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[FLAGS])
    }

    pub fn request_id(&self) -> u32 {
        let data = self.buffer.as_ref();
        BigEndian::read_u32(&data[REQUEST_ID])
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> MandatoryHeaderBuffer<&'a T> {
    pub fn src_nbma_addr(&self) -> &'a [u8] {
        let shtl: u8 = self.src_nbma_addr_tl().into();
        let range = self.src_nbma_addr_offset()..shtl as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }

    pub fn src_nbma_saddr(&self) -> &'a [u8] {
        let sstl: u8 = self.src_nbma_saddr_tl().into();
        let range = self.src_nbma_saddr_offset()..sstl as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }

    pub fn src_proto_addr(&self) -> &'a [u8] {
        let range = self.src_proto_addr_offset()..self.src_proto_addr_len() as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }

    pub fn dst_proto_addr(&self) -> &'a [u8] {
        let range = self.dst_proto_addr_offset()..self.dst_proto_addr_len() as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }
}

impl<'a, T: AsRef<[u8]> + AsMut<[u8]> + ?Sized> MandatoryHeaderBuffer<&'a mut T> {
    pub fn src_nbma_addr_mut(&mut self) -> &mut [u8]{
        let shtl: u8 = self.src_nbma_addr_tl().into();
        let range = self.src_nbma_addr_offset()..shtl as usize;
        let mut data = self.buffer.as_mut();
        &mut data[range]
    }

    pub fn src_nbma_saddr_mut(&mut self) -> &mut [u8]{
        let sstl: u8 = self.src_nbma_saddr_tl().into();
        let range = self.src_nbma_saddr_offset()..sstl as usize;
        let mut data = self.buffer.as_mut();
        &mut data[range]
    }

    pub fn src_proto_addr_mut(&mut self) -> &mut [u8]{
        let range = self.src_proto_addr_offset()..self.src_proto_addr_len() as usize;
        let mut data = self.buffer.as_mut();
        &mut data[range]
    }

    pub fn dst_proto_addr_mut(&mut self) -> &mut [u8]{
        let range = self.dst_proto_addr_offset()..self.dst_proto_addr_len() as usize;
        let mut data = self.buffer.as_mut();
        &mut data[range]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> MandatoryHeaderBuffer<T> {
    pub fn set_src_nbma_addr_tl(&mut self, value: AddrTL) {
        let mut data = self.buffer.as_mut();
        data[SHTL] = value.into()
    }

    pub fn set_src_nbma_saddr_tl(&mut self, value: AddrTL) {
        let mut data = self.buffer.as_mut();
        data[SSTL] = value.into()
    }

    pub fn set_src_proto_addr_len(&mut self, value: u8) {
        let mut data = self.buffer.as_mut();
        data[SRC_PROTO_LEN] = value
    }

    pub fn set_dst_proto_addr_len(&mut self, value: u8) {
        let mut data = self.buffer.as_mut();
        data[DST_PROTO_LEN] = value
    }

    pub fn set_flags(&mut self, value: u16) {
        let mut data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[FLAGS], value)
    }

    pub fn set_request_id(&mut self, value: u32) {
        let mut data = self.buffer.as_mut();
        BigEndian::write_u32(&mut data[REQUEST_ID], value)
    }
}
