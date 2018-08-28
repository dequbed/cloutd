
use std::net::IpAddr;
use std::collections::HashMap;

use std::process::Command;

use futures::{Future, Async};
use futures_locks::RwLock;

use {Operation, RegistrationReplyMessage, RegistrationCode, Error};

use server::{BoxedFuture, Service};

pub type Peers = HashMap<IpAddr, IpAddr>;

pub struct Registration {
    map: RwLock<Peers>
}

impl Registration {
    pub fn new(map: RwLock<Peers>) -> Self {
        Registration {
            map: map,
        }
    }
}

impl Service for Registration {
    type Request = Operation;
    type Response = Operation;
    type Future = BoxedFuture<Self::Response>;

    fn call(&mut self, request: Self::Request) -> Self::Future {
        use Operation::*;
        if let RegistrationRequest(msg) = request {
            let (hdr, cies) = msg.into_parts();
            let cie = cies[0].clone();
            let rid = hdr.request_id;
            let src_n_a = hdr.src_nbma_addr;
            let src_p_a = hdr.src_proto_addr;

            let f = self.map.write();

            let f = f.and_then(move |mut map| {
                for cie in cies.iter() {
                    let nbma_addr = cie.client_nbma_addr.unwrap_or(hdr.src_nbma_addr);
                    let proto_addr = cie.client_proto_addr.unwrap_or(hdr.src_proto_addr);

                    map.insert(proto_addr, nbma_addr);

                    let out = Command::new("ip").args(&["neigh", "add", "dev", "gre0",
                                                      "nud", "reachable"])
                        .arg("to").arg(format!("{}", proto_addr))
                        .arg("lladdr").arg(format!("{}", nbma_addr)).output().unwrap();
                    let outstr = String::from_utf8(out.stderr).unwrap();
                    trace!("{}", outstr);
                    let out = Command::new("ip").args(&["neigh", "change", "dev", "gre0",
                                                      "nud", "reachable"])
                        .arg("to").arg(format!("{}", proto_addr))
                        .arg("lladdr").arg(format!("{}", nbma_addr)).output().unwrap();

                    let outstr = String::from_utf8(out.stderr).unwrap();
                    trace!("{}", outstr);
                }

                debug!("NBMA Associations are now: {:?}", *map);

                Ok(())
            });

            let f = f.map_err(|_| Error::Invalid);

            Box::new(f.and_then(move |_| Ok(RegistrationReply(RegistrationReplyMessage::new(rid, RegistrationCode::Success, cie, src_n_a, src_p_a, [10,0,0,1].into(), true)))))
        } else {
            panic!("Invalid request was passed to Registration Handler");
        }
    }
}
