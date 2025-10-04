// Command Queue Interpreter - processes user input commands
//
// Ported from: mcl-cpp-reference/Interpreter.cc
//
// C++ pattern: Interpreter class with command queue and expansion logic
// Rust pattern: CommandQueue struct with expansion methods

use chrono::{Datelike, Timelike}; // For day(), month(), hour(), minute(), etc.

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
            let expanded = self.expand_variables(s);
            self.add(&expanded, flags & !EXPAND_VARIABLES, back);
        } else if flags & EXPAND_ALIASES != 0 {
            self.expand_aliases(s, flags);
        } else if flags & EXPAND_SPEEDWALK != 0 {
            self.expand_speedwalk(s, flags);
        } else if flags & EXPAND_SEMICOLON != 0 {
            self.expand_semicolon(s, flags);
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
                    // Use back=false to maintain order
                    self.add(trimmed, EXPAND_ALL, false);
                    current.clear();
                } else {
                    current.push(ch);
                }
            }

            // Add final segment (C++ line 314)
            if !current.is_empty() {
                let trimmed = current.trim();
                self.add(trimmed, flags & !EXPAND_SEMICOLON, false);
            }
        } else {
            // No semicolons, just add with flag cleared (C++ line 318)
            self.add(s, flags & !EXPAND_SEMICOLON, false);
        }
    }

    /// Expand speedwalk notation (C++ Interpreter::expandSpeedwalk, lines 89-150)
    /// Example: "3n2e" -> "n;n;n;e;e"
    fn expand_speedwalk(&mut self, s: &str, flags: u32) {
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
        self.add(input, flags & !EXPAND_SPEEDWALK, true);
    }

    /// Expand variable references (C++ Interpreter::expandVariables, lines 152-227)
    /// Example: "%h" -> hostname, "%p" -> port, etc.
    fn expand_variables(&self, s: &str) -> String {
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
                            // hostname - TODO: need current_session reference
                            result.push_str("");
                        }
                        'p' => {
                            // port - TODO: need current_session reference
                            result.push_str("0");
                        }
                        'n' => {
                            // MUD name - TODO: need current_session reference
                            result.push_str("");
                        }
                        'P' => {
                            // local port - TODO: need current_session reference
                            result.push_str("0");
                        }
                        'f' => {
                            // FTP port (mud_port + 6) - TODO: need current_session
                            result.push_str("0");
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
    fn expand_aliases(&mut self, s: &str, flags: u32) {
        // Empty string special case (C++ lines 326-327)
        if s.is_empty() {
            self.add("", EXPAND_NONE, true);
            return;
        }

        // TODO: Call sys/command hook (C++ lines 333-337)

        // Extract alias name (C++ lines 340-347)
        let (_name, _args_start) = if let Some(first_ch) = s.chars().next() {
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

        // TODO: Look up alias in currentSession->mud or globalMUD (C++ lines 353-356)
        // For now, just pass through without alias expansion

        // No alias found - pass through (C++ line 364)
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
        // Commands execute in the order they appear
        assert_eq!(cmds[0], "north");
        assert_eq!(cmds[1], "south");
        assert_eq!(cmds[2], "east");
    }

    #[test]
    fn escaped_semicolon() {
        let mut cq = CommandQueue::new();
        cq.expand_semicolon("say hello\\;goodbye", EXPAND_SEMICOLON);

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
        let result = cq.expand_variables("Time: %H:%m");

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
        let result = cq.expand_variables("100%% complete");

        assert_eq!(result, "100% complete");
    }

    #[test]
    fn variable_expansion_no_variables() {
        let cq = CommandQueue::new();
        let result = cq.expand_variables("no variables here");

        assert_eq!(result, "no variables here");
    }

    #[test]
    fn variable_expansion_month() {
        let cq = CommandQueue::new();
        let result = cq.expand_variables("Month: %M");

        // Should be like "Month: Jan" or "Month: Dec"
        assert!(result.starts_with("Month: "));
        let month = &result[7..];
        let valid_months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        assert!(valid_months.contains(&month));
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
}
