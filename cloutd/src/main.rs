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
extern crate log;

extern crate mio;
#[macro_use]
extern crate futures;
extern crate libc;
extern crate pnet_sys;

#[macro_use]
extern crate nom;

extern crate pnetlink;

use tokio_codec::Decoder;

use pnetlink::ptokio;
use pnetlink::socket::NetlinkProtocol;

use futures::Future;
use futures::Stream;

use tokio::runtime::Runtime;

mod nhrp;
mod netlink;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let nhrpsock = nhrp::NhrpSocket::new().unwrap();
    let nlsock = pnetlink::ptokio::NetlinkSocket::bind(NetlinkProtocol::Route, 4 | 2, rt.reactor()).unwrap();

    let f: nhrp::NhrpFramed<nhrp::NhrpCodec> = nhrp::NhrpFramed::new(nhrpsock, nhrp::NhrpCodec);
    let c = ptokio::NetlinkCodec {};
    let n = c.framed(nlsock);

    let future = f.for_each(|frame| {println!("{:?}", frame); Ok(())}).map_err(|e| println!("{:?}", e));
    let nfuture = n.for_each(|frame| {println!("{:?}", frame); Ok(())}).map_err(|e| println!("{:?}", e));

    rt.spawn(future);
    rt.spawn(nfuture);

    rt.shutdown_on_idle().wait().unwrap();
}
