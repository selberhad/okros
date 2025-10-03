use crate::socket::{Socket, ConnState};
use crate::config::Config;
use std::io;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Mud {
    pub sock: Option<Socket>,
    pub state: ConnState,
}

impl Mud {
    pub fn new() -> Self { Self{ sock: None, state: ConnState::Idle } }
    pub fn connect_from_config(&mut self, cfg: &Config) -> io::Result<()> {
        if let Some((ip, port)) = &cfg.server {
            let mut s = Socket::new()?;
            let _ = s.connect_ipv4(*ip, *port);
            self.state = s.state;
            self.sock = Some(s);
            Ok(())
        } else { Err(io::Error::new(io::ErrorKind::InvalidInput, "no server")) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    #[test]
    fn connect_loopback_from_config() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut cfg = Config::new(); cfg.server = Some((Ipv4Addr::LOCALHOST, port));
        let mut m = Mud::new();
        m.connect_from_config(&cfg).unwrap();
        assert!(matches!(m.state, ConnState::Connecting | ConnState::Connected));
    }
}

