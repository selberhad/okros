use crate::action::Action;
use crate::alias::Alias;
use crate::config::Config;
use crate::macro_def::Macro;
use crate::socket::{ConnState, Socket};
use std::io;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Mud {
    pub sock: Option<Socket>,
    pub state: ConnState,
    pub alias_list: Vec<Alias>,
    pub action_list: Vec<Action>,
    pub macro_list: Vec<Macro>,
}

impl Mud {
    pub fn new() -> Self {
        Self {
            sock: None,
            state: ConnState::Idle,
            alias_list: Vec::new(),
            action_list: Vec::new(),
            macro_list: Vec::new(),
        }
    }

    /// Find alias by name
    pub fn find_alias(&self, name: &str) -> Option<&Alias> {
        self.alias_list.iter().find(|a| a.name == name)
    }

    /// Find macro by key code
    pub fn find_macro(&self, key: i32) -> Option<&Macro> {
        self.macro_list.iter().find(|m| m.key == key)
    }
    pub fn connect_from_config(&mut self, cfg: &Config) -> io::Result<()> {
        if let Some((ip, port)) = &cfg.server {
            let mut s = Socket::new()?;
            let _ = s.connect_ipv4(*ip, *port);
            self.state = s.state;
            self.sock = Some(s);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "no server"))
        }
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
        let mut cfg = Config::new();
        cfg.server = Some((Ipv4Addr::LOCALHOST, port));
        let mut m = Mud::new();
        m.connect_from_config(&cfg).unwrap();
        assert!(matches!(
            m.state,
            ConnState::Connecting | ConnState::Connected
        ));
    }
}
