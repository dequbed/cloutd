use nix::sys::socket::{
    recvmsg, sendmsg, socket, AddressFamily, MsgFlags, RecvMsg, SockFlag, SockProtocol, SockType,
    SockaddrLike,
};
use std::io::{IoSlice, IoSliceMut};
use std::net::IpAddr;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, mem};

use tokio::io::unix::AsyncFd;

#[derive(Debug)]
#[repr(transparent)]
pub struct NhrpSocket {
    io: AsyncFd<RawNhrpSocket>,
}

impl NhrpSocket {
    pub fn new_v4() -> io::Result<NhrpSocket> {
        let protocol: SockProtocol = unsafe { mem::transmute(0x2001i32.to_be()) };
        let socket = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::SOCK_NONBLOCK,
            protocol,
        )?;

        Ok(Self {
            io: AsyncFd::new(RawNhrpSocket { socket })?,
        })
    }

    pub async fn send_vectored(
        &self,
        bufs: &[IoSlice<'_>],
        addr: &impl SockaddrLike,
    ) -> io::Result<usize> {
        loop {
            let mut guard = self.io.writable().await?;

            match guard.try_io(|asyncfd| asyncfd.get_ref().send_vectored(bufs, addr)) {
                Err(_would_block) => continue,
                Ok(result) => return result,
            }
        }
    }

    pub async fn recv_vectored<'a, S: SockaddrLike + 'a>(
        &self,
        bufs: &mut [IoSliceMut<'_>],
    ) -> io::Result<RecvMsg<'a, S>> {
        loop {
            let mut guard = self.io.readable().await?;

            match guard.try_io(|asyncfd| asyncfd.get_ref().recv_vectored(bufs)) {
                Err(_would_block) => continue,
                Ok(result) => return result,
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
