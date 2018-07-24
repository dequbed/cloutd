use libc as c;

use pnet_sys as p;

use std::io;
use std::mem;
use std::net::SocketAddr;
use mio::{self, Evented, Token, Ready, PollOpt};
use mio::unix::EventedFd;
use std::os::unix::io::RawFd;

use futures::{Poll, Async};

use tokio::reactor::Handle;
use tokio::reactor::PollEvented2 as PollEvented;

#[derive(Debug)]
pub struct NhrpSocket {
    io: PollEvented<NhrpRawSocket>
}

impl NhrpSocket {
    pub fn new() -> io::Result<NhrpSocket> {
        let s = NhrpRawSocket::new()?;

        Ok (NhrpSocket { io: PollEvented::new(s) })
    }
    pub fn new_with_handle(handle: &Handle) -> io::Result<NhrpSocket> {
        let s = NhrpRawSocket::new()?;

        Ok (NhrpSocket { io: PollEvented::new_with_handle(s, handle)? })
    }

    pub fn poll_send_to(&mut self, buf: &[u8], addr: &SocketAddr) -> Poll<usize, io::Error> {
        try_ready!(self.io.poll_write_ready());

        match self.io.get_ref().send_to(buf, addr) {
            Ok(n) => Ok(n.into()),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_write_ready()?;
                Ok(Async::NotReady)
            },
            Err(e) => Err(e)
        }
    }

    pub fn poll_recv_from(&mut self, buf: &mut [u8]) -> Poll<(usize, SocketAddr), io::Error> {
        try_ready!(self.io.poll_read_ready(Ready::readable()));

        let mut caddr: p::SockAddrStorage = unsafe { mem::zeroed() };

        match self.io.get_ref().recv_from(buf, &mut caddr) {
            Ok(n) => Ok((n, p::sockaddr_to_addr(&caddr, mem::size_of::<p::SockAddrStorage>())?).into()),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_read_ready(Ready::readable())?;
                Ok(Async::NotReady)
            },
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug)]
struct NhrpRawSocket {
    fd: RawFd,
}

impl NhrpRawSocket {
    pub fn new() -> io::Result<NhrpRawSocket> {
        let protocol: i32 = 0x2001;
        let fd = unsafe {  p::socket(c::AF_PACKET, c::SOCK_DGRAM | c::O_NONBLOCK, protocol.to_le()) };

        if fd < 0 {
            let err = io::Error::last_os_error();
            return Err(err);
        }

        Ok (NhrpRawSocket { fd: fd })

    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let mut caddr = unsafe { mem::zeroed() };
        let slen = p::addr_to_sockaddr(*addr, &mut caddr);
        let caddr_ptr = (&caddr as *const p::SockAddrStorage) as *const p::SockAddr;

        let cbuf = buf.as_ptr();
        let len = buf.len();
        let flags = 0;
        let res = unsafe { c::sendto(self.fd, cbuf as *const c::c_void, len, flags, caddr_ptr, slen) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(res as usize)
    }

    pub fn recv_from(&self, buf: &mut [u8], caddr: *mut p::SockAddrStorage) -> io::Result<usize> {
        let mut caddrlen = mem::size_of::<p::SockAddrStorage>() as p::SockLen;

        let cbuf = buf.as_ptr();
        let len = buf.len();
        let flags = 0;
        let res = unsafe { c::recvfrom(self.fd, cbuf as *mut c::c_void, len, flags, caddr as *mut p::SockAddr, &mut caddrlen) };

        if res < 0 {
            return Err(io::Error::last_os_error())
        }
        Ok(res as usize)
    }
}

impl Evented for NhrpRawSocket {
    fn register(&self, poll: &mio::Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        EventedFd(&self.fd).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &mio::Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        EventedFd(&self.fd).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
        EventedFd(&self.fd).deregister(poll)
    }
}

impl Drop for NhrpRawSocket {
    fn drop(&mut self) {
        unsafe { c::close(self.fd) };
    }
}
