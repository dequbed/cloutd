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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FixedHeader {
    afn: u16,
    protype: u16,
    prosnap: [u8; 5],
    hopcount: u8,
    length: u16,
    checksum: u16,
    extoffset: u16,
    version: u8,
    optype: NhrpOp,
}

impl FixedHeader {
    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn optype(&self) -> NhrpOp {
        self.optype
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<FixedHeader> for NhrpBuffer<&'a T> {
    fn parse(&self) -> Result<FixedHeader> {
        Ok(FixedHeader {
            afn: self.afn(),
            protype: self.protype(),
            prosnap: self.prosnap(),
            hopcount: self.hopcount(),
            length: self.length(),
            checksum: self.checksum(),
            extoffset: self.extoffset(),
            version: self.version(),
            optype: self.optype(),
        })
    }
}

impl Emitable for FixedHeader {
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
    }
}
