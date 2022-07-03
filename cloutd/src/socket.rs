use nix::sys::socket::{
    recvmsg, sendmsg, socket, AddressFamily, MsgFlags, RecvMsg, SockFlag, SockProtocol, SockType,
    SockaddrLike,
};
use std::io::{IoSlice, IoSliceMut};
use std::net::IpAddr;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, mem};

use thiserror::Error;
use miette::Diagnostic;

use tokio::io::unix::AsyncFd;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("NHRP socket could not be opened")]
    #[diagnostic(code("nhrp::socket::open"))]
    Socket(#[source] io::Error),

    #[error("nhrp socket async wrapper could not be constructed")]
    #[diagnostic(code("nhrp::socket::asyncfd"))]
    AsyncFd(#[source] io::Error),

    #[error("waiting for nhrp socket readiness failed")]
    #[diagnostic(code("nhrp::socket::readiness"))]
    Readiness(#[source] io::Error),

    #[error("receiving NHRP message failed")]
    #[diagnostic(code("nhrp::socket::recv"))]
    Recv(#[source] io::Error),

    #[error("sending NHRP message failed")]
    #[diagnostic(code("nhrp::socket::send"))]
    Send(#[source] io::Error),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct NhrpSocket {
    io: AsyncFd<RawNhrpSocket>,
}

impl NhrpSocket {
    pub fn new_v4() -> Result<NhrpSocket, Error> {
        let protocol: SockProtocol = unsafe { mem::transmute(0x2001i32.to_be()) };
        let socket = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::SOCK_NONBLOCK,
            protocol,
        ).map_err(|errno| Error::Socket(io::Error::from_raw_os_error(errno as i32)))?;

        Ok(Self {
            io: AsyncFd::new(RawNhrpSocket { socket }).map_err(Error::AsyncFd)?,
        })
    }

    pub async fn send_vectored(
        &self,
        bufs: &[IoSlice<'_>],
        addr: &impl SockaddrLike,
    ) -> Result<usize, Error> {
        loop {
            let mut guard = self.io.writable().await.map_err(Error::Readiness)?;

            match guard.try_io(|asyncfd| asyncfd.get_ref().send_vectored(bufs, addr)) {
                Err(_would_block) => continue,
                Ok(result) => return result.map_err(Error::Send),
            }
        }
    }

    pub async fn recv_vectored<'a, S: SockaddrLike + 'a>(
        &self,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Result<RecvMsg<'a, S>, Error> {
        loop {
            let mut guard = self.io.readable().await.map_err(Error::Readiness)?;

            match guard.try_io(|asyncfd| asyncfd.get_ref().recv_vectored(bufs)) {
                Err(_would_block) => continue,
                Ok(result) => return result.map_err(Error::Recv),
            }
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct RawNhrpSocket {
    socket: RawFd,
}

impl RawNhrpSocket {
    pub fn send_vectored(
        &self,
        bufs: &[IoSlice<'_>],
        addr: &impl SockaddrLike,
    ) -> io::Result<usize> {
        sendmsg(self.as_raw_fd(), bufs, &[], MsgFlags::empty(), Some(addr))
            .map_err(|errno| io::Error::from_raw_os_error(errno as i32))
    }

    pub fn recv_vectored<'a, 'outer, 'inner, S: SockaddrLike + 'a>(
        &self,
        bufs: &'outer mut [IoSliceMut<'inner>],
    ) -> io::Result<RecvMsg<'a, S>> {
        recvmsg(self.as_raw_fd(), bufs, None, MsgFlags::empty())
            .map_err(|errno| io::Error::from_raw_os_error(errno as i32))
    }
}

impl AsRawFd for RawNhrpSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket
    }
}
