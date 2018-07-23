use tokio_codec::{Decoder, Encoder};

pub struct NhrpPacket {
    // Fixed Header:
    /// Address Family Number
    afn: u16,
    /// Protocol Type
    prototype: u16,
    protosnap: u32,
    protosnap2: u8,
    /// Hop Count
    hopcnt: u8,
    /// Packet Size
    pktsz: u16,
    /// Checksum
    chksum: u16,
    /// Extension Offset
    extoff: u16,
    /// Operation Version
    opversion: u8,
    /// Operation Type
    optype: u8,
    /// type/length of NBMA address
    shtl: u8,
    /// type/length of NBMA sub-address
    sstl: u8,
}

pub struct NhrpCodec;


impl Decoder for NhrpCodec {
    type Item = NhrpPacket;
}
