use crate::action::{Action, ActionType};
use crate::alias::Alias;
use crate::macro_def::Macro;
use crate::mud::{Mud, MudList};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: Option<(Ipv4Addr, u16)>,
    pub mud_list: MudList,
    pub global_mud: Mud, // Global aliases/actions/macros
}

impl Config {
    pub fn new() -> Self {
        Self {
            server: None,
            mud_list: MudList::new(),
            global_mud: Mud::new("__global__", "", 0),
        }
    }

    pub fn set_server_str(&mut self, s: &str) -> Result<(), String> {
        let (ip_s, port_s) = s
            .split_once(':')
            .ok_or_else(|| "expected ip:port".to_string())?;
        let ip = ip_s.parse::<Ipv4Addr>().map_err(|e| e.to_string())?;
        let port = port_s.parse::<u16>().map_err(|e| e.to_string())?;
        self.server = Some((ip, port));
        Ok(())
    }

    /// Load config from file (supports both old and new formats)
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = File::open(path).map_err(|e| format!("Failed to open config: {}", e))?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Read error at line {}: {}", line_num + 1, e))?;

            // Skip comments and empty lines
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            self.parse_line(&line, line_num + 1)?;
        }

        Ok(())
    }

    /// Parse a single config line
    fn parse_line(&mut self, line: &str, line_num: usize) -> Result<(), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "mud" if parts.len() >= 2 => {
                // New format: MUD mudname { ... }
                // For now, return error - we'll implement block parsing next
                Err(format!(
                    "Line {}: MUD block format not yet implemented",
                    line_num
                ))
            }
            // Old format: mudname hostname port [commands]
            _ if parts.len() >= 3 => {
                let mudname = parts[0];
                let hostname = parts[1];
                let port: u16 = parts[2]
                    .parse()
                    .map_err(|_| format!("Line {}: Invalid port number", line_num))?;

                let commands = parts
                    .get(3..)
                    .map(|rest| rest.join(" "))
                    .unwrap_or_default();

                let mut mud = Mud::new(mudname, hostname, port);
                mud.commands = commands;
                self.mud_list.insert(mud);
                Ok(())
            }
            _ => Err(format!("Line {}: Invalid config line format", line_num)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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

    #[test]
    fn config_old_format() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "# Comment line").unwrap();
        writeln!(tmpfile, "TestMUD 127.0.0.1 4000").unwrap();
        writeln!(tmpfile, "Nodeka nodeka.com 23 look").unwrap();
        writeln!(tmpfile, "").unwrap(); // Empty line
        writeln!(tmpfile, "AnotherMUD 192.168.1.1 5000 connect").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        assert_eq!(cfg.mud_list.count(), 3);

        let mud1 = cfg.mud_list.find("TestMUD").unwrap();
        assert_eq!(mud1.hostname, "127.0.0.1");
        assert_eq!(mud1.port, 4000);
        assert_eq!(mud1.commands, "");

        let mud2 = cfg.mud_list.find("Nodeka").unwrap();
        assert_eq!(mud2.hostname, "nodeka.com");
        assert_eq!(mud2.port, 23);
        assert_eq!(mud2.commands, "look");

        let mud3 = cfg.mud_list.find("AnotherMUD").unwrap();
        assert_eq!(mud3.hostname, "192.168.1.1");
        assert_eq!(mud3.port, 5000);
        assert_eq!(mud3.commands, "connect");
    }

    #[test]
    fn config_invalid_line() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "InvalidLine").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        assert!(cfg.load_file(tmpfile.path()).is_err());
    }
}
