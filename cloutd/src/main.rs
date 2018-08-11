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

use futures::Future;
use futures::Stream;

use tokio::runtime::Runtime;

mod nhrp;
mod netlink;

fn main() {
    let nhrpsock = nhrp::NhrpSocket::new().unwrap();

    let f: nhrp::NhrpFramed<nhrp::NhrpCodec> = nhrp::NhrpFramed::new(nhrpsock, nhrp::NhrpCodec);

    let future = f.for_each(|frame| {println!("{:?}", frame); Ok(())}).map_err(|e| println!("{:?}", e));

    let mut rt = Runtime::new().unwrap();

    rt.spawn(future);

    rt.shutdown_on_idle().wait().unwrap();
}
