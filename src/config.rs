use std::net::Ipv4Addr;

#[derive(Default, Debug, Clone)]
pub struct Config {
    pub server: Option<(Ipv4Addr, u16)>,
}

impl Config {
    pub fn new() -> Self { Self::default() }
    pub fn set_server_str(&mut self, s: &str) -> Result<(), String> {
        let (ip_s, port_s) = s.split_once(':').ok_or_else(|| "expected ip:port".to_string())?;
        let ip = ip_s.parse::<Ipv4Addr>().map_err(|e| e.to_string())?;
        let port = port_s.parse::<u16>().map_err(|e| e.to_string())?;
        self.server = Some((ip, port));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_server_ok() {
        let mut c = Config::new();
        c.set_server_str("127.0.0.1:4000").unwrap();
        assert_eq!(c.server.unwrap().1, 4000);
    }
    #[test]
    fn parse_server_bad() {
        let mut c = Config::new();
        assert!(c.set_server_str("bad").is_err());
    }
}

