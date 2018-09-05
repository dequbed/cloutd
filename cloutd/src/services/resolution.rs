
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
            let dst_p_a = hdr.dst_proto_addr;

            let mut dst_n_a = None;
            let mut code = ResolutionCode::NoBindingExists;

            let f = self.map.write();

            let f = f.and_then(move |mut map| {
                match map.get(dst_p_a) {
                    Some(nbma_addr) => {
                        debug!("Found NBMA address {} for requested proto address {}", nbma_addr, dst_p_a);
                        dst_n_a = Some(nbma_addr);
                        code = ResolutionCode::Success;
                    },
                    None => {
                        debug!("Could not find NBMA address for requested proto address {}", dst_p_a);
                    }
                }

                Ok(())
            });

            let f = f.map_err(|_| Error::Invalid);

            Box::new(f.and_then(move |_| Ok(ResolutionReply(ResolutionReplyMessage::new(rid, code, src_n_a, src_p_a, dst_n_a, dst_p_a, ((hdr.flags >> 15) as bool), true, true, (((hdr.flags >> 11) & 1) as bool), false, 60, 255)))))
        } else {
            panic!("Invalid request was passed to Resolution Handler");
        }
    }
}
