// Command Queue Interpreter - processes user input commands
//
// Ported from: mcl-cpp-reference/Interpreter.cc
//
// C++ pattern: Interpreter class with command queue and expansion logic
// Rust pattern: CommandQueue struct with expansion methods

/// Expansion flags (C++ Interpreter.h:4-12)
pub const EXPAND_NONE: u32 = 0x00;
pub const EXPAND_VARIABLES: u32 = 0x01;
pub const EXPAND_ALIASES: u32 = 0x02;
pub const EXPAND_SEMICOLON: u32 = 0x04;
pub const EXPAND_SPEEDWALK: u32 = 0x08;
pub const EXPAND_ALL: u32 = 0xffff;

/// Default flags for entry from the input line (C++ line 12)
pub const EXPAND_INPUT: u32 = EXPAND_ALIASES | EXPAND_SPEEDWALK;

/// Command queue interpreter (C++ Interpreter class, Interpreter.cc:15, 49-79)
pub struct CommandQueue {
    commands: Vec<String>,
    command_character: char,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            command_character: '#',
        }
    }

    /// Add command to queue with expansion (C++ Interpreter::add, lines 237-274)
    pub fn add(&mut self, s: &str, flags: u32, back: bool) {
        // TODO: Implement full expansion logic
        // For now, just add to queue
        let cmd = s.to_string();

        if back {
            self.commands.insert(0, cmd);
        } else {
            self.commands.push(cmd);
        }
    }

    /// Execute commands in queue (C++ Interpreter::execute, lines 49-79)
    /// Returns commands that should be sent to MUD
    pub fn execute(&mut self) -> Vec<String> {
        let mut result = Vec::new();
        let mut count = 0;

        while !self.commands.is_empty() {
            let line = self.commands.remove(0);

            // Prevent infinite recursion (C++ lines 57-63)
            count += 1;
            if count > 100 {
                eprintln!("Recursing alias? Next command would be \"{}\".", line);
                self.commands.clear();
                break;
            }

            // TODO: Call sys/send hook (C++ line 68)

            // MCL command vs MUD command (C++ lines 71-77)
            if line.starts_with(self.command_character) {
                // TODO: Call mclCommand() - for now skip
                eprintln!("MCL command not yet implemented: {}", line);
            } else {
                // Return command to be sent to MUD
                result.push(line);
            }
        }

        result
    }

    pub fn set_command_character(&mut self, c: char) {
        self.command_character = c;
    }

    pub fn get_command_character(&self) -> char {
        self.command_character
    }

    /// Expand semicolon-separated commands (C++ Interpreter::expandSemicolon, lines 276-319)
    fn expand_semicolon(&mut self, s: &str, flags: u32) {
        if s.contains(';') {
            let mut current = String::new();
            let mut chars = s.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '\\' && chars.peek() == Some(&';') {
                    // Escaped semicolon (C++ lines 286-288)
                    current.push(';');
                    chars.next(); // consume the ';'
                } else if ch == ';' {
                    // Split here (C++ lines 289-312)
                    let trimmed = current.trim_end();
                    self.add(trimmed, EXPAND_ALL, true);
                    current.clear();
                } else {
                    current.push(ch);
                }
            }

            // Add final segment (C++ line 314)
            if !current.is_empty() {
                let trimmed = current.trim();
                self.add(trimmed, flags & !EXPAND_SEMICOLON, true);
            }
        } else {
            // No semicolons, just add with flag cleared (C++ line 318)
            self.add(s, flags & !EXPAND_SEMICOLON, true);
        }
    }

    /// Expand speedwalk notation (C++ Interpreter::expandSpeedwalk, lines 89-150)
    /// Example: "3n2e" -> "n;n;n;e;e"
    fn expand_speedwalk(&mut self, s: &str, flags: u32) {
        // TODO: Implement full speedwalk expansion
        // For now, just pass through without EXPAND_SPEEDWALK flag
        self.add(s, flags & !EXPAND_SPEEDWALK, true);
    }

    /// Expand variable references (C++ Interpreter::expandVariables, lines 152-227)
    /// Example: "%h" -> hostname, "%p" -> port, etc.
    fn expand_variables(&self, s: &str) -> String {
        // TODO: Implement variable expansion
        // For now, just return as-is
        s.to_string()
    }

    /// Expand aliases (C++ Interpreter::expandAliases, lines 322-366)
    fn expand_aliases(&mut self, s: &str, flags: u32) {
        // TODO: Implement alias expansion
        // For now, just pass through without EXPAND_ALIASES flag
        self.add(s, flags & !EXPAND_ALIASES, true);
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_add_and_execute() {
        let mut cq = CommandQueue::new();
        cq.add("north", EXPAND_NONE, false); // back=false for FIFO order
        cq.add("south", EXPAND_NONE, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "north");
        assert_eq!(cmds[1], "south");
    }

    #[test]
    fn prevents_infinite_recursion() {
        let mut cq = CommandQueue::new();
        // Add 200 commands to trigger recursion limit
        for _ in 0..200 {
            cq.add("test", EXPAND_NONE, true);
        }

        let cmds = cq.execute();
        // Should stop at 100 and clear the rest
        assert!(cmds.len() <= 100);
    }

    #[test]
    fn semicolon_expansion() {
        let mut cq = CommandQueue::new();
        cq.expand_semicolon("north;south;east", EXPAND_SEMICOLON);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "east"); // Reverse order due to back=true
        assert_eq!(cmds[1], "south");
        assert_eq!(cmds[2], "north");
    }

    #[test]
    fn escaped_semicolon() {
        let mut cq = CommandQueue::new();
        cq.expand_semicolon("say hello\\;goodbye", EXPAND_SEMICOLON);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "say hello;goodbye");
    }
}
