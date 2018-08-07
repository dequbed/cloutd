use ::nhrp::socket::NhrpSocket;

use std::io;
use std::net::IpAddr;

use futures::{Async, Future, Poll};

#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct RecvNhrp<T> {
    /// None means future was completed
    state: Option<RecvNhrpInner<T>>
}

#[derive(Debug)]
struct RecvNhrpInner<T> {
    socket: NhrpSocket,
    buffer: T
}

impl<T> RecvNhrp<T> {
    pub(crate) fn new(socket: NhrpSocket, buffer: T) -> RecvNhrp<T> {
        let inner = RecvNhrpInner { socket: socket, buffer: buffer };
        RecvNhrp { state: Some(inner) }
    }
}

impl<T> Future for RecvNhrp<T>
    where T: AsMut<[u8]>,
{
    type Item = (NhrpSocket, T, usize, IpAddr);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        let (n, addr) = {
            let ref mut inner = self.state.as_mut().expect("RecvNhrp polled after completion");
            try_ready!(inner.socket.poll_recv_from(inner.buffer.as_mut()))
        };

        let inner = self.state.take().unwrap();
        Ok(Async::Ready((inner.socket, inner.buffer, n, addr)))
    }
}
