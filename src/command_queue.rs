// Command Queue Interpreter - processes user input commands
//
// Ported from: mcl-cpp-reference/Interpreter.cc
//
// C++ pattern: Interpreter class with command queue and expansion logic
// Rust pattern: CommandQueue struct with expansion methods

use chrono::{Datelike, Timelike}; // For day(), month(), hour(), minute(), etc.

/// Session context for variable expansion
pub struct SessionContext {
    pub hostname: String,
    pub port: u16,
    pub name: String,
    pub local_port: u16,
}

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
    speedwalk_enabled: bool,
    speedwalk_character: char,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            command_character: '#',
            speedwalk_enabled: true,  // C++ opt_speedwalk default
            speedwalk_character: '/', // C++ opt_speedwalk_character default
        }
    }

    /// Add command to queue with expansion (C++ Interpreter::add, lines 237-274)
    pub fn add(&mut self, s: &str, flags: u32, back: bool) {
        self.add_with_context(s, flags, back, None, None);
    }

    /// Add command with optional context for variable/alias expansion
    pub fn add_with_context(
        &mut self,
        s: &str,
        flags: u32,
        back: bool,
        session: Option<&SessionContext>,
        mud: Option<&crate::mud::Mud>,
    ) {
        // Escape character short circuit (C++ lines 238-244)
        if !s.is_empty() && s.starts_with('\\') {
            let cmd = s[1..].to_string();
            if back {
                self.commands.insert(0, cmd);
            } else {
                self.commands.push(cmd);
            }
            return;
        }

        // Expansion pipeline (C++ lines 247-273)
        if flags & EXPAND_VARIABLES != 0 {
            let expanded = self.expand_variables(s, session);
            self.add_with_context(&expanded, flags & !EXPAND_VARIABLES, back, session, mud);
        } else if flags & EXPAND_ALIASES != 0 {
            self.expand_aliases(s, flags, session, mud);
        } else if flags & EXPAND_SPEEDWALK != 0 {
            self.expand_speedwalk(s, flags, session, mud);
        } else if flags & EXPAND_SEMICOLON != 0 {
            self.expand_semicolon(s, flags, session, mud);
        } else {
            // No more expansion - add to queue
            if back {
                self.commands.insert(0, s.to_string());
            } else {
                self.commands.push(s.to_string());
            }
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
    fn expand_semicolon(
        &mut self,
        s: &str,
        flags: u32,
        session: Option<&SessionContext>,
        mud: Option<&crate::mud::Mud>,
    ) {
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
                    // Use back=false to maintain order, pass through flags minus SEMICOLON
                    self.add_with_context(trimmed, flags & !EXPAND_SEMICOLON, false, session, mud);
                    current.clear();
                } else {
                    current.push(ch);
                }
            }

            // Add final segment (C++ line 314)
            if !current.is_empty() {
                let trimmed = current.trim();
                self.add_with_context(trimmed, flags & !EXPAND_SEMICOLON, false, session, mud);
            }
        } else {
            // No semicolons, just add with flag cleared (C++ line 318)
            self.add_with_context(s, flags & !EXPAND_SEMICOLON, false, session, mud);
        }
    }

    /// Expand speedwalk notation (C++ Interpreter::expandSpeedwalk, lines 89-150)
    /// Example: "3n2e" -> "n;n;n;e;e"
    fn expand_speedwalk(
        &mut self,
        s: &str,
        flags: u32,
        session: Option<&SessionContext>,
        mud: Option<&crate::mud::Mud>,
    ) {
        const LEGAL_STANDARD: &str = "nsewud";
        const LEGAL_EXTENDED: &str = "nsewudhjkl";
        const MAX_SPEEDWALK_REPEAT: usize = 99;

        let mut input = s;
        let mut try_speedwalk = self.speedwalk_enabled;
        let legal_speedwalk;

        // Check for speedwalk character prefix (C++ lines 95-98)
        if !input.is_empty() && input.chars().next().unwrap() == self.speedwalk_character {
            try_speedwalk = true;
            legal_speedwalk = LEGAL_EXTENDED;
            input = &input[1..];
        } else {
            legal_speedwalk = LEGAL_STANDARD;
        }

        if try_speedwalk {
            // Validate string contains only digits and legal directions (C++ lines 104-106)
            let is_speedwalk = input.chars().all(|c| c.is_ascii_digit() || legal_speedwalk.contains(c))
                && !input.is_empty()
                && !input.eq_ignore_ascii_case("news") // Hardcoded exception (C++ line 109)
                && legal_speedwalk.contains(input.chars().last().unwrap()); // Must end with direction

            if is_speedwalk {
                // Parse speedwalk string (C++ lines 111-144)
                let mut repeat = 0;
                let chars: Vec<char> = input.chars().collect();

                for &ch in &chars {
                    if ch.is_ascii_digit() {
                        repeat = repeat * 10 + (ch as usize - '0' as usize);
                    } else {
                        // Direction character - expand with repeat count
                        repeat = repeat.clamp(1, MAX_SPEEDWALK_REPEAT);

                        // Expand direction (C++ lines 125-140)
                        for _ in 0..repeat {
                            let dir_str;
                            let dir = match ch {
                                'h' => "nw",
                                'j' => "ne",
                                'k' => "sw",
                                'l' => "se",
                                _ => {
                                    // Standard direction - single character
                                    dir_str = ch.to_string();
                                    &dir_str
                                }
                            };
                            // Use back=false to maintain order (append to end)
                            self.add(dir, EXPAND_NONE, false);
                        }
                        repeat = 0;
                    }
                }
                return;
            }
        }

        // Not a speedwalk - pass through without flag (C++ line 149)
        self.add_with_context(input, flags & !EXPAND_SPEEDWALK, false, session, mud);
    }

    /// Expand variable references (C++ Interpreter::expandVariables, lines 152-227)
    /// Example: "%h" -> hostname, "%p" -> port, etc.
    fn expand_variables(&self, s: &str, session: Option<&SessionContext>) -> String {
        // Quick check - no % means no variables (C++ lines 153-154)
        if !s.contains('%') {
            return s.to_string();
        }

        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next_ch) = chars.peek() {
                    chars.next(); // consume next char
                    match next_ch {
                        // Session variables (C++ lines 168-186)
                        'h' => {
                            // hostname
                            if let Some(sess) = session {
                                result.push_str(&sess.hostname);
                            }
                        }
                        'p' => {
                            // port
                            if let Some(sess) = session {
                                result.push_str(&sess.port.to_string());
                            } else {
                                result.push_str("0");
                            }
                        }
                        'n' => {
                            // MUD name
                            if let Some(sess) = session {
                                result.push_str(&sess.name);
                            }
                        }
                        'P' => {
                            // local port
                            if let Some(sess) = session {
                                result.push_str(&sess.local_port.to_string());
                            } else {
                                result.push_str("0");
                            }
                        }
                        'f' => {
                            // FTP port (mud_port + 6)
                            if let Some(sess) = session {
                                result.push_str(&(sess.port + 6).to_string());
                            } else {
                                result.push_str("0");
                            }
                        }

                        // Time variables using strftime (C++ lines 190-207)
                        'H' => {
                            // Hour (00-23)
                            use std::time::SystemTime;
                            let now = SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            let tm =
                                chrono::NaiveDateTime::from_timestamp_opt(now as i64, 0).unwrap();
                            result.push_str(&format!("{:02}", tm.hour()));
                        }
                        'm' => {
                            // Minute (00-59)
                            use std::time::SystemTime;
                            let now = SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            let tm =
                                chrono::NaiveDateTime::from_timestamp_opt(now as i64, 0).unwrap();
                            result.push_str(&format!("{:02}", tm.minute()));
                        }
                        'M' => {
                            // Month name abbreviated (Jan, Feb, etc.)
                            use std::time::SystemTime;
                            let now = SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            let tm =
                                chrono::NaiveDateTime::from_timestamp_opt(now as i64, 0).unwrap();
                            let month = tm.month();
                            let months = [
                                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep",
                                "Oct", "Nov", "Dec",
                            ];
                            result.push_str(months[(month - 1) as usize]);
                        }
                        'd' => {
                            // Day of month (01-31)
                            use std::time::SystemTime;
                            let now = SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            let tm =
                                chrono::NaiveDateTime::from_timestamp_opt(now as i64, 0).unwrap();
                            result.push_str(&format!("{:02}", tm.day()));
                        }

                        // Literal % (C++ lines 209-211)
                        '%' => result.push('%'),

                        // Unknown - just output the character (C++ lines 215-216)
                        _ => result.push(next_ch),
                    }
                } else {
                    // % at end of string
                    result.push('%');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Expand aliases (C++ Interpreter::expandAliases, lines 322-366)
    fn expand_aliases(
        &mut self,
        s: &str,
        flags: u32,
        session: Option<&SessionContext>,
        mud: Option<&crate::mud::Mud>,
    ) {
        // Empty string special case (C++ lines 326-327)
        if s.is_empty() {
            self.add("", EXPAND_NONE, false);
            return;
        }

        // TODO: Call sys/command hook (C++ lines 333-337)

        // Extract alias name and arguments (C++ lines 340-347)
        let (name, args_start) = if let Some(first_ch) = s.chars().next() {
            if !first_ch.is_alphabetic() {
                // Non-alphabetic first char - single char alias (C++ lines 341-345)
                let name = &s[..first_ch.len_utf8()];
                (name, first_ch.len_utf8())
            } else {
                // Find first whitespace (C++ line 347)
                if let Some(pos) = s.find(char::is_whitespace) {
                    (&s[..pos], pos)
                } else {
                    (s, s.len())
                }
            }
        } else {
            ("", 0)
        };

        // Look up alias in MUD (C++ lines 353-356)
        if let Some(mud_ref) = mud {
            if let Some(alias) = mud_ref.find_alias(name) {
                // Found alias - expand it (C++ lines 358-361)
                let args = if args_start < s.len() {
                    s[args_start..].trim_start()
                } else {
                    ""
                };
                let expanded = alias.expand(args);
                // Expand everything again (C++ line 361)
                self.add_with_context(&expanded, EXPAND_ALL, false, session, mud);
                return;
            }
        }

        // No alias found - pass through (C++ line 364)
        self.add_with_context(s, flags & !EXPAND_ALIASES, false, session, mud);
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
        cq.add("north;south;east", EXPAND_SEMICOLON, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 3);
        // Commands execute in the order they appear
        assert_eq!(cmds[0], "north");
        assert_eq!(cmds[1], "south");
        assert_eq!(cmds[2], "east");
    }

    #[test]
    fn escaped_semicolon() {
        let mut cq = CommandQueue::new();
        cq.add("say hello\\;goodbye", EXPAND_SEMICOLON, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "say hello;goodbye");
    }

    #[test]
    fn speedwalk_simple() {
        let mut cq = CommandQueue::new();
        cq.add("3n2e", EXPAND_SPEEDWALK, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 5);
        assert_eq!(cmds[0], "n");
        assert_eq!(cmds[1], "n");
        assert_eq!(cmds[2], "n");
        assert_eq!(cmds[3], "e");
        assert_eq!(cmds[4], "e");
    }

    #[test]
    fn speedwalk_extended_diagonal() {
        let mut cq = CommandQueue::new();
        cq.add("/2h", EXPAND_SPEEDWALK, false); // h = nw

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "nw");
        assert_eq!(cmds[1], "nw");
    }

    #[test]
    fn speedwalk_mixed_directions() {
        let mut cq = CommandQueue::new();
        cq.add("2n1s3u", EXPAND_SPEEDWALK, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 6);
        assert_eq!(cmds[0], "n");
        assert_eq!(cmds[1], "n");
        assert_eq!(cmds[2], "s");
        assert_eq!(cmds[3], "u");
        assert_eq!(cmds[4], "u");
        assert_eq!(cmds[5], "u");
    }

    #[test]
    fn speedwalk_news_exception() {
        let mut cq = CommandQueue::new();
        cq.add("news", EXPAND_SPEEDWALK, false);

        let cmds = cq.execute();
        // "news" should NOT be expanded as speedwalk
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "news");
    }

    #[test]
    fn speedwalk_invalid_not_expanded() {
        let mut cq = CommandQueue::new();
        cq.add("3hello", EXPAND_SPEEDWALK, false); // Not all direction chars

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "3hello");
    }

    #[test]
    fn variable_expansion_time() {
        let cq = CommandQueue::new();
        let result = cq.expand_variables("Time: %H:%m", None);

        // Should have format like "Time: 14:30"
        assert!(result.starts_with("Time: "));
        assert!(result.contains(":"));
        // Should be digits
        let parts: Vec<&str> = result[6..].split(':').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<u32>().is_ok());
        assert!(parts[1].parse::<u32>().is_ok());
    }

    #[test]
    fn variable_expansion_literal_percent() {
        let cq = CommandQueue::new();
        let result = cq.expand_variables("100%% complete", None);

        assert_eq!(result, "100% complete");
    }

    #[test]
    fn variable_expansion_no_variables() {
        let cq = CommandQueue::new();
        let result = cq.expand_variables("no variables here", None);

        assert_eq!(result, "no variables here");
    }

    #[test]
    fn variable_expansion_month() {
        let cq = CommandQueue::new();
        let result = cq.expand_variables("Month: %M", None);

        // Should be like "Month: Jan" or "Month: Dec"
        assert!(result.starts_with("Month: "));
        let month = &result[7..];
        let valid_months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        assert!(valid_months.contains(&month));
    }

    #[test]
    fn variable_expansion_with_session() {
        use super::SessionContext;

        let cq = CommandQueue::new();
        let session = SessionContext {
            hostname: "mud.example.com".to_string(),
            port: 4000,
            name: "TestMUD".to_string(),
            local_port: 12345,
        };

        let result = cq.expand_variables("Connecting to %h:%p (%n)", Some(&session));
        assert_eq!(result, "Connecting to mud.example.com:4000 (TestMUD)");

        let result2 = cq.expand_variables("FTP port: %f", Some(&session));
        assert_eq!(result2, "FTP port: 4006");
    }

    #[test]
    fn alias_expansion_with_mud() {
        use crate::alias::Alias;
        use crate::mud::Mud;

        let mut cq = CommandQueue::new();
        let mut mud = Mud::empty();

        // Add alias: "n" -> "north"
        mud.alias_list.push(Alias::new("n", "north"));

        // Test alias expansion
        cq.add_with_context("n", EXPAND_ALIASES, false, None, Some(&mud));

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "north");
    }

    #[test]
    fn escape_character_bypass() {
        let mut cq = CommandQueue::new();
        cq.add("\\3n2e", EXPAND_ALL, false);

        let cmds = cq.execute();
        // Should NOT expand speedwalk, just remove escape char
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "3n2e");
    }

    #[test]
    fn full_expansion_pipeline() {
        let mut cq = CommandQueue::new();
        // Variables, then speedwalk (if variables expand to valid speedwalk)
        cq.add(
            "north;south;east",
            EXPAND_VARIABLES | EXPAND_SEMICOLON,
            false,
        );

        let cmds = cq.execute();
        // Should expand semicolons
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "north");
        assert_eq!(cmds[1], "south");
        assert_eq!(cmds[2], "east");
    }

    #[test]
    fn speedwalk_with_zero_prefix() {
        let mut cq = CommandQueue::new();
        cq.add("0n", EXPAND_SPEEDWALK, false);

        let cmds = cq.execute();
        // 0 should be treated as 1 (C++ line 118: max(1, repeat))
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "n");
    }

    #[test]
    fn speedwalk_all_directions() {
        let mut cq = CommandQueue::new();
        cq.add("nsewud", EXPAND_SPEEDWALK, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 6);
        assert_eq!(cmds[0], "n");
        assert_eq!(cmds[1], "s");
        assert_eq!(cmds[2], "e");
        assert_eq!(cmds[3], "w");
        assert_eq!(cmds[4], "u");
        assert_eq!(cmds[5], "d");
    }

    #[test]
    fn speedwalk_extended_all_diagonals() {
        let mut cq = CommandQueue::new();
        cq.add("/hjkl", EXPAND_SPEEDWALK, false);

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 4);
        assert_eq!(cmds[0], "nw");
        assert_eq!(cmds[1], "ne");
        assert_eq!(cmds[2], "sw");
        assert_eq!(cmds[3], "se");
    }

    #[test]
    fn speedwalk_max_repeat_capped() {
        let mut cq = CommandQueue::new();
        cq.add("150n", EXPAND_SPEEDWALK, false); // 150 > 99 max

        let cmds = cq.execute();
        // Should be capped at 99
        assert_eq!(cmds.len(), 99);
    }

    #[test]
    fn alias_with_arguments() {
        use crate::alias::Alias;
        use crate::mud::Mud;

        let mut cq = CommandQueue::new();
        let mut mud = Mud::empty();

        // Alias with arguments: "t" -> "tell %1"
        mud.alias_list.push(Alias::new("t", "tell %1"));

        cq.add_with_context("t bob", EXPAND_ALIASES, false, None, Some(&mud));

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "tell bob");
    }

    #[test]
    fn alias_recursive_expansion() {
        use crate::alias::Alias;
        use crate::mud::Mud;

        let mut cq = CommandQueue::new();
        let mut mud = Mud::empty();

        // Alias that expands to another command with semicolons
        mud.alias_list.push(Alias::new("greet", "say hello;bow"));

        cq.add_with_context("greet", EXPAND_ALL, false, None, Some(&mud));

        let cmds = cq.execute();
        // Should expand alias, then expand semicolons
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "say hello");
        assert_eq!(cmds[1], "bow");
    }

    #[test]
    fn variable_expansion_all_session_vars() {
        use super::SessionContext;

        let cq = CommandQueue::new();
        let session = SessionContext {
            hostname: "mud.test.com".to_string(),
            port: 4000,
            name: "TestMUD".to_string(),
            local_port: 54321,
        };

        let result = cq.expand_variables("Host:%h Port:%p Name:%n Local:%P FTP:%f", Some(&session));
        assert_eq!(
            result,
            "Host:mud.test.com Port:4000 Name:TestMUD Local:54321 FTP:4006"
        );
    }

    #[test]
    fn semicolon_then_speedwalk() {
        // Test that semicolon splits first, then each part is expanded
        let mut cq = CommandQueue::new();

        // Split on semicolons first
        cq.add("3n;look", EXPAND_SEMICOLON, false);
        let cmds = cq.execute();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "3n"); // Not expanded yet
        assert_eq!(cmds[1], "look");

        // Now test with both flags - speedwalk is checked first in pipeline
        let mut cq2 = CommandQueue::new();
        cq2.add("3n2e", EXPAND_SPEEDWALK, false);
        let cmds2 = cq2.execute();
        assert_eq!(cmds2.len(), 5);
    }

    #[test]
    fn complex_expansion_chain() {
        use super::SessionContext;
        use crate::alias::Alias;
        use crate::mud::Mud;

        let mut cq = CommandQueue::new();
        let mut mud = Mud::empty();
        let session = SessionContext {
            hostname: "game.com".to_string(),
            port: 5000,
            name: "Game".to_string(),
            local_port: 12345,
        };

        // Alias that uses variables
        mud.alias_list.push(Alias::new("conn", "connect %h %p"));

        cq.add_with_context("conn", EXPAND_ALL, false, Some(&session), Some(&mud));

        let cmds = cq.execute();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "connect game.com 5000");
    }
}
