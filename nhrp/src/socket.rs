/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */
#![allow(dead_code)]

use libc as c;

use std::io;
use std::mem;
use mio::{self, Evented, Token, Ready, PollOpt};
use mio::unix::EventedFd;
use std::os::unix::io::RawFd;

#[allow(unused_imports)]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use byteorder::{ByteOrder, NativeEndian};
use iovec::IoVec;
use iovec::unix;

use futures::{Poll, Async};

use tokio::reactor::Handle;
use tokio::reactor::PollEvented2;

pub type SockAddr = c::sockaddr_ll;

#[derive(Debug)]
pub struct NhrpSocket {
    io: PollEvented2<NhrpRawSocket>,
}

impl NhrpSocket {
    pub fn new() -> io::Result<NhrpSocket> {
        let s = NhrpRawSocket::new()?;
        let io = PollEvented2::new(s);

        Ok (NhrpSocket { io: io })
    }
    pub fn new_with_handle(handle: &Handle) -> io::Result<NhrpSocket> {
        let s = NhrpRawSocket::new()?;
        let io = PollEvented2::new_with_handle(s, handle)?;

        Ok (NhrpSocket { io: io })
    }

    pub fn poll_send_to(&mut self, buf: &[u8], addr: &IpAddr) -> Poll<usize, io::Error> {
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

    pub fn poll_recv_from(&mut self, buf: &mut [u8]) -> Poll<(usize, IpAddr), io::Error> {
        try_ready!(self.io.poll_read_ready(Ready::readable()));
        let mut caddr = unsafe { mem::zeroed() };

        match self.io.get_ref().recv_from(buf, &mut caddr) {
            Ok(n) => {
                let a = sockaddr_to_addr(&caddr);
                Ok((n, a.unwrap()).into())
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_read_ready(Ready::readable())?;
                Ok(Async::NotReady)
            },
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug)]
pub struct NhrpRawSocket {
    fd: RawFd,
}

impl NhrpRawSocket {
    pub fn new() -> io::Result<NhrpRawSocket> {
        let protocol: i16 = 0x2001i16.to_be();
        let fd = unsafe {  c::socket(c::PF_PACKET, c::SOCK_DGRAM, protocol as i32) };

        if fd < 0 {
            let err = io::Error::last_os_error();
            return Err(err);
        }

        unsafe {
            c::fcntl(fd, c::F_SETFL, c::O_NONBLOCK);
        }

        Ok (NhrpRawSocket { fd: fd })
    }

    pub fn send_to(&self, buf: &[u8], addr: &IpAddr) -> io::Result<usize> {
        let mut caddr = unsafe { mem::zeroed() };
        let slen = addr_to_sockaddr(*addr, &mut caddr);
        let caddr_ptr = &caddr as *const SockAddr as *const c::sockaddr;

        let iov: &[&IoVec] = &mut [buf.into()];
        let iovs = unix::as_os_slice(iov);
        let msg = c::msghdr {
            msg_name: caddr_ptr as *mut c::c_void,
            msg_namelen: slen,
            msg_iov: iovs.as_ptr() as *mut c::iovec,
            msg_iovlen: iovs.len(),
            msg_control: (0 as *mut c::c_void),
            msg_controllen: 0,
            msg_flags: 0,
        };

        let res = unsafe { c::sendmsg(self.fd, &msg as *const c::msghdr, 0) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(res as usize)
    }

    pub fn recv_from(&self, buf: &mut [u8], caddr: *mut SockAddr) -> io::Result<usize> {
        let mut caddrlen = mem::size_of::<SockAddr>() as c::socklen_t;


        let cbuf = buf.as_ptr();
        let len = buf.len();
        let flags = 0;
        let res = unsafe { c::recvfrom(self.fd, cbuf as *mut c::c_void, len, flags, caddr as *mut c::sockaddr, &mut caddrlen) };

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

pub fn addr_to_sockaddr(addr: IpAddr, sockaddr: &mut SockAddr) -> c::socklen_t {
    use std::net::IpAddr::*;
    use std::mem;
    match addr {
        V4(addr) => {
            (*sockaddr).sll_family = c::AF_PACKET as u16;
            (*sockaddr).sll_protocol = 0x2001u16.to_be();
            (*sockaddr).sll_ifindex = 6; // TODO!
            (*sockaddr).sll_halen = 4u8;

            let bytes = addr.octets();
            let mut addrbuf = &mut sockaddr.sll_addr[0..4];
            addrbuf.copy_from_slice(&bytes);
            addrbuf.reverse();

            mem::size_of::<SockAddr>() as c::socklen_t
        }
        _ => {0}
    }
}

pub fn sockaddr_to_addr(addr: &SockAddr) -> io::Result<IpAddr> {
    assert_eq!(addr.sll_family as c::c_int, c::AF_PACKET, "Passed sockaddr is not a PF_PACKET sockaddr!");
    match addr.sll_hatype {
        778 /*c::ARPHRD_IPGRE*/ => {
            if addr.sll_protocol == 0x2001u16.to_be() {

                let ip = NativeEndian::read_u32(&addr.sll_addr[0..4]).into();
                Ok(IpAddr::V4(ip))
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "Not implemented yet, sorry."))
            }
        },
        823 /*c::ARPHRD_IP6GRE*/ =>
            Err(io::Error::new(io::ErrorKind::InvalidData, "IPv6 is not implemented yet, sorry.")),
        v => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Protocol {} is not implemented.", v))),
    }
}
