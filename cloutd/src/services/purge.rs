
use std::process::Command;

use futures::{Future, Async};
use futures_locks::RwLock;

use {Operation, RegistrationReplyMessage, RegistrationCode, Error};

use super::Peers;

use server::{BoxedFuture, Service};

pub struct Purge {
    map: RwLock<Peers>
}

impl Purge {
    pub fn new(map: RwLock<Peers>) -> Self {
        Purge {
            map: map,
        }
    }
}

/*
 *-            PurgeRequest(msg) => {
 *-
 *-                for cie in msg.cie().iter() {
 *-                    let proto_addr = cie.client_proto_addr.unwrap_or(msg.header().src_proto_addr);
 *-                    server.remove(&proto_addr);
 *-
 *-                    let out = Command::new("ip").args(&["neigh", "remove", "dev", "gre0"])
 *-                        .arg("to").arg(format!("{}", proto_addr)).output().unwrap();
 *-                    let outstr = String::from_utf8(out.stderr).unwrap();
 *-                    trace!("{}", outstr);
 *-                }
 *-
 *-                trace!("NBMA associations are now: {:?}", server);
 *-
 *-                let header = FixedHeader::new(header.afn(), header.protocol_type(),
 *-                    header.hopcount(), NhrpOp::PurgeReply);
 *-                let response = NhrpMessage::new(header, PurgeReply(msg), Vec::new());
 *-                Some(Ok((response, sourceaddr)))
 *-            },
 */

impl Service for Purge {
    type Request = Operation;
    type Response = Operation;
    type Future = BoxedFuture<Self::Response>;

    fn call(&mut self, request: Self::Request) -> Self::Future {
        use Operation::*;
        if let PurgeRequest(msg) = request {
            let f = self.map.write();

            let f = f.and_then(move |mut map| {
                for cie in msg.cie().iter() {
                    let proto_addr = cie.client_proto_addr.unwrap_or(msg.header().src_proto_addr);

                    map.remove(&proto_addr);

                    let out = Command::new("ip").args(&["neigh", "devl", "dev", "gre0"])
                        .arg("to").arg(format!("{}", proto_addr)).output().unwrap();
                    let outstr = String::from_utf8(out.stderr).unwrap();
                    trace!("{}", outstr);
                }

                debug!("NBMA Associations are now: {:?}", *map);

                Ok(PurgeReply(msg))
            });

            let f = f.map_err(|_| Error::Invalid);

            Box::new(f)
        } else {
            panic!("Invalid request was passed to Registration Handler");
        }
    }
}
