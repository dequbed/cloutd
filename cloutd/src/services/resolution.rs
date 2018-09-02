
use std::net::IpAddr;
use std::collections::HashMap;

use std::process::Command;

use futures::{Future, Async};
use futures_locks::RwLock;

use {Operation, ResolutionReplyMessage, ResolutionCode, Error};

use server::{BoxedFuture, Service};

pub type Peers = HashMap<IpAddr, IpAddr>;

pub struct Resolution {
    map: RwLock<Peers>
}

impl Resolution {
    pub fn new(map: RwLock<Peers>) -> Self {
        Resolution {
            map: map,
        }
    }
}

impl Service for Resolution {
    type Request = Operation;
    type Response = Operation;
    type Future = BoxedFuture<Self::Response>;

    fn call(&mut self, request: Self::Request) -> Self::Future {
        use Operation::*;
        if let ResolutionRequest(msg) = request {
            let (hdr, cies) = msg.into_parts();
            let cie = cies[0].clone();
            let rid = hdr.request_id;
            let src_n_a = hdr.src_nbma_addr;
            let src_p_a = hdr.src_proto_addr;
            let dst_n_a = None;
            let dst_p_a = hdr.dst_proto_addr;

            let f = self.map.write();

            let f = f.and_then(move |mut map| {
                match map.get(dst_p_a) {
                    Some(nbma_addr) => {
                        debug!("Found NBMA address {} for requested proto address {}", nbma_addr, dst_p_a);
                        dst_n_a = Some(nbma_addr);
                    },
                    None => {
                        debug!("Could not find NBMA address for requested proto address {}", dst_p_a);
                    }
                }

                Ok(())
            });

            let f = f.map_err(|_| Error::Invalid);

            Box::new(f.and_then(move |_| Ok(ResolutionReply(ResolutionReplyMessage::new(rid, ResolutionCode::Success, cie, src_n_a, src_p_a, [10,0,0,1].into(), true)))))
        } else {
            panic!("Invalid request was passed to Resolution Handler");
        }
    }
}
