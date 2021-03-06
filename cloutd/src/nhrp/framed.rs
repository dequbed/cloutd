/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io;
use std::net::{IpAddr, Ipv4Addr/*, Ipv6Addr*/};

use futures::{Async, Poll, Stream, Sink, StartSend, AsyncSink};

use super::NhrpSocket;

use tokio_codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};


#[must_use = "sinks do nothing unless polled"]
#[derive(Debug)]
pub struct NhrpFramed<C> {
    socket: NhrpSocket,
    codec: C,
    rd: BytesMut,
    wr: BytesMut,
    out_addr: IpAddr,
    flushed: bool,
}

impl<C: Decoder> Stream for NhrpFramed<C> {
    type Item = (C::Item, IpAddr);
    type Error = C::Error;

    fn poll(&mut self) -> Poll<Option<(Self::Item)>, Self::Error> {
        self.rd.reserve(INITIAL_RD_CAPACITY);

        let (_n, addr) = {
            match unsafe { self.socket.poll_recv_from(self.rd.bytes_mut()) } {
                Ok(Async::Ready((n, addr))) => {
                    unsafe { self.rd.advance_mut(n) };
                    (n, addr)
                },
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) => return Err(e.into())
            }
        };

        let frame_res = self.codec.decode(&mut self.rd);
        self.rd.clear();
        let frame = frame_res?;
        let result = frame.map(|frame| (frame, addr)); // frame -> (frame, addr)
        Ok(Async::Ready(result))
    }
}

impl<C: Encoder> Sink for NhrpFramed<C> {
    type SinkItem = (C::Item, IpAddr);
    type SinkError = C::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if !self.flushed {
            match try!(self.poll_complete()) {
                Async::Ready(()) => {},
                Async::NotReady => return Ok(AsyncSink::NotReady(item)),
            }
        }

        let (frame, out_addr) = item;
        self.codec.encode(frame, &mut self.wr)?;
        self.out_addr = out_addr;
        self.flushed = false;
        debug!("Frame encoded"; "length" => self.wr.len());

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), C::Error> {
        if self.flushed {
            return Ok(Async::Ready(()));
        }

        trace!("Flushing frame"; "length" => self.wr.len());
        let n = try_ready!(self.socket.poll_send_to(&self.wr, &self.out_addr));
        trace!("Written {} bytes", n);

        let wrote_all = n == self.wr.len();
        self.wr.clear();
        self.flushed = true;

        if wrote_all {
            Ok(Async::Ready(()))
        } else {
            Err(io::Error::new(io::ErrorKind::Other,
                               "failed to write entire datagram to socket").into())
        }
    }

    fn close(&mut self) -> Poll<(), C::Error> {
        try_ready!(self.poll_complete());
        Ok(().into())
    }
}

const INITIAL_RD_CAPACITY: usize = 64 * 1024;
const INITIAL_WR_CAPACITY: usize = 8 * 1024;

impl<C> NhrpFramed<C>{
    #![allow(dead_code)]
    pub fn new(socket: NhrpSocket, codec: C) -> NhrpFramed<C> {
        NhrpFramed {
            socket: socket,
            codec: codec,
            rd: BytesMut::with_capacity(INITIAL_RD_CAPACITY),
            wr: BytesMut::with_capacity(INITIAL_WR_CAPACITY),
            out_addr: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            flushed: true,
        }
    }


    pub fn get_ref(&self) -> &NhrpSocket {
        &self.socket
    }
    pub fn get_mut(&mut self) -> &mut NhrpSocket {
        &mut self.socket
    }
    pub fn into_inner(self) -> NhrpSocket {
        self.socket
    }
}
