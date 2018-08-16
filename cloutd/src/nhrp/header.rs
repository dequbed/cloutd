use super::{NhrpBuffer, FIXED_HEADER_LEN};
use {Parseable, Emitable, Result};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum NhrpOp {
    ResolutionRequest,
    ResolutionReply,
    RegistrationRequest,
    RegistrationReply,
    PurgeRequest,
    PurgeReply,
    ErrorIndication,
    Other(u8),
}
impl From<u8> for NhrpOp {
    fn from(value: u8) -> NhrpOp {
        use self::NhrpOp::*;
        match value {
            1 => ResolutionRequest,
            2 => ResolutionReply,
            3 => RegistrationRequest,
            4 => RegistrationReply,
            5 => PurgeRequest,
            6 => PurgeReply,
            7 => ErrorIndication,
            _ => Other(value),
        }
    }
}
impl From<NhrpOp> for u8 {
    fn from(value: NhrpOp) -> u8 {
        use self::NhrpOp::*;
        match value {
            ResolutionRequest => 1,
            ResolutionReply => 2,
            RegistrationRequest => 3,
            RegistrationReply => 4,
            PurgeRequest => 5,
            PurgeReply => 6,
            ErrorIndication => 7,
            Other(value) => value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AddrTL {
    NSAP(u8),
    E164(u8),
}
impl From<u8> for AddrTL {
    fn from(value: u8) -> AddrTL {
        use self::AddrTL::*;
        if value & 64 == 64 {
            E164(value ^ 64) // Unset the 6th bit so the contained u8 is the length
        } else {
            NSAP(value)
        }
    }
}
impl From<AddrTL> for u8 {
    // FIXME: Technically only valid for values <64
    fn from(value: AddrTL) -> u8 {
        use self::AddrTL::*;
        match value {
            E164(v) => v | 64,
            NSAP(v) => v
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct NhrpHeader {
    afn: u16,
    protype: u16,
    prosnap: [u8; 5],
    hopcount: u8,
    length: u16,
    checksum: u16,
    extoffset: u16,
    version: u8,
    optype: NhrpOp,
    shtl: AddrTL,
    sstl: AddrTL,
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NhrpHeader> for NhrpBuffer<&'a T> {
    fn parse(&self) -> Result<NhrpHeader> {
        Ok(NhrpHeader {
            afn: self.afn(),
            protype: self.protype(),
            prosnap: self.prosnap(),
            hopcount: self.hopcount(),
            length: self.length(),
            checksum: self.checksum(),
            extoffset: self.extoffset(),
            version: self.version(),
            optype: self.optype(),
            shtl: self.shtl(),
            sstl: self.sstl(),
        })
    }
}

impl Emitable for NhrpHeader {
    fn buffer_len(&self) -> usize {
        FIXED_HEADER_LEN
    }

    fn emit(&self, buffer: &mut [u8]) {
        let mut buffer = NhrpBuffer::new(buffer);
        buffer.set_afn(self.afn);
        buffer.set_protype(self.protype);
        buffer.set_prosnap(self.prosnap);
        buffer.set_hopcount(self.hopcount);
        buffer.set_length(self.length);
        buffer.set_checksum(self.checksum);
        buffer.set_extoffset(self.extoffset);
        buffer.set_version(self.version);
        buffer.set_optype(self.optype);
        buffer.set_shtl(self.shtl);
        buffer.set_sstl(self.sstl);
    }
}
