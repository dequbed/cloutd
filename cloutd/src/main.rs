#![feature(alloc_system, allocator_api)]

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

use tokio::reactor::Reactor;
use tokio::executor::current_thread::{self,CurrentThread};

use futures::Stream;
use futures::Future;
use futures::future::lazy;


mod nhrp;

fn main() {
    let r = Reactor::new().unwrap();
    let h = r.handle();

    let s = nhrp::NhrpSocket::new_with_handle(&h).unwrap();
    println!("{:?}", &s);
    let f = nhrp::frame::NhrpFramed::new(s, nhrp::codec::NhrpCodec);


    let ft = f.for_each(|p| {
        println!("{:?}", p);
        Ok(())
    }).map_err(|e| println!("Error occured: {}", e));

    println!("Starting up!");
    current_thread::block_on_all(lazy(|| {

        current_thread::spawn(ft);

        Ok::<_, ()>(())
    }));
    //tokio::runtime::run(ft);
    println!("Done?!");
}

