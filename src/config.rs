use crate::action::{Action, ActionType};
use crate::alias::Alias;
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

    /// Create config with offline MUD as entry #0
    pub fn with_offline_mud() -> Self {
        let mut config = Self::new();

        // Add offline/internal MUD as first entry
        let mut offline = Mud::new("Offline", "", 0);
        offline.comment = "Internal test MUD (no network required)".to_string();
        config.mud_list.insert(offline);

        config
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
    /// Automatically adds Offline MUD as entry #0 if not present
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = File::open(path).map_err(|e| format!("Failed to open config: {}", e))?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines().enumerate();

        while let Some((line_num, line_result)) = lines.next() {
            let line =
                line_result.map_err(|e| format!("Read error at line {}: {}", line_num + 1, e))?;

            // Skip comments and empty lines
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            // Check for MUD block format
            if parts[0].eq_ignore_ascii_case("mud") && parts.len() >= 2 {
                let mudname = parts[1].trim_end_matches('{').trim();
                self.read_mud_block(mudname, &mut lines)?;
            } else {
                // Old format or other config line
                self.parse_line(&line, line_num + 1)?;
            }
        }

        // Ensure Offline MUD is present (add as entry #0 if not found)
        self.ensure_offline_mud();

        Ok(())
    }

    /// Ensure Offline MUD exists in list (prepend if not present)
    fn ensure_offline_mud(&mut self) {
        // Check if Offline MUD already exists
        if self.mud_list.find("Offline").is_some() {
            return;
        }

        // Create and prepend Offline MUD
        let mut offline = Mud::new("Offline", "", 0);
        offline.comment = "Internal test MUD (no network required)".to_string();

        // Create new list with Offline first
        let mut new_list = MudList::new();
        new_list.insert(offline);

        // Add all existing MUDs
        for mud in self.mud_list.iter() {
            new_list.insert(mud.clone());
        }

        self.mud_list = new_list;
    }

    /// Read a MUD block in new format: MUD mudname { ... }
    fn read_mud_block(
        &mut self,
        mudname: &str,
        lines: &mut impl Iterator<Item = (usize, Result<String, std::io::Error>)>,
    ) -> Result<(), String> {
        let mut mud = Mud::new(mudname, "", 0);

        while let Some((line_num, line_result)) = lines.next() {
            let line =
                line_result.map_err(|e| format!("Read error at line {}: {}", line_num + 1, e))?;

            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for end of block
            if trimmed.starts_with('}') {
                // Add completed MUD to list
                self.mud_list.insert(mud);
                return Ok(());
            }

            // Parse block line
            self.parse_mud_block_line(&mut mud, trimmed, line_num + 1)?;
        }

        Err(format!(
            "MUD block for '{}' not properly terminated with }}",
            mudname
        ))
    }

    /// Parse a line inside a MUD block
    fn parse_mud_block_line(
        &mut self,
        mud: &mut Mud,
        line: &str,
        line_num: usize,
    ) -> Result<(), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0].to_lowercase().as_str() {
            "host" if parts.len() >= 3 => {
                mud.hostname = parts[1].to_string();
                mud.port = parts[2]
                    .trim_end_matches(';')
                    .parse()
                    .map_err(|_| format!("Line {}: Invalid port number", line_num))?;
                Ok(())
            }
            "commands" if parts.len() >= 2 => {
                mud.commands = parts[1..].join(" ").trim_end_matches(';').to_string();
                Ok(())
            }
            "inherit" if parts.len() >= 2 => {
                let parent_name = parts[1].trim_end_matches(';');
                if let Some(parent) = self.mud_list.find(parent_name) {
                    mud.inherits = Some(Box::new(parent.clone()));
                    Ok(())
                } else {
                    Err(format!(
                        "Line {}: Parent MUD '{}' not found",
                        line_num, parent_name
                    ))
                }
            }
            "alias" if parts.len() >= 3 => {
                let name = parts[1];
                let expansion = parts[2..].join(" ").trim_end_matches(';').to_string();
                mud.alias_list.push(Alias::new(name, &expansion));
                Ok(())
            }
            "action" if parts.len() >= 3 => {
                // Parse action: action "pattern" commands
                let rest = parts[1..].join(" ").trim_end_matches(';').to_string();
                match Action::parse(&rest, ActionType::Trigger) {
                    Ok(action) => {
                        mud.action_list.push(action);
                        Ok(())
                    }
                    Err(e) => Err(format!("Line {}: {}", line_num, e)),
                }
            }
            "subst" if parts.len() >= 3 => {
                // Parse substitution: subst "pattern" replacement
                let rest = parts[1..].join(" ").trim_end_matches(';').to_string();
                match Action::parse(&rest, ActionType::Replacement) {
                    Ok(action) => {
                        mud.action_list.push(action);
                        Ok(())
                    }
                    Err(e) => Err(format!("Line {}: {}", line_num, e)),
                }
            }
            "macro" if parts.len() >= 3 => {
                // TODO: Implement macro parsing (need key name lookup)
                // For now, skip macros
                Ok(())
            }
            _ => Err(format!(
                "Line {}: Unknown or invalid MUD block keyword: {}",
                line_num, parts[0]
            )),
        }
    }

    /// Parse a single config line (old format)
    fn parse_line(&mut self, line: &str, line_num: usize) -> Result<(), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        // Old format: mudname hostname port [commands]
        if parts.len() >= 3 {
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
        } else {
            Err(format!("Line {}: Invalid config line format", line_num))
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

        // Offline MUD is auto-added as entry #0
        assert_eq!(cfg.mud_list.count(), 4);

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

    #[test]
    fn config_new_format_basic() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "MUD TestMUD {{").unwrap();
        writeln!(tmpfile, "  host 127.0.0.1 4000;").unwrap();
        writeln!(tmpfile, "  commands look;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        // Offline MUD is auto-added
        assert_eq!(cfg.mud_list.count(), 2);
        let mud = cfg.mud_list.find("TestMUD").unwrap();
        assert_eq!(mud.hostname, "127.0.0.1");
        assert_eq!(mud.port, 4000);
        assert_eq!(mud.commands, "look");
    }

    #[test]
    fn config_new_format_with_aliases() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "MUD Nodeka {{").unwrap();
        writeln!(tmpfile, "  host nodeka.com 23;").unwrap();
        writeln!(tmpfile, "  alias n north;").unwrap();
        writeln!(tmpfile, "  alias s south;").unwrap();
        writeln!(tmpfile, "  alias look l;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        let mud = cfg.mud_list.find("Nodeka").unwrap();
        assert_eq!(mud.hostname, "nodeka.com");
        assert_eq!(mud.alias_list.len(), 3);
        assert!(mud.find_alias("n").is_some());
        assert!(mud.find_alias("s").is_some());
        assert!(mud.find_alias("look").is_some());
    }

    #[test]
    fn config_new_format_with_inheritance() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        // Define parent first
        writeln!(tmpfile, "MUD Parent {{").unwrap();
        writeln!(tmpfile, "  host parent.com 4000;").unwrap();
        writeln!(tmpfile, "  alias p parent_alias;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        // Define child that inherits from parent
        writeln!(tmpfile, "MUD Child {{").unwrap();
        writeln!(tmpfile, "  host child.com 5000;").unwrap();
        writeln!(tmpfile, "  inherit Parent;").unwrap();
        writeln!(tmpfile, "  alias c child_alias;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        let child = cfg.mud_list.find("Child").unwrap();
        assert_eq!(child.hostname, "child.com");
        assert!(child.inherits.is_some());

        // Child should find its own alias
        assert!(child.find_alias("c").is_some());
        // Child should find parent's alias via inheritance
        assert!(child.find_alias("p").is_some());
    }

    #[test]
    fn config_mixed_old_and_new_format() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        // Old format
        writeln!(tmpfile, "OldMUD 192.168.1.1 6000").unwrap();
        // New format
        writeln!(tmpfile, "MUD NewMUD {{").unwrap();
        writeln!(tmpfile, "  host 10.0.0.1 7000;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        // Another old format
        writeln!(tmpfile, "OldMUD2 127.0.0.1 8000 connect").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        // Offline MUD is auto-added
        assert_eq!(cfg.mud_list.count(), 4);
        assert!(cfg.mud_list.find("Offline").is_some());
        assert!(cfg.mud_list.find("OldMUD").is_some());
        assert!(cfg.mud_list.find("NewMUD").is_some());
        assert!(cfg.mud_list.find("OldMUD2").is_some());
    }

    #[test]
    fn config_new_format_unterminated_block() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "MUD BadMUD {{").unwrap();
        writeln!(tmpfile, "  host 127.0.0.1 4000;").unwrap();
        // Missing closing brace
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        let result = cfg.load_file(tmpfile.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("not properly terminated with }"));
    }

    #[test]
    fn config_new_format_invalid_parent() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "MUD Child {{").unwrap();
        writeln!(tmpfile, "  host child.com 5000;").unwrap();
        writeln!(tmpfile, "  inherit NonExistent;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        let result = cfg.load_file(tmpfile.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn config_with_offline_mud() {
        let cfg = Config::with_offline_mud();
        assert_eq!(cfg.mud_list.count(), 1);
        let offline = cfg.mud_list.get(0).unwrap();
        assert_eq!(offline.name, "Offline");
        assert!(offline.comment.contains("Internal"));
    }

    #[test]
    fn config_ensure_offline_mud_prepended() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "TestMUD 127.0.0.1 4000").unwrap();
        writeln!(tmpfile, "Nodeka nodeka.com 23").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        // Should have 3 MUDs: Offline + TestMUD + Nodeka
        assert_eq!(cfg.mud_list.count(), 3);

        // Offline should be first
        let first = cfg.mud_list.get(0).unwrap();
        assert_eq!(first.name, "Offline");

        // Other MUDs should follow
        assert!(cfg.mud_list.find("TestMUD").is_some());
        assert!(cfg.mud_list.find("Nodeka").is_some());
    }

    #[test]
    fn config_full_automation_features() {
        // Integration test: Config file with aliases, actions, and macros
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "# Full automation test config").unwrap();
        writeln!(tmpfile, "MUD TestMUD {{").unwrap();
        writeln!(tmpfile, "  host 127.0.0.1 4000;").unwrap();
        writeln!(tmpfile, "  alias n north;").unwrap();
        writeln!(tmpfile, "  alias s south;").unwrap();
        writeln!(tmpfile, "  alias say tell bob %1;").unwrap();
        writeln!(tmpfile, "  action \"^You are hungry\" eat bread;").unwrap();
        writeln!(tmpfile, "  subst \"stupid\" smart;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        let mud = cfg.mud_list.find("TestMUD").unwrap();

        // Verify aliases loaded
        assert_eq!(mud.alias_list.len(), 3);
        assert!(mud.find_alias("n").is_some());
        assert!(mud.find_alias("s").is_some());
        assert!(mud.find_alias("say").is_some());

        // Verify alias expansion works
        let say_alias = mud.find_alias("say").unwrap();
        assert_eq!(say_alias.expand("hello"), "tell bob hello");

        // Verify actions loaded
        assert_eq!(mud.action_list.len(), 2);

        // First action should be trigger
        assert_eq!(mud.action_list[0].pattern, "^You are hungry");
        assert_eq!(mud.action_list[0].commands, "eat bread");
        assert_eq!(
            mud.action_list[0].action_type,
            crate::action::ActionType::Trigger
        );

        // Second action should be replacement
        assert_eq!(mud.action_list[1].pattern, "stupid");
        assert_eq!(mud.action_list[1].commands, "smart");
        assert_eq!(
            mud.action_list[1].action_type,
            crate::action::ActionType::Replacement
        );
    }

    #[test]
    fn config_automation_with_inheritance() {
        // Test that child MUD inherits parent's automation features
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "MUD Parent {{").unwrap();
        writeln!(tmpfile, "  host parent.com 4000;").unwrap();
        writeln!(tmpfile, "  alias p parent_alias;").unwrap();
        writeln!(tmpfile, "  action \"^parent trigger\" parent_action;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        writeln!(tmpfile, "").unwrap();
        writeln!(tmpfile, "MUD Child {{").unwrap();
        writeln!(tmpfile, "  host child.com 5000;").unwrap();
        writeln!(tmpfile, "  inherit Parent;").unwrap();
        writeln!(tmpfile, "  alias c child_alias;").unwrap();
        writeln!(tmpfile, "}}").unwrap();
        tmpfile.flush().unwrap();

        let mut cfg = Config::new();
        cfg.load_file(tmpfile.path()).unwrap();

        let child = cfg.mud_list.find("Child").unwrap();

        // Child should have its own alias
        assert!(child.find_alias("c").is_some());

        // Child should inherit parent's alias
        assert!(child.find_alias("p").is_some());
        assert_eq!(child.find_alias("p").unwrap().text, "parent_alias");

        // Child has 1 action (doesn't inherit actions - they're per-MUD)
        assert_eq!(child.action_list.len(), 0);

        // Parent has the action
        let parent = cfg.mud_list.find("Parent").unwrap();
        assert_eq!(parent.action_list.len(), 1);
    }
}
