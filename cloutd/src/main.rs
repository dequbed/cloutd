/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#![feature(alloc_system, allocator_api)]
#![feature(int_to_from_bytes)]

extern crate alloc_system;

use alloc_system::System;

#[global_allocator]
static GLOBAL: System = System;


extern crate tokio;
extern crate tokio_codec;
extern crate tokio_current_thread;
extern crate bytes;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;

extern crate mio;
#[macro_use]
extern crate futures;
extern crate futures_locks;
extern crate libc;

extern crate rtnetlink;
extern crate netlink_socket;

extern crate core;
extern crate byteorder;
extern crate iovec;

use slog::Drain;

use futures::Future;
use futures::Stream;
use futures::Sink;

use tokio::runtime::Runtime;

use byteorder::{ByteOrder, NativeEndian};

use netlink_socket::{TokioSocket, SocketAddr, Protocol};
use rtnetlink::{NetlinkFramed, NetlinkCodec, NetlinkMessage};

use std::net::IpAddr;
use std::collections::HashMap;

use std::process::Command;

mod netlink;

mod error;
use error::*;

mod traits;
use traits::*;

mod server;

mod services;

fn mainw() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let _logguard = slog_scope::set_global_logger(log);

    info!("Starting up.");

    trace!("Constructing eventloop...");
    let mut rt = match Runtime::new() {
        Ok(r) => {
            trace!("Constructed eventloop.");
            r
        },
        Err(e) => {
            error!("Failed to construct eventloop"; "error" => %e);
            return;
        }
    };

    trace!("Opening Netlink socket...");
    let nlsock = {
        let mut socket = match TokioSocket::new(Protocol::Route) {
            Ok(s) => {
                trace!("Created Netlink socket.");
                s
            },
            Err(e) => {
                error!("Failed to create Netlink socket"; "error" => %e);
                return;
            },
        };
        let _port = match socket.bind_auto() {
            Ok(s) => {
                let port = s.port_number();
                trace!("Bound to port {}.", port);
                port
            },
            Err(e) => {
                error!("Failed to bind socket"; "error" => %e);
                return;
            },
        };

        socket
    };

    // nlstream will produce a Stream of RTNL messages from the kernel to us, including Neighbour
    // table lookups. We need to filter/split those messages and act on each of the messages of the
    // filtered/split stream.
    let (nlsink,_nlstream) = NetlinkFramed::new(nlsock, NetlinkCodec::<NetlinkMessage>::new()).split();

    // Configure the Neighbour table of $INTERFACE to use application probes only:
    let ifcfg: NetlinkMessage = pkt(6);
    let kernel = SocketAddr::new(0,0);
    let sendfut = nlsink.send((ifcfg, kernel))
                        .and_then(|sink| sink.flush())
                        .map_err(|e| error!("{:?}", e));

    // We need the interface configured to do any NHRP processing as NHC (FIXME: Not so much on NHS)
    trace!("Configuring Network interface...");
    let _nlsink = rt.block_on(sendfut);
    trace!("Configured Network interface.");

    trace!("Opening NHRP socket...");
    let nhrpsock = match nhrp::NhrpSocket::new() {
        Ok(s) => {
            trace!("Opened NHRP socket.");
            s
        },
        Err(e) => {
            error!("Failed to open NHRP socket"; "error" => %e);
            return;
        }
    };
    //1. nhrpstream <- listen
    let t = nhrp::NhrpFramed::new(nhrpsock, nhrp::NhrpCodec::<NhrpMessage>::new());

    let server = server::ServerProto::new(t);

    let server = server.map_err(|e| error!("{:?}", e));

    trace!("Spawning futures...");
    rt.spawn(server);
    trace!("Spawned futures.");

    rt.shutdown_on_idle().wait().unwrap();
}

fn main() {
    mainw()
}

fn pkt(ifid: u32) -> NetlinkMessage {
    use rtnetlink::{NetlinkBuffer, Parseable};
    //
    //
    // FIXME: If it's stupid but it works it's still stupid and you're lucky.
    //
    let mut buf: [u8; 72] = [
        0x48, 0x00, 0x00, 0x00, // Length = 72
        0x43, 0x00, // Message type = Set Neighbour Table
        0x01, 0x01, // Flags = REQUEST | REPLACE
        0x26, 0xf4, 0x73, 0x5b, // Sequence
        0x00, 0x00, 0x00, 0x00, // Port ID
        // Payload:
        0x02, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x01, 0x00, 0x61, 0x72, 0x70, 0x5f, 0x63, 0x61, 0x63,
        0x68, 0x65, 0x00, // "arp-cache"
        0x00, 0x00, // Padding
        0x24, 0x00, 0x06, 0x00, // Flags, 36 bytes of them.
            0x08, 0x00, 0x01, 0x00, 0x06, 0x00, 0x00, 0x00, // IFID = 6
            0x08, 0x00, 0x09, 0x00, 0x01, 0x00, 0x00, 0x00, // APP_PROBES = 1
            0x08, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, // MCAST_PROBES = 0
            0x08, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, // UCAST_PROBES = 0
    ];
    // Write proper ifid
    NativeEndian::write_u32(&mut buf[44..48], ifid);
    NetlinkBuffer::new_checked(&&buf[..]).unwrap().parse().unwrap()
}
