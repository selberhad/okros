use crate::action::Action;
use crate::alias::Alias;
use crate::config::Config;
use crate::macro_def::Macro;
use crate::socket::{ConnState, Socket};
use std::io;
use std::net::Ipv4Addr;

/// MUD definition - can be saved/loaded from config file
/// May or may not have an active socket connection
#[derive(Debug)]
pub struct Mud {
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub commands: String, // Auto-execute commands on connect
    pub comment: String,
    pub inherits: Option<Box<Mud>>, // Parent MUD for inheritance
    pub alias_list: Vec<Alias>,
    pub action_list: Vec<Action>,
    pub macro_list: Vec<Macro>,
    // Runtime state (not saved to config, not cloned)
    pub sock: Option<Socket>,
    pub state: ConnState,
    pub loaded: bool, // Have we connected once? (Perl scripts loaded)
}

impl Clone for Mud {
    fn clone(&self) -> Self {
        // NOTE: Runtime state (sock, state, loaded) is not cloned
        Self {
            name: self.name.clone(),
            hostname: self.hostname.clone(),
            port: self.port,
            commands: self.commands.clone(),
            comment: self.comment.clone(),
            inherits: self.inherits.clone(),
            alias_list: self.alias_list.clone(),
            action_list: self.action_list.clone(),
            macro_list: self.macro_list.clone(),
            sock: None,
            state: ConnState::Idle,
            loaded: false,
        }
    }
}

impl Mud {
    /// Create new MUD definition
    pub fn new(name: &str, hostname: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            hostname: hostname.to_string(),
            port,
            commands: String::new(),
            comment: String::new(),
            inherits: None,
            alias_list: Vec::new(),
            action_list: Vec::new(),
            macro_list: Vec::new(),
            sock: None,
            state: ConnState::Idle,
            loaded: false,
        }
    }

    /// Create empty MUD (for current session with no config)
    pub fn empty() -> Self {
        Self::new("", "", 0)
    }

    /// Create MUD with inheritance
    pub fn with_inherits(name: &str, hostname: &str, port: u16, inherits: Option<Mud>) -> Self {
        let mut mud = Self::new(name, hostname, port);
        mud.inherits = inherits.map(Box::new);
        mud
    }

    /// Find alias by name (with inheritance)
    pub fn find_alias(&self, name: &str) -> Option<&Alias> {
        // Check own list first
        if let Some(alias) = self.alias_list.iter().find(|a| a.name == name) {
            return Some(alias);
        }
        // Check parent MUD if not found
        if let Some(ref parent) = self.inherits {
            return parent.find_alias(name);
        }
        None
    }

    /// Find macro by key code (with inheritance)
    pub fn find_macro(&self, key: i32) -> Option<&Macro> {
        // Check own list first
        if let Some(macro_) = self.macro_list.iter().find(|m| m.key == key) {
            return Some(macro_);
        }
        // Check parent MUD if not found
        if let Some(ref parent) = self.inherits {
            return parent.find_macro(key);
        }
        None
    }

    /// Check all actions for trigger matches (C++ Session.cc:640 triggerCheck)
    /// Returns vector of command strings to execute for matching triggers
    pub fn check_action_match(
        &self,
        text: &str,
        interp: &mut dyn crate::plugins::stack::Interpreter,
    ) -> Vec<String> {
        use crate::action::ActionType;

        let mut commands = Vec::new();

        // Check own actions first
        for action in &self.action_list {
            if action.action_type == ActionType::Trigger {
                if let Some(cmd) = action.check_match(text, interp) {
                    commands.push(cmd);
                }
            }
        }

        // Check parent MUD actions
        if let Some(ref parent) = self.inherits {
            commands.extend(parent.check_action_match(text, interp));
        }

        commands
    }

    /// Check all actions for text replacements (C++ Session.cc:640 triggerCheck)
    /// Returns modified text if any replacements matched, None otherwise
    pub fn check_replacement(
        &self,
        text: &str,
        interp: &mut dyn crate::plugins::stack::Interpreter,
    ) -> Option<String> {
        use crate::action::ActionType;

        let mut current = text.to_string();
        let mut modified = false;

        // Check own replacements first
        for action in &self.action_list {
            if action.action_type == ActionType::Replacement
                || action.action_type == ActionType::Gag
            {
                if let Some(replaced) = action.check_replacement(&current, interp) {
                    current = replaced;
                    modified = true;
                }
            }
        }

        // Check parent MUD replacements
        if let Some(ref parent) = self.inherits {
            if let Some(replaced) = parent.check_replacement(&current, interp) {
                current = replaced;
                modified = true;
            }
        }

        if modified {
            Some(current)
        } else {
            None
        }
    }

    /// Connect to this MUD's hostname/port
    pub fn connect(&mut self) -> io::Result<()> {
        if self.hostname.is_empty() || self.port == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "MUD has no hostname/port",
            ));
        }
        // Parse hostname as IPv4 for now (DNS resolution not yet implemented here)
        let ip: Ipv4Addr = self
            .hostname
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid IPv4 address"))?;

        let mut s = Socket::new()?;
        let _ = s.connect_ipv4(ip, self.port);
        self.state = s.state;
        self.sock = Some(s);
        Ok(())
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

/// Collection of MUD definitions
#[derive(Debug, Clone)]
pub struct MudList {
    muds: Vec<Mud>,
}

impl MudList {
    pub fn new() -> Self {
        Self { muds: Vec::new() }
    }

    /// Add a MUD definition to the list
    pub fn insert(&mut self, mud: Mud) {
        self.muds.push(mud);
    }

    /// Find MUD by name
    pub fn find(&self, name: &str) -> Option<&Mud> {
        self.muds.iter().find(|m| m.name == name)
    }

    /// Find MUD by name (mutable)
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Mud> {
        self.muds.iter_mut().find(|m| m.name == name)
    }

    /// Get MUD by index
    pub fn get(&self, index: usize) -> Option<&Mud> {
        self.muds.get(index)
    }

    /// Number of MUDs in list
    pub fn count(&self) -> usize {
        self.muds.len()
    }

    /// Iterate over all MUDs
    pub fn iter(&self) -> impl Iterator<Item = &Mud> {
        self.muds.iter()
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
        let mut m = Mud::empty();
        m.connect_from_config(&cfg).unwrap();
        assert!(matches!(
            m.state,
            ConnState::Connecting | ConnState::Connected
        ));
    }

    #[test]
    fn mud_definition() {
        let mut mud = Mud::new("TestMUD", "127.0.0.1", 4000);
        mud.commands = "look".to_string();
        assert_eq!(mud.name, "TestMUD");
        assert_eq!(mud.hostname, "127.0.0.1");
        assert_eq!(mud.port, 4000);
        assert_eq!(mud.commands, "look");
    }

    #[test]
    fn mud_list() {
        let mut list = MudList::new();
        list.insert(Mud::new("MUD1", "127.0.0.1", 4000));
        list.insert(Mud::new("MUD2", "192.168.1.1", 5000));

        assert_eq!(list.count(), 2);
        assert!(list.find("MUD1").is_some());
        assert!(list.find("MUD2").is_some());
        assert!(list.find("MUD3").is_none());
    }

    #[test]
    fn mud_inheritance() {
        let mut parent = Mud::new("Parent", "127.0.0.1", 4000);
        parent
            .alias_list
            .push(crate::alias::Alias::new("p", "parent command"));

        let mut child = Mud::with_inherits("Child", "192.168.1.1", 5000, Some(parent));
        child
            .alias_list
            .push(crate::alias::Alias::new("c", "child command"));

        // Child can find its own alias
        assert!(child.find_alias("c").is_some());
        // Child can find parent's alias
        assert!(child.find_alias("p").is_some());
        // Child can't find non-existent alias
        assert!(child.find_alias("x").is_none());
    }

    #[test]
    fn mud_find_macro() {
        let mut mud = Mud::new("TestMUD", "127.0.0.1", 4000);
        mud.macro_list
            .push(crate::macro_def::Macro::new(1, "north"));
        mud.macro_list
            .push(crate::macro_def::Macro::new(2, "south"));
        mud.macro_list.push(crate::macro_def::Macro::new(3, "look"));

        // Find existing macros
        assert!(mud.find_macro(1).is_some());
        assert_eq!(mud.find_macro(1).unwrap().text, "north");
        assert!(mud.find_macro(2).is_some());
        assert_eq!(mud.find_macro(2).unwrap().text, "south");
        assert!(mud.find_macro(3).is_some());
        assert_eq!(mud.find_macro(3).unwrap().text, "look");

        // Non-existent key
        assert!(mud.find_macro(99).is_none());
    }

    #[test]
    fn mud_macro_inheritance() {
        let mut parent = Mud::new("Parent", "127.0.0.1", 4000);
        parent
            .macro_list
            .push(crate::macro_def::Macro::new(1, "parent_north"));
        parent
            .macro_list
            .push(crate::macro_def::Macro::new(2, "parent_south"));

        let mut child = Mud::with_inherits("Child", "192.168.1.1", 5000, Some(parent));
        child
            .macro_list
            .push(crate::macro_def::Macro::new(3, "child_look"));

        // Child can find its own macro
        assert!(child.find_macro(3).is_some());
        assert_eq!(child.find_macro(3).unwrap().text, "child_look");

        // Child can find parent's macros
        assert!(child.find_macro(1).is_some());
        assert_eq!(child.find_macro(1).unwrap().text, "parent_north");
        assert!(child.find_macro(2).is_some());
        assert_eq!(child.find_macro(2).unwrap().text, "parent_south");

        // Child can't find non-existent macro
        assert!(child.find_macro(99).is_none());
    }

    #[test]
    fn mud_macro_override() {
        // Test that child macro overrides parent macro with same key
        let mut parent = Mud::new("Parent", "127.0.0.1", 4000);
        parent
            .macro_list
            .push(crate::macro_def::Macro::new(1, "parent_command"));

        let mut child = Mud::with_inherits("Child", "192.168.1.1", 5000, Some(parent));
        child
            .macro_list
            .push(crate::macro_def::Macro::new(1, "child_override"));

        // Child's macro should be found, not parent's
        assert!(child.find_macro(1).is_some());
        assert_eq!(child.find_macro(1).unwrap().text, "child_override");
    }
}
