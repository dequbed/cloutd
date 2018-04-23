#![feature(alloc_system, global_allocator, allocator_api)]

extern crate alloc_system;

use alloc_system::System;

#[global_allocator]
static GLOBAL: System = System;

extern crate pnetlink;

use std::collections::HashMap;

use std::io::Read;

use pnetlink::socket::{NetlinkSocket, NetlinkProtocol};
use pnetlink::packet::netlink::{NetlinkConnection, NetlinkPacket, NetlinkReader};
use pnetlink::packet::route::link::Links;
use pnetlink::packet::route::neighbour::{Neighbours, NeighbourState};

fn main() {
    let mut s = NetlinkSocket::bind(NetlinkProtocol::Route, 4).unwrap();

    let mut reader = NetlinkReader::new(&mut s);
    while let Ok(Some(pkt)) = reader.read_netlink() {
        println!("{:?}, {:?}: {:?}", pkt.get_seq(), pkt.get_pid(), pkt.get_kind());
    }
}
