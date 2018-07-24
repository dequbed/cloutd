use nom::{be_u8, be_u16, be_u32};

#[derive(Debug)]
pub enum AddrTL {
    NSAP(u8),
    E164(u8),
}
impl AddrTL {
    pub fn val(&self) -> u8 {
        match *self {
            AddrTL::NSAP(v) => v,
            AddrTL::E164(v) => v
        }
    }
}

#[derive(Debug)]
pub struct NhrpPacket {
    header: FixedHeader,
    mandatory: MandatoryPart,
    extensions: Vec<Extension>,
}

#[derive(Debug)]
pub struct FixedHeader {
    // Fixed Header:
    /// Address Family Number
    afn: u16,
    /// Protocol Type
    prototype: u16,
    protosnap: [u8; 5],
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
    shtl: AddrTL,
    /// type/length of NBMA sub-address
    sstl: AddrTL,
}

#[derive(Debug)]
pub enum MandatoryPart {
    ReqRep {
        hdr: ReqMandatoryPart,
        cie: Vec<ClientInformationEntry>,
    },
    Error(ErrMandatoryPart),
}

#[derive(Debug)]
pub struct ReqMandatoryPart {
    /// Lenght (in octets) of Source Protocol Address
    src_proto_addr_len: u8,

    /// Lenght (in octets) of Destination Protocol Address
    dst_proto_addr_len: u8,

    /// Flags specific to a message type
    flags: u16,

    /// A value which, when coupled with the address of the source, provides a unique identifier
    /// for the information contained in a "request" packet.
    /// This value is copied directly from an "request" packet into the associated "reply".  When
    /// a sender of a "request" receives "reply", it will compare the Request ID and source
    /// address information in the received "reply" against that found in its outstanding
    /// "request" list.  When a match is found then the "request" is considered to be
    /// acknowledged.
    /// The value is taken from a 32 bit counter that is incremented each time a new "request" is
    /// transmitted.  The same value MUST be used when resending a "request", i.e., when a "reply"
    /// has not been received for a "request" and a retry is sent after an appropriate interval.
    request_id: u32,

    /// The Source NBMA address field is the address of the source station which is sending the
    /// "request". If the field's length as specified in ar$shtl is 0 then no storage is allocated
    /// for this address at all.
    // NOTE: std::Vec::with_capacity(0) will *NOT* allocate any memory.
    src_nbma_addr: Vec<u8>,

    /// The Source NBMA address field is the subaddress of the source station which is sending the
    /// "request". If the field's length as specified in ar$sstl is 0 then no storage is allocated
    /// for this address at all.
    src_nbma_saddr: Vec<u8>,

    /// This is the protocol address of the station which is sending the "request".  This is also
    /// the protocol address of the station toward which a "reply" packet is sent.
    src_proto_addr: Vec<u8>,

    /// This is the protocol address of the station toward which a "request" packet is sent.
    dst_proto_addr: Vec<u8>,
}

#[derive(Debug)]
pub struct ErrMandatoryPart {
    /// Lenght (in octets) of Source Protocol Address
    src_proto_addr_len: u8,

    /// Lenght (in octets) of Destination Protocol Address
    dst_proto_addr_len: u8,

    error_code: u16,

    error_offset: u16,

    /// The Source NBMA address field is the address of the source station which is sending the
    /// "request". If the field's length as specified in ar$shtl is 0 then no storage is allocated
    /// for this address at all.
    // NOTE: std::Vec::with_capacity(0) will *NOT* allocate any memory.
    src_nbma_addr: Vec<u8>,

    /// The Source NBMA address field is the subaddress of the source station which is sending the
    /// "request". If the field's length as specified in ar$sstl is 0 then no storage is allocated
    /// for this address at all.
    src_nbma_saddr: Vec<u8>,

    /// This is the protocol address of the station which is sending the "request".  This is also
    /// the protocol address of the station toward which a "reply" packet is sent.
    src_proto_addr: Vec<u8>,

    /// This is the protocol address of the station toward which a "request" packet is sent.
    dst_proto_addr: Vec<u8>,
}

#[derive(Debug)]
pub struct ClientInformationEntry {
    /// This field is message specific.  See the relevant message sections below.  In general, this
    /// field is a NAK code; i.e., when the field is 0 in a reply then the packet is acknowledging
    /// a request and if it contains any other value the packet contains a negative acknowledgment.
    code: u8,

    /// This field is message specific.  See the relevant message sections below.  In general,
    /// however, this fields is used to indicate that the information carried in an NHRP message
    /// pertains to an equivalence class of internetwork layer addresses rather than just a single
    /// internetwork layer address specified. All internetwork layer addresses that match the first
    /// "Prefix Length" bit positions for the specific internetwork layer address are included in
    /// the equivalence class.  If this field is set to 0x00 then this field MUST be ignored and no
    /// equivalence information is assumed (note that 0x00 is thus equivalent to 0xFF).
    prefixlen: u8,

    /// This field gives the maximum transmission unit for the relevant client station.  If this
    /// value is 0 then either the default MTU is used or the MTU negotiated via signaling is used
    /// if such negotiation is possible for the given NBMA.
    mtu: u16,

    /// The Holding Time field specifies the number of seconds for which the Next Hop NBMA
    /// information specified in the CIE is considered to be valid.  Cached information SHALL be
    /// discarded when the holding time expires.  This field must be set to 0 on a NAK.
    holding_time: u16,

    /// Type & length of next hop NBMA address specified in the CIE.  This field is interpreted in
    /// the context of the 'address family number' indicated by ar$afn
    client_addr_typelen: AddrTL,

    /// Type & length of next hop NBMA subaddress specified in the CIE.  This field is interpreted
    /// in the context of the 'address family number' indicated by ar$afn.  When an NBMA technology
    /// has no concept of a subaddress, the subaddress is always null with a length of 0.  When the
    /// address length is specified as 0 no storage is allocated for the address.
    client_saddr_typelen: AddrTL,

    /// This field holds the length in octets of the Client Protocol Address specified in the CIE.
    client_proto_len: u8,

    /// This field specifies the preference for use of the specific CIE relative to other CIEs.
    /// Higher values indicate higher preference.  Action taken when multiple CIEs have equal or
    /// highest preference value is a local matter.
    preferences: u8,

    /// This is the client's NBMA address.
    client_nbma_addr: Vec<u8>,

    /// This is the client's NBMA subaddress.
    client_nbma_saddr: Vec<u8>,

    /// This is the client's internetworking layer address specified.
    client_proto_addr: Vec<u8>,
}

#[derive(Debug)]
pub struct Extension {
    //FIXME
}

named!(pub parse<NhrpPacket>, do_parse!(
    h: fixed >>
    m: apply!(mandatory, h.optype, h.shtl.val(), h.sstl.val()) >>
    e: many0!(apply!(extensions, h.pktsz, h.extoff)) >>
    (NhrpPacket { header: h, mandatory: m, extensions: e })
));

named!(addrtl<AddrTL>, bits!(do_parse!(
    tag_bits!(u8, 1, 0) >>
    r: switch!( take_bits!(u8, 1),
        0 => do_parse!(
            len: take_bits!(u8, 6) >>
            (AddrTL::E164(len)))
        |
        1 => do_parse!(
            len: take_bits!(u8, 6) >>
            (AddrTL::NSAP(len)))
    ) >>
    (r)
)));

named!(fixed<FixedHeader>, do_parse!(
    afn: be_u16 >>
    prototype: be_u16 >>
    p1: be_u8 >> p2: be_u8 >> p3: be_u8 >> p4: be_u8 >> p5: be_u8 >>
    hopcnt: be_u8 >>
    pktsz: be_u16 >>
    chksum: be_u16 >>
    extoff: be_u16 >>
    opversion: be_u8 >>
    optype: be_u8 >>
    shtl: addrtl >>
    sstl: addrtl >>

    (FixedHeader {
        afn: afn,
        prototype: prototype,
        protosnap: [p1,p2,p3,p4,p5],
        hopcnt: hopcnt,
        pktsz: pktsz,
        chksum: chksum,
        extoff: extoff,
        opversion: opversion,
        optype: optype,
        shtl: shtl,
        sstl: sstl
    })
));


named_args!(mandatory(optype: u8, snal: u8, snsl: u8)<MandatoryPart>, switch!(value!(optype),
    7 => apply!(errmandatory, snal, snsl)
    |
    _ => apply!(reqmandatory, snal, snsl)
));


named_args!(reqmandatory(src_nbma_addr_len: u8, src_nbma_saddr_len: u8)<MandatoryPart>,
    do_parse!(
        hdr: do_parse!(
            src_proto_addr_len: be_u8 >>
            dst_proto_addr_len: be_u8 >>
            flags: be_u16 >>
            request_id: be_u32 >>
            src_nbma_addr: take!(src_nbma_addr_len) >>
            src_nbma_saddr: take!(src_nbma_saddr_len) >>
            src_proto_addr: take!(src_proto_addr_len) >>
            dst_proto_addr: take!(dst_proto_addr_len) >>

            (ReqMandatoryPart {
                src_proto_addr_len: src_proto_addr_len,
                dst_proto_addr_len: dst_proto_addr_len,
                flags: flags,
                request_id: request_id,
                // FIXME: DON'T COPY FFS
                src_nbma_addr: src_nbma_addr.to_vec(),
                src_nbma_saddr: src_nbma_saddr.to_vec(),
                src_proto_addr: src_proto_addr.to_vec(),
                dst_proto_addr: dst_proto_addr.to_vec(),
            })
        ) >>
        cie: many0!(cie) >>
        (MandatoryPart::ReqRep { hdr, cie })
    )
);

named_args!(errmandatory(src_nbma_addr_len: u8, src_nbma_saddr_len: u8)<MandatoryPart>,
    do_parse!(
        src_proto_addr_len: be_u8 >>
        dst_proto_addr_len: be_u8 >>
        error_code: be_u16 >>
        error_offset: be_u16 >>
        src_nbma_addr: take!(src_nbma_addr_len) >>
        src_nbma_saddr: take!(src_nbma_saddr_len) >>
        src_proto_addr: take!(src_proto_addr_len) >>
        dst_proto_addr: take!(dst_proto_addr_len) >>

        (MandatoryPart::Error(ErrMandatoryPart {
            src_proto_addr_len: src_proto_addr_len,
            dst_proto_addr_len: dst_proto_addr_len,
            error_code: error_code,
            error_offset: error_offset,
            src_nbma_addr: src_nbma_addr.to_vec(),
            src_nbma_saddr: src_nbma_saddr.to_vec(),
            src_proto_addr: src_proto_addr.to_vec(),
            dst_proto_addr: dst_proto_addr.to_vec(),
        }))
    )
);

named!(cie<ClientInformationEntry>, do_parse!(
    code: be_u8 >>
    prefixlen: be_u8 >>
    mtu: be_u16 >>
    holding_time: be_u16 >>
    client_addr_typelen: addrtl >>
    client_saddr_typelen: addrtl >>
    client_proto_len: be_u8 >>
    preferences: be_u8 >>
    client_nbma_addr: take!(client_addr_typelen.val()) >>
    client_nbma_saddr: take!(client_saddr_typelen.val()) >>
    client_proto_addr: take!(client_proto_len) >>

    (ClientInformationEntry {
        code: code,
        prefixlen: prefixlen,
        mtu: mtu,
        holding_time: holding_time,
        client_addr_typelen: client_addr_typelen,
        client_saddr_typelen: client_saddr_typelen,
        client_proto_len: client_proto_len,
        preferences: preferences,
        client_nbma_addr: client_nbma_addr.to_vec(),
        client_nbma_saddr: client_nbma_saddr.to_vec(),
        client_proto_addr: client_proto_addr.to_vec(),
    })
));

named_args!(extensions(_pktsz: u16, _extof: u16)<Extension>, value!(Extension {}));
