use {Parseable, Emitable, Result, Error};

use super::buffer::{ExtensionBuffer, EXTENSION_HEADER_LEN};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Extension {
    EndOfExtensions,

    Other {
        etype: ExtensionType,
        compulsory: bool,
        data: Vec<u8>
    }
}

impl Extension {
    pub fn length(&self) -> usize {
        use self::Extension::*;
        match self {
            EndOfExtensions => 0,
            Other { data, .. } => data.len(),
        }
    }

    pub fn compulsory(&self) -> bool {
        use self::Extension::*;
        match *self {
            EndOfExtensions => true,
            Other { compulsory, .. } => compulsory
        }
    }

    pub fn etype(&self) -> ExtensionType {
        use self::Extension::*;
        use self::ExtensionType::*;
        match *self {
            EndOfExtensions => NHRP(0),
            Other { etype, .. } => etype
        }
    }

    pub fn data(&self) -> &[u8] {
        use self::Extension::*;
        match self {
            EndOfExtensions => &[],
            Other { data, .. } => &data[..]
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExtensionType {
    NHRP(u16),
    ATM(u16),
    IETF(u16),
    Experimental(u16),
}
pub const EndOfExtensionsType: ExtensionType = ExtensionType::NHRP(0);

impl From<u16> for ExtensionType {
    fn from(value: u16) -> ExtensionType {
        use self::ExtensionType::*;
        match value {
            0x0000...0x0FFF => NHRP(value),
            0x1000...0x11FF => ATM(value),
            0x1200...0x37FF => IETF(value),
            0x3800...0x3FFF => Experimental(value),
            _ => panic!("Value of {} is invalid for ExtensionType, maximum is 0x3FFF.", value),
        }
    }
}
impl From<ExtensionType> for u16 {
    fn from(value: ExtensionType) -> u16 {
        use self::ExtensionType::*;
        match value {
            NHRP(value) => value,
            ATM(value) => value,
            IETF(value) => value,
            Experimental(value) => value,
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<Extension> for ExtensionBuffer<&'a T> {
    fn parse(&self) -> Result<Extension> {
        use self::Extension::*;
        use self::ExtensionType::*;
        Ok(match self.extensiontype() {
            NHRP(0) => EndOfExtensions,
            etype => Other {
                etype: etype,
                compulsory: self.compulsory(),
                data: self.payload().to_vec(),
            }
        })
    }
}

impl Emitable for Extension {
    fn buffer_len(&self) -> usize {
        EXTENSION_HEADER_LEN + self.length()
    }

    fn emit(&self, buffer: &mut [u8]) {
        let mut buffer = ExtensionBuffer::new(buffer);
        buffer.set_extensiontype(self.etype());
        buffer.set_compulsory(self.compulsory());
        buffer.set_length(self.length() as u16);
        buffer.payload_mut().copy_from_slice(self.data());
    }
}
