#![allow(dead_code)]
use super::header::*;
use super::{Field, Index, Rest, Result, Error};

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
        NhrpBuffer { buffer }
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
        u16::from_be_bytes(data[AFN].try_into().unwrap())
    }

    pub fn protype(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from_be_bytes(data[PROTYPE].try_into().unwrap())
    }

    pub fn prosnap(&self) -> [u8; 5] {
        let data = self.buffer.as_ref();
        let mut s = [0; 5];
        s.copy_from_slice(data[SNAP].try_into().unwrap());
        s
    }

    pub fn protocol_type(&self) -> ProtocolType {
        let protype = self.protype().into();
        ProtocolType {
            protype: protype,
            prosnap: self.prosnap(),
        }
    }

    pub fn hopcount(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[HOPCOUNT]
    }

    pub fn length(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from_be_bytes(data[PKTSIZE].try_into().unwrap())
    }

    pub fn checksum(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from_be_bytes(data[CHECKSUM].try_into().unwrap())
    }

    pub fn extoffset(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from_be_bytes(data[EXTOFFSET].try_into().unwrap())
    }

    pub fn version(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[VERSION]
    }

    pub fn optype(&self) -> NhrpOp {
        let data = self.buffer.as_ref();
        data[OPTYPE].into()
    }

    pub fn calculate_checksum(&self) -> u16 {
        let len = self.length() as usize;
        let mut uints = vec![0; len/2].into_boxed_slice();
        for i in 0..(len/2) {
            let start = 2 * i;
            let end = start+2;
            let array = self.buffer.as_ref()[start..end].try_into().unwrap();
            uints[i] = u16::from_be_bytes(array);
        }
        let mut checksum = 0u32;

        checksum = uints.iter().fold(checksum, |a,x| a + *x as u32);

        if len & 1 == 1 {
            checksum += self.buffer.as_ref()[len-1] as u32
        }

        // This one goes wrong somehow
        while checksum & 0xffff0000 != 0 {
            checksum = (checksum & 0xffff) + (checksum >> 16);
        }

        !checksum as u16
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
        data[AFN].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_protype(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        data[PROTYPE].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_prosnap(&mut self, value: [u8; 5]) {
        let data = self.buffer.as_mut();
        data[SNAP].copy_from_slice(&value);
    }

    pub fn set_protocol_type(&mut self, value: ProtocolType) {
        self.set_protype(value.protype.into());
        self.set_prosnap(value.prosnap);
    }

    pub fn set_hopcount(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[HOPCOUNT] = value
    }

    pub fn set_length(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        data[PKTSIZE].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_checksum(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        data[CHECKSUM].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_extoffset(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        data[EXTOFFSET].copy_from_slice(&value.to_be_bytes());
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
