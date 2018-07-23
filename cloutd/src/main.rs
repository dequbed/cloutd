#![feature(alloc_system, allocator_api)]

extern crate alloc_system;

use alloc_system::System;

#[global_allocator]
static GLOBAL: System = System;

extern crate tokio;
extern crate tokio_codec;
extern crate bytes;

#[macro_use]
extern crate log;

extern crate mio;
#[macro_use]
extern crate futures;
extern crate libc;
extern crate pnet_sys;

use tokio::reactor::Reactor;

mod nhrp;

fn main() {
    let r = Reactor::new().unwrap();
    let h = r.handle();

    let s = nhrp::NhrpSocket::new_with_handle(&h).unwrap();

}

