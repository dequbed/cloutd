use libc as c;

use pnet_sys as p;

use std::io;
use std::mem;
use std::net::SocketAddr;
use mio::{self, Evented, Token, Ready, PollOpt};
use mio::unix::EventedFd;
use std::os::unix::io::RawFd;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::{Poll, Async};

use tokio::reactor::Handle;
use tokio::reactor::PollEvented2;

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
                self.io.clear_write_ready();
                Ok(Async::NotReady)
            },
            Err(e) => Err(e)
        }
    }

    pub fn poll_recv_from(&mut self, buf: &mut [u8]) -> Poll<(usize, IpAddr), io::Error> {
        try_ready!(self.io.poll_read_ready(Ready::readable()));
        let mut caddr: p::SockAddrStorage = unsafe { mem::zeroed() };

        match self.io.get_ref().recv_from(buf, &mut caddr) {
            Ok(n) => {
                let a = sockaddr_to_addr(&caddr, mem::size_of::<p::SockAddrStorage>());
                Ok((n, a.unwrap()).into())
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_read_ready(Ready::readable());
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
        println!("recvfrom {}", len);
        let flags = 0;
        let res = unsafe { c::recvfrom(self.fd, cbuf as *mut c::c_void, len, flags, caddr as *mut c::sockaddr, &mut caddrlen) };

        println!("Received {} bytes", res);

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

pub fn addr_to_sockaddr(_addr: IpAddr, _storage: &mut p::SockAddrStorage) -> p::SockLen {
    0
}

pub fn sockaddr_to_addr(storage: &p::SockAddrStorage, len: usize) -> io::Result<IpAddr> {
    match storage.ss_family as c::c_int {
        c::PF_PACKET => {
            assert!(len as usize >= mem::size_of::<c::sockaddr_ll>());
            let storage: &c::sockaddr_ll = unsafe { mem::transmute(storage) };
            println!("sll_addr: {:?}, sll_protocol: {:#06X}, sll_hatype: {:#06X}, sll_pkttype: {:#06X}", storage.sll_addr, u16::from_be(storage.sll_protocol), u16::from_be(storage.sll_hatype), storage.sll_pkttype);
            if storage.sll_protocol == 0x2001u16.to_be() {

                // FIXME: Find a better way to figure out the underlying address type
                //        Maybe don't strip the GRE header (aka use PROTO_RAW in the socket) and
                //        look at the ethertype?

                match storage.sll_halen {
                    4 => {
                        let mut addr: [u8; 4] = [0;4];
                        addr.clone_from_slice(&storage.sll_addr[0..4]);
                        let ip = u32::from_be(u32::from_bytes(addr));
                        let a = (ip >> 24) as u8;
                        let b = (ip >> 16) as u8;
                        let c = (ip >> 8) as u8;
                        let d = ip as u8;
                        Ok(IpAddr::V4(Ipv4Addr::new(a, b, c, d)))
                    },
                    // FIXME: IPv6 Support!
                    _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Not implemented yet, sorry."))
                }
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "Not implemented yet, sorry."))
            }
        },
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Not implemented yet, sorry."))
    }
}
