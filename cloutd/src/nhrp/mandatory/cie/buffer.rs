#![allow(dead_code)]
use {Field, Index, Rest, Result, Error};
use byteorder::{ByteOrder, BigEndian};

const CODE: Index = 0;
const PREFIX_LEN: Index = 1;
const MTU: Field = 4..6;
const HOLDING_TIME: Field = 6..8;
const CLI_ADDR_TL: Index = 8;
const CLI_SADDR_TL: Index = 9;
const CLI_PROTO_LEN: Index = 10;
const PREFERENCE: Index = 11;
const ADDRS: Rest = 12..;

pub struct CieBuffer<T> {
    buffer: T,
}

impl<T: AsRef<[u8]>> CieBuffer<T> {
    pub fn new(buffer: T) -> CieBuffer<T> {
        CieBuffer { buffer: buffer }
    }

    pub fn new_checked(buffer: T) -> Result<CieBuffer<T>> {
        let packet = Self::new(buffer);
        packet.check_buffer_length()?;
        Ok(packet)
    }

    fn check_buffer_length(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < CLI_PROTO_LEN || len < self.length() as usize {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    // FIXME: Actually CLI_[S]ADDR_TL is context-specific
    pub fn length(&self) -> u32 {
        12 + self.cli_nbma_addr_tl() as u32
           + self.cli_nbma_saddr_tl() as u32
           + self.cli_proto_addr_len() as u32
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn code(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[CODE]
    }

    pub fn prefix_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[PREFIX_LEN]
    }

    pub fn mtu(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[MTU])
    }

    pub fn holding_time(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[HOLDING_TIME])
    }

    // FIXME: Actually CLI_[S]ADDR_TL is context-specific
    pub fn cli_nbma_addr_tl(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[CLI_ADDR_TL]
    }
    pub fn cli_nbma_addr_offset(&self) -> usize {
        ADDRS.start
    }

    // FIXME: Actually CLI_[S]ADDR_TL is context-specific
    pub fn cli_nbma_saddr_tl(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[CLI_SADDR_TL]
    }
    pub fn cli_nbma_saddr_offset(&self) -> usize {
        self.cli_nbma_addr_offset() + self.cli_nbma_addr_tl() as usize
    }

    pub fn cli_proto_addr_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[CLI_PROTO_LEN]
    }
    pub fn cli_proto_addr_offset(&self) -> usize {
        // FIXME: Actually CLI_[S]ADDR_TL is context-specific
        self.cli_nbma_saddr_offset() + self.cli_nbma_saddr_tl() as usize
    }

    pub fn preference(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[PREFERENCE]
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> CieBuffer<&'a T> {
    // FIXME: Also actually context-specific
    pub fn cli_nbma_addr(&self) -> &'a [u8] {
        let offset = self.cli_nbma_addr_offset();
        let len = self.cli_nbma_addr_tl() as usize;
        let range = offset..(offset+len);
        let data = self.buffer.as_ref();
        &data[range]
    }
    pub fn cli_nbma_saddr(&self) -> &'a [u8] {
        let offset = self.cli_nbma_saddr_offset();
        let len = self.cli_nbma_saddr_tl() as usize;
        let range = offset..(offset+len);
        let data = self.buffer.as_ref();
        &data[range]
    }

    pub fn cli_proto_addr(&self) -> &'a [u8] {
        let offset = self.cli_proto_addr_offset();
        let len = self.cli_proto_addr_len() as usize;
        let range = offset..(offset+len);
        let data = self.buffer.as_ref();
        &data[range]
    }
}

impl<'a, T: AsRef<[u8]> + AsMut<[u8]> + ?Sized> CieBuffer<&'a mut T> {
    // FIXME: Also actually context-specific
    pub fn cli_nbma_addr_mut(&mut self) -> &mut [u8] {
        let offset = self.cli_nbma_addr_offset();
        let len = self.cli_nbma_addr_tl() as usize;
        let range = offset..(offset+len);
        let data = self.buffer.as_mut();
        &mut data[range]
    }
    pub fn cli_nbma_saddr_mut(&mut self) -> &mut [u8] {
        let offset = self.cli_nbma_saddr_offset();
        let len = self.cli_nbma_saddr_tl() as usize;
        let range = offset..(offset+len);
        let data = self.buffer.as_mut();
        &mut data[range]
    }

    pub fn cli_proto_addr_mut(&mut self) -> &mut [u8] {
        let offset = self.cli_proto_addr_offset();
        let len = self.cli_proto_addr_len() as usize;
        let range = offset..(offset+len);
        let data = self.buffer.as_mut();
        &mut data[range]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> CieBuffer<T> {
    pub fn set_code(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[CODE] = value
    }

    pub fn set_prefix_len(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[PREFIX_LEN] = value
    }

    pub fn set_mtu(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[MTU], value)
    }

    pub fn set_holding_time(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        BigEndian::write_u16(&mut data[HOLDING_TIME], value)
    }

    // FIXME: Actually CLI_[S]ADDR_TL is context-specific
    pub fn set_cli_nbma_addr_tl(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[CLI_ADDR_TL] = value
    }

    // FIXME: Actually CLI_[S]ADDR_TL is context-specific
    pub fn set_cli_nbma_saddr_tl(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[CLI_SADDR_TL] = value
    }

    pub fn set_cli_proto_addr_len(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[CLI_PROTO_LEN] = value
    }

    pub fn set_preference(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[PREFERENCE] = value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CieIterator<T> {
    position: usize,
    buffer: T,
}

impl<T> CieIterator<T> {
    pub fn new(buffer: T) -> Self {
        CieIterator {
            position: 0,
            buffer: buffer
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized + 'a> Iterator for CieIterator<&'a T> {
    type Item = Result<CieBuffer<&'a [u8]>>;

    fn next(&mut self) -> Option<Self::Item> {
        // We already exhausted the buffer or got an invalid/empty one.
        if self.position >= self.buffer.as_ref().len() {
            return None
        }

        match CieBuffer::new_checked(&self.buffer.as_ref()[self.position..]) {
            Ok(ciebuffer) => {
                self.position += ciebuffer.length() as usize;
                Some(Ok(ciebuffer))
            },
            Err(e) => {
                // Fuse the iterator. We currently really can't recover from an invalid CIE.
                self.position = self.buffer.as_ref().len();
                Some(Err(e))
            }
        }
    }
}
