use ::nhrp::socket::NhrpSocket;

use std::io;
use std::net::SocketAddr;

use futures::{Async, Future, Poll};


#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct SendNhrp<T> {
    state: Option<SendNhrpInner<T>>
}

#[derive(Debug)]
struct SendNhrpInner<T> {
    socket: NhrpSocket,
    buffer: T,
    addr: SocketAddr,
}

impl<T> SendNhrp<T> {
    pub(crate) fn new(socket: NhrpSocket, buffer: T, addr: SocketAddr) -> SendNhrp<T> {
        let inner = SendNhrpInner { socket: socket, buffer: buffer, addr: addr };
        SendNhrp { state: Some(inner) }
    }
}

impl<T> Future for SendNhrp<T>
    where T: AsRef<[u8]>, T: AsMut<[u8]>
{
    type Item = (NhrpSocket, T);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        {
            let ref mut inner =
                self.state.as_mut().expect("SendNhrp polled after completion");
            let n = try_ready!(inner.socket.poll_send_to(inner.buffer.as_mut(), &inner.addr));

            if n != inner.buffer.as_ref().len() {
                return Err(io::Error::new(io::ErrorKind::Other,
                            "failed to send entire Packet at once"));
            }
        }

        let inner = self.state.take().unwrap();

        Ok(Async::Ready((inner.socket, inner.buffer)))
    }
}
