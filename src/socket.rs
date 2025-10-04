use std::io;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::RawFd;

use libc::{self, c_int};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    Idle,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug)]
pub struct Socket {
    fd: RawFd,
    pub state: ConnState,
    pub last_error: Option<i32>,
    pub local: Option<SocketAddr>,
    pub remote: Option<SocketAddr>,
}

impl Socket {
    pub fn new() -> io::Result<Self> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // set nonblocking
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        Ok(Self {
            fd,
            state: ConnState::Idle,
            last_error: None,
            local: None,
            remote: None,
        })
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.fd
    }

    pub fn connect_ipv4(&mut self, ip: Ipv4Addr, port: u16) -> io::Result<()> {
        let mut addr: libc::sockaddr_in = unsafe { mem::zeroed() };
        addr.sin_family = libc::AF_INET as libc::sa_family_t;
        addr.sin_port = u16::to_be(port);
        addr.sin_addr = libc::in_addr {
            s_addr: u32::from(ip).to_be(),
        };
        let ret = unsafe {
            libc::connect(
                self.fd,
                &addr as *const _ as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_in>() as u32,
            )
        };
        if ret == 0 {
            self.state = ConnState::Connected;
            self.fill_endpoints();
            Ok(())
        } else {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINPROGRESS) {
                self.state = ConnState::Connecting;
                Ok(())
            } else {
                self.state = ConnState::Error;
                self.last_error = err.raw_os_error();
                Err(err)
            }
        }
    }

    pub fn on_writable(&mut self) -> io::Result<()> {
        if self.state != ConnState::Connecting {
            return Ok(());
        }
        // Check SO_ERROR
        let mut err: c_int = 0;
        let mut len = mem::size_of::<c_int>() as libc::socklen_t;
        let rc = unsafe {
            libc::getsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                &mut err as *mut _ as *mut _,
                &mut len,
            )
        };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        if err == 0 {
            self.state = ConnState::Connected;
            self.fill_endpoints();
            Ok(())
        } else {
            self.state = ConnState::Error;
            self.last_error = Some(err);
            Err(io::Error::from_raw_os_error(err))
        }
    }

    fn fill_endpoints(&mut self) {
        // local
        let mut ss: libc::sockaddr_in = unsafe { mem::zeroed() };
        let mut len = mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        let rc = unsafe {
            libc::getsockname(self.fd, &mut ss as *mut _ as *mut libc::sockaddr, &mut len)
        };
        if rc == 0 {
            let port = u16::from_be(ss.sin_port);
            let ip = Ipv4Addr::from(u32::from_be(ss.sin_addr.s_addr));
            self.local = Some(SocketAddr::new(IpAddr::V4(ip), port));
        }
        let mut ps: libc::sockaddr_in = unsafe { mem::zeroed() };
        let mut len2 = mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        let rc2 = unsafe {
            libc::getpeername(self.fd, &mut ps as *mut _ as *mut libc::sockaddr, &mut len2)
        };
        if rc2 == 0 {
            let port = u16::from_be(ps.sin_port);
            let ip = Ipv4Addr::from(u32::from_be(ps.sin_addr.s_addr));
            self.remote = Some(SocketAddr::new(IpAddr::V4(ip), port));
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::time::Duration;

    fn wait_writable(fd: RawFd, timeout_ms: i32) -> io::Result<bool> {
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        };
        let rc = unsafe { libc::poll(&mut pfd as *mut libc::pollfd, 1, timeout_ms) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(rc > 0 && (pfd.revents & libc::POLLOUT) != 0)
    }

    #[test]
    fn nonblocking_connect_loopback() {
        // Start a loopback server
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        // Connect nonblocking
        let mut s = Socket::new().unwrap();
        let res = s.connect_ipv4(Ipv4Addr::LOCALHOST, addr.port());
        assert!(res.is_ok());
        // If still connecting, wait for writable and finalize
        if s.state == ConnState::Connecting {
            assert!(wait_writable(s.as_raw_fd(), 1000).unwrap());
            let _ = s.on_writable();
        }
        assert_eq!(s.state, ConnState::Connected);
        assert!(s.local.is_some());
        // Accept the connection to complete 3-way handshake
        let _accepted = listener.accept().unwrap();
    }

    #[test]
    fn connect_refused() {
        // Choose an unlikely port; bind a listener then close to ensure refusal.
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut s = Socket::new().unwrap();
        let _ = s.connect_ipv4(Ipv4Addr::LOCALHOST, port);
        if s.state == ConnState::Connecting {
            let _ = wait_writable(s.as_raw_fd(), 500).unwrap();
            let _ = s.on_writable();
        }
        assert_eq!(s.state, ConnState::Error);
        assert_eq!(s.last_error.is_some(), true);
    }
}
