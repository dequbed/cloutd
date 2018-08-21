use super::{NhrpBuffer, FIXED_HEADER_LEN};
use {Parseable, Emitable, Result, Error};
use byteorder::{ByteOrder, BigEndian};

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum ProtocolClass {
    NLPID(u8),
    Future(u16),
    ATM(u8),
    Private(u8),
    Ethertype(u16),
}
impl From<u16> for ProtocolClass {
    fn from(value: u16) -> ProtocolClass {
        use ProtocolClass::*;

        let mut bytes = [0; 2];
        BigEndian::write_u16(&mut bytes, value);
        match bytes[0] {
            0x00 => NLPID(bytes[1]),
            0x01 | 0x02 | 0x03 => Future(BigEndian::read_u16(&bytes)),
            0x04 => ATM(bytes[1]),
            0x05 => Private(bytes[1]),
            _ => Ethertype(BigEndian::read_u16(&bytes)),
        }
    }
}
// # FIXME This should be able to be statically typechecked, even without dependent types!
impl From<ProtocolClass> for u16 {
    fn from(value: ProtocolClass) -> u16 {
        use ProtocolClass::*;

        let bytes: [u8; 2];
        match value {
            NLPID(v) => bytes = [0x00, v],
            Future(v) => {
                assert!(0x0100 <= v && v <= 0x03FF, "`Future` ProtocolClass has invalid value: {}", v);
                return v;
            },
            ATM(v) => bytes = [0x04, v],
            Private(v) => bytes = [0x05, v],
            Ethertype(v) => {
                assert!(v >= 0x0600, "`Ethertype` ProtocolClass is not a valid Ethertype: {}", v);
                return v;
            }
        }

        BigEndian::read_u16(&bytes)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct ProtocolType {
    pub protype: ProtocolClass,
    pub prosnap: [u8; 5],
}
impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<ProtocolType> for &'a T {
    fn parse(&self) -> Result<ProtocolType> {
        if self.as_ref().len() < 7 {
            return Err(Error::Truncated);
        }

        let buf = &self.as_ref();
        let protype = BigEndian::read_u16(&buf[0..2]);
        let mut prosnap: [u8; 5] = [0; 5];
        prosnap.copy_from_slice(&buf[2..7]);

        Ok(ProtocolType {
            protype: protype.into(),
            prosnap: prosnap
        })
    }
}
impl Emitable for ProtocolType {
    fn buffer_len(&self) -> usize {
        7
    }

    fn emit(&self, buffer: &mut [u8]) {
        BigEndian::write_u16(&mut buffer[0..2], self.protype.into());
        let buffer = &mut buffer[2..7];
        buffer.copy_from_slice(&self.prosnap);
    }
}

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
    protocol_type: ProtocolType,
    hopcount: u8,
    optype: NhrpOp,
}

impl FixedHeader {
    pub fn new(afn: u16, protocol_type: ProtocolType, hopcount: u8, optype: NhrpOp) -> FixedHeader {
        FixedHeader {
            afn: afn,
            protocol_type: protocol_type,
            hopcount: hopcount,
            optype: optype,
        }
    }

    // FIXME: Use these to do lazy parsing.
    pub fn afn(&self) -> u16 {
        self.afn
    }
    pub fn protocol_type(&self) -> ProtocolType {
        self.protocol_type
    }
    pub fn hopcount(&self) -> u8 {
        self.hopcount
    }
    pub fn optype(&self) -> NhrpOp {
        self.optype
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<FixedHeader> for NhrpBuffer<&'a T> {
    fn parse(&self) -> Result<FixedHeader> {
        Ok(FixedHeader {
            afn: self.afn(),
            protocol_type: self.protocol_type(),
            hopcount: self.hopcount(),
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
        buffer.set_protocol_type(self.protocol_type);
        buffer.set_hopcount(self.hopcount);
        buffer.set_optype(self.optype);
    }
}
