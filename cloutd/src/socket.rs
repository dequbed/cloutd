use nix::sys::socket::{recvmsg, sendmsg, socket, AddressFamily, MsgFlags, RecvMsg, SockFlag, SockProtocol, SockType, SockaddrLike, recvfrom};
use std::io::{IoSlice, IoSliceMut};
use std::net::IpAddr;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, mem};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use thiserror::Error;
use miette::Diagnostic;
use nix::errno::Errno;
use tokio::io::{AsyncRead, AsyncWrite};

use tokio::io::unix::AsyncFd;
use crate::error::{ErrnoAdvice, ErrnoErr};

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("opening NHRP packet socket failed")]
    #[diagnostic(code(nhrp::socket::open))]
    Socket {
        #[source]
        #[diagnostic_source]
        errno: ErrnoErr,
        #[help]
        help: Option<&'static str>,
    },

    #[error("nhrp socket async wrapper could not be constructed")]
    #[diagnostic(code(nhrp::socket::asyncfd))]
    AsyncFd(#[source] io::Error),

    #[error("waiting for nhrp socket readiness failed")]
    #[diagnostic(code(nhrp::socket::readiness))]
    Readiness(#[source] io::Error),

    #[error("receiving NHRP message failed")]
    #[diagnostic(code(nhrp::socket::recv))]
    Recv(#[source] io::Error),

    #[error("sending NHRP message failed")]
    #[diagnostic(code(nhrp::socket::send))]
    Send(#[source] io::Error),
}
impl Error {
    fn socket(err: ErrnoErr) -> Self {
        Self::Socket {
            help: err.advice,
            errno: err,
        }
    }
}

pub struct Advice;
impl ErrnoAdvice for Advice {
    const EPERM: Option<&'static str> = Some("Opening raw packet sockets requires root privileges");
}

#[derive(Debug)]
#[repr(transparent)]
pub struct NhrpSocket {
    io: AsyncFd<RawNhrpSocket>,
}

impl NhrpSocket {
    pub fn new() -> Result<Self, Error> {
        let protocol: SockProtocol = unsafe { mem::transmute(0x2001i32.to_be()) };
        let socket = socket(
            AddressFamily::Packet,
            SockType::Datagram,
            SockFlag::SOCK_NONBLOCK,
            protocol,
        ).map_err(|errno| Error::socket(ErrnoErr::with::<Advice>(errno).into()))?;

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

    pub async fn recv<S: SockaddrLike>(&self, buf: &mut [u8]) -> Result<(usize, Option<S>), Error> {
        loop {
            let mut guard = self.io.readable().await.map_err(Error::Readiness)?;

            match guard.try_io(|asyncfd| asyncfd.get_ref().recv(buf)) {
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

    pub fn recv<S: SockaddrLike>(&self, buf: &mut [u8]) -> io::Result<(usize, Option<S>)> {
        recvfrom(self.as_raw_fd(), buf)
            .map_err(|errno| io::Error::from_raw_os_error(errno as i32))
    }
}

impl AsRawFd for RawNhrpSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket
    }
}
