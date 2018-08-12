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
extern crate libc;
extern crate pnet_sys;

#[macro_use]
extern crate nom;

extern crate pnetlink;

use slog::Drain;

use tokio_codec::Decoder;

use pnetlink::ptokio;
use pnetlink::socket::NetlinkProtocol;

use futures::Future;
use futures::Stream;

use tokio::runtime::Runtime;

mod nhrp;
mod netlink;

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
    trace!("Opening Netlink socket...");
    let nlsock = match pnetlink::ptokio::NetlinkSocket::bind(NetlinkProtocol::Route, 4 | 2, rt.reactor()) {
        Ok(s) => {
            trace!("Opened Netlink socket.");
            s
        },
        Err(e) => {
            error!("Failed to open Netlink socket"; "error" => %e);
            return;
        }
    };

    let f: nhrp::NhrpFramed<nhrp::NhrpCodec> = nhrp::NhrpFramed::new(nhrpsock, nhrp::NhrpCodec);
    let c = ptokio::NetlinkCodec {};
    let n = c.framed(nlsock);

    let future = f.for_each(|frame| {trace!("{:?}", frame); Ok(())}).map_err(|e| error!("{:?}", e));
    let nfuture = n.for_each(|frame| {trace!("{:?}", frame); Ok(())}).map_err(|e| error!("{:?}", e));

    trace!("Spawning futures...");
    rt.spawn(future);
    rt.spawn(nfuture);
    trace!("Spawned futures.");

    rt.shutdown_on_idle().wait().unwrap();
}

fn main() {
    mainw()
}
