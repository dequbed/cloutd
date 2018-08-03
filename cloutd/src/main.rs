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

// 
use std::mem;
use mio::{Poll, Events, Token, Ready, PollOpt};
use pnet_sys as p;
//

/*
 *fn main() {
 *    let r = Reactor::new().unwrap();
 *    let h = r.handle();
 *
 *    let s = nhrp::NhrpSocket::new_with_handle(&h).unwrap();
 *    println!("{:?}", &s);
 *    let f = nhrp::frame::NhrpFramed::new(s, nhrp::codec::NhrpCodec);
 *
 *
 *    let ft = f.for_each(|p| {
 *        println!("{:?}", p);
 *        Ok(())
 *    }).map_err(|e| println!("Error occured: {}", e));
 *
 *    println!("Starting up!");
 *    current_thread::block_on_all(lazy(|| {
 *
 *        current_thread::spawn(ft);
 *
 *        Ok::<_, ()>(())
 *    }));
 *    //tokio::runtime::run(ft);
 *    println!("Done?!");
 *}
 */

const SERVER: Token = Token(0);

fn main() {
    let poll = Poll::new().unwrap();
    let s = nhrp::socket::NhrpRawSocket::new().unwrap();

    poll.register(&s, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for events in events.iter() {
            match events.token() {
                SERVER => {
                    let mut caddr: p::SockAddrStorage = unsafe { mem::zeroed() };

                    let mut buf = Vec::with_capacity(100);

                    match s.recv_from(&mut buf, &mut caddr) {
                        Ok(n) => {
                            let a = p::sockaddr_to_addr(&caddr, mem::size_of::<p::SockAddrStorage>());
                            println!("{:?}", a);
                        },
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                },
                _ => unreachable!()
            }
        }
    }
}
