// InputLine - Bottom input prompt window with command history
//
// Ported from: mcl-cpp-reference/InputLine.cc
//
// C++ pattern: MainInputLine : public InputLine : public Window
// Rust pattern: InputLine owns Window and integrates history

use crate::command_queue::{CommandQueue, EXPAND_INPUT, EXPAND_NONE, EXPAND_SEMICOLON};
use crate::history::{HistoryId, HistorySet};
use crate::window::Window;

const MAX_INPUT_BUF: usize = 4096;
const MAX_PROMPT_BUF: usize = 80;

/// InputLine displays user input at bottom of screen (C++ InputLine class, InputLine.cc:199-505)
pub struct InputLine {
    pub win: Box<Window>,

    // Input buffer (C++ lines 27-31)
    input_buf: Vec<u8>,
    cursor_pos: usize, // Where next character will be inserted
    max_pos: usize,    // How many characters in buffer
    left_pos: usize,   // Left edge for horizontal scrolling

    // Prompt (C++ line 28)
    prompt_buf: String,

    // History (C++ lines 35-36)
    history_id: HistoryId,
    history_pos: usize, // For cycling through history

    // Config
    color: u8,
    histwordsize: usize,    // Minimum length to save to history
    expand_semicolon: bool, // Expand semicolons in execute()
    echo_input: bool,       // Echo input to output window
}

impl InputLine {
    /// Create InputLine as child of parent (C++ InputLine::InputLine, lines 199-204)
    pub fn new(parent: *mut Window, width: usize, color: u8, history_id: HistoryId) -> Self {
        let mut win = Window::new(parent, width, 1); // height = 1 row
        win.color = color;
        win.clear();

        Self {
            win,
            input_buf: Vec::new(),
            cursor_pos: 0,
            max_pos: 0,
            left_pos: 0,
            prompt_buf: "mcl>".to_string(), // Default prompt (C++ line 197)
            history_id,
            history_pos: 0,
            color,
            histwordsize: 3,        // C++ opt_histwordsize default
            expand_semicolon: true, // C++ opt_expand_semicolon default
            echo_input: false,      // C++ opt_echoinput default
        }
    }

    /// Set prompt text (C++ InputLine::set_prompt, lines 489-505)
    pub fn set_prompt(&mut self, prompt: &str) {
        // Strip color codes and newlines (C++ lines 493-499)
        let mut result = String::new();
        let mut chars = prompt.chars();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip color code (simplified - C++ checks for SET_COLOR byte)
                chars.next();
            } else if ch == '\n' || ch == '\r' {
                result.push(' ');
            } else if result.len() < MAX_PROMPT_BUF - 1 {
                result.push(ch);
            }
        }

        self.prompt_buf = result;
        self.win.dirty = true;
    }

    /// Set input buffer contents (C++ InputLine::set, lines 212-220)
    pub fn set(&mut self, s: &str) {
        self.input_buf = s.as_bytes().to_vec();
        self.max_pos = self.input_buf.len();
        self.cursor_pos = self.max_pos;
        self.left_pos = 0;
        self.adjust();
        self.win.dirty = true;
    }

    /// Clear input line (C++ sets to empty string)
    pub fn clear(&mut self) {
        self.set("");
    }

    /// Handle keypress (C++ InputLine::keypress, lines 232-431)
    /// Returns true if key was handled
    pub fn keypress(
        &mut self,
        key: i32,
        history: &mut HistorySet,
        command_queue: &mut CommandQueue,
    ) -> bool {
        // TODO: Call embed_interp->run_quietly("keypress", ...) (C++ line 236-250)

        match key {
            // Backspace / Ctrl-H (C++ lines 253-267)
            0x08 | 0x7F => {
                if self.cursor_pos > 0 {
                    if self.cursor_pos == self.max_pos {
                        self.max_pos -= 1;
                        self.cursor_pos -= 1;
                    } else {
                        // In middle of line
                        self.input_buf.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                        self.max_pos -= 1;
                    }
                    self.left_pos = self.left_pos.saturating_sub(1);
                }
            }

            // Ctrl-A: Home (C++ lines 269-271)
            0x01 => {
                self.cursor_pos = 0;
                self.left_pos = 0;
            }

            // Ctrl-C: Save to history but don't execute (C++ lines 272-278)
            0x03 => {
                if self.max_pos > 0 {
                    let text = String::from_utf8_lossy(&self.input_buf[..self.max_pos]);
                    history.add(self.history_id, &text, None);
                    self.set("");
                    // TODO: status->setf("Line added to history but not sent")
                }
            }

            // Ctrl-J / Ctrl-K: Delete to EOL (C++ lines 279-281)
            0x0A | 0x0B => {
                self.max_pos = self.cursor_pos;
            }

            // Escape: Clear line (C++ lines 282-284)
            0x1B => {
                self.set("");
            }

            // Ctrl-E: End (C++ lines 285-288)
            0x05 => {
                self.cursor_pos = self.max_pos;
                self.adjust();
            }

            // Ctrl-U: Delete from BOL to cursor (C++ lines 289-294)
            0x15 => {
                let remaining = self.input_buf.split_off(self.cursor_pos);
                self.input_buf = remaining;
                self.max_pos -= self.cursor_pos;
                self.cursor_pos = 0;
                self.adjust();
            }

            // Ctrl-W: Delete word (C++ lines 295-313)
            0x17 => {
                if self.cursor_pos > 0 {
                    let mut bow = self.cursor_pos - 1;

                    // Skip trailing whitespace
                    while bow > 0 && (self.input_buf[bow] as char).is_whitespace() {
                        bow -= 1;
                    }
                    // Skip word
                    while bow > 0 && !(self.input_buf[bow] as char).is_whitespace() {
                        bow -= 1;
                    }
                    // Don't eat the space
                    if bow > 0 {
                        bow += 1;
                    }

                    // Delete from bow to cursor_pos
                    self.input_buf.drain(bow..self.cursor_pos);
                    self.max_pos -= self.cursor_pos - bow;
                    self.cursor_pos = bow;
                    self.adjust();
                }
            }

            // Delete key: Delete to right (C++ lines 314-321)
            0x14E => {
                // ncurses KEY_DC
                if self.cursor_pos < self.max_pos {
                    self.input_buf.remove(self.cursor_pos);
                    self.max_pos -= 1;
                }
            }

            // Enter: Execute line (C++ lines 322-340)
            0x0D | 0x0A if key == 0x0D => {
                // Get input text
                let text = String::from_utf8_lossy(&self.input_buf[..self.max_pos]).to_string();

                // Save to history if long enough (C++ lines 326-327)
                if text.len() >= self.histwordsize {
                    history.add(self.history_id, &text, None);
                }

                // Reset history cycling (C++ line 329)
                self.history_pos = 0;

                // Clear input line (C++ lines 330-337)
                self.cursor_pos = 0;
                self.max_pos = 0;
                self.left_pos = 0;
                // TODO: move/resize window (C++ lines 335-337)

                // Execute (C++ line 339)
                self.execute(&text, command_queue);
            }

            // Arrow left (C++ lines 358-366)
            0x104 => {
                // ncurses KEY_LEFT
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.left_pos = self.left_pos.saturating_sub(1);
                }
            }

            // Arrow right (C++ lines 367-376)
            0x105 => {
                // ncurses KEY_RIGHT
                if self.cursor_pos < self.max_pos {
                    self.cursor_pos += 1;
                    // Scroll only when approaching right margin (C++ line 373)
                    if self.cursor_pos > 7 * self.win.width / 8 {
                        self.adjust();
                    }
                }
            }

            // Arrow up: History recall (C++ lines 377-407)
            0x103 => {
                // ncurses KEY_UP
                if self.history_id == HistoryId::None {
                    // TODO: status->setf("No history available")
                } else {
                    // Simple cycling mode (C++ lines 398-406)
                    if let Some((s, _ts)) = history.get(self.history_id, self.history_pos + 1) {
                        self.set(s);
                        self.history_pos += 1;
                    } else {
                        // TODO: status->setf("No previous history")
                    }
                }
            }

            // Arrow down: History forward (C++ lines 408-423)
            0x102 => {
                // ncurses KEY_DOWN
                if self.history_id == HistoryId::None {
                    // TODO: status->setf("No history available")
                } else if self.history_pos <= 1 {
                    self.history_pos = 0;
                    self.set("");
                } else if let Some((s, _ts)) = history.get(self.history_id, self.history_pos - 1) {
                    self.set(s);
                    self.history_pos -= 1;
                } else {
                    self.history_pos = 0;
                    self.set("");
                }
            }

            // Normal printable character (C++ lines 342-357)
            ch if ch >= 0x20 && ch < 0x100 => {
                if self.max_pos < MAX_INPUT_BUF - 1 {
                    if self.cursor_pos == self.max_pos {
                        // At EOL
                        self.input_buf.push(ch as u8);
                        self.max_pos += 1;
                        self.cursor_pos += 1;
                    } else {
                        // In middle
                        self.input_buf.insert(self.cursor_pos, ch as u8);
                        self.max_pos += 1;
                        self.cursor_pos += 1;
                    }
                    self.adjust();
                }
            }

            _ => return false, // Unhandled key
        }

        self.win.dirty = true;
        true
    }

    /// Execute command (C++ MainInputLine::execute, lines 512-522)
    fn execute(&mut self, text: &str, command_queue: &mut CommandQueue) {
        // TODO: Call embed_interp->run_quietly("sys/userinput", ...) (C++ line 513)

        // Add to interpreter queue with expansion flags (C++ lines 515-518)
        if self.expand_semicolon {
            command_queue.add(text, EXPAND_INPUT | EXPAND_SEMICOLON, false);
        } else {
            command_queue.add(text, EXPAND_INPUT, false);
        }

        // TODO: Echo input if opt_echoinput (C++ lines 520-521)
        // if self.echo_input {
        //     output->printf("%c>> %s\n", SOFT_CR, text);
        // }
    }

    /// Adjust left_pos for horizontal scrolling (C++ InputLine::adjust, lines 476-487)
    fn adjust(&mut self) {
        // TODO: Handle multiline input (C++ lines 477-482)

        // Single-line scrolling (C++ lines 484-486)
        let prompt_len = self.prompt_buf.len();
        while 1 + prompt_len + self.cursor_pos - self.left_pos >= self.win.width {
            self.left_pos += 1;
        }
    }

    /// Redraw window (C++ InputLine::redraw, lines 433-456)
    pub fn redraw(&mut self) {
        let width = self.win.width;
        let prompt_len = self.prompt_buf.len();

        // Fill with spaces in input color
        let blank = ((self.color as u16) << 8) | (b' ' as u16);
        for a in &mut self.win.canvas {
            *a = blank;
        }

        // Write prompt
        for (i, ch) in self.prompt_buf.bytes().enumerate().take(width) {
            self.win.canvas[i] = ((self.color as u16) << 8) | (ch as u16);
        }

        // Write input buffer (C++ line 448 - show "<" if scrolled)
        let mut x = prompt_len;
        if self.left_pos > 0 && x < width {
            self.win.canvas[x] = ((self.color as u16) << 8) | (b'<' as u16);
            x += 1;
        }

        // Write visible portion of input
        for i in self.left_pos..self.max_pos {
            if x >= width {
                break;
            }
            self.win.canvas[x] = ((self.color as u16) << 8) | (self.input_buf[i] as u16);
            x += 1;
        }

        // Update cursor position (C++ lines 450-451)
        let cursor_offset = if self.left_pos > 0 { 1 } else { 0 };
        self.win.cursor_x = prompt_len + cursor_offset + self.cursor_pos - self.left_pos;
        self.win.cursor_y = 0;

        self.win.dirty = false;
    }

    /// Get mutable window pointer for tree operations
    pub fn window_mut_ptr(&mut self) -> *mut Window {
        self.win.as_mut()
    }

    // Config setters
    pub fn set_histwordsize(&mut self, size: usize) {
        self.histwordsize = size;
    }

    pub fn set_expand_semicolon(&mut self, enabled: bool) {
        self.expand_semicolon = enabled;
    }

    pub fn set_echo_input(&mut self, enabled: bool) {
        self.echo_input = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn basic_editing() {
        let mut il = InputLine::new(ptr::null_mut(), 80, 0x07, HistoryId::None);
        let mut hist = HistorySet::new(10);
        let mut cq = CommandQueue::new();

        // Type "hello"
        il.keypress('h' as i32, &mut hist, &mut cq);
        il.keypress('e' as i32, &mut hist, &mut cq);
        il.keypress('l' as i32, &mut hist, &mut cq);
        il.keypress('l' as i32, &mut hist, &mut cq);
        il.keypress('o' as i32, &mut hist, &mut cq);

        assert_eq!(il.max_pos, 5);
        assert_eq!(&il.input_buf[..5], b"hello");
    }

    #[test]
    fn backspace() {
        let mut il = InputLine::new(ptr::null_mut(), 80, 0x07, HistoryId::None);
        let mut hist = HistorySet::new(10);
        let mut cq = CommandQueue::new();

        il.keypress('a' as i32, &mut hist, &mut cq);
        il.keypress('b' as i32, &mut hist, &mut cq);
        il.keypress(0x7F, &mut hist, &mut cq); // Backspace

        assert_eq!(il.max_pos, 1);
        assert_eq!(&il.input_buf[..1], b"a");
    }

    #[test]
    fn ctrl_a_and_ctrl_e() {
        let mut il = InputLine::new(ptr::null_mut(), 80, 0x07, HistoryId::None);
        let mut hist = HistorySet::new(10);
        let mut cq = CommandQueue::new();

        il.set("hello");
        il.keypress(0x01, &mut hist, &mut cq); // Ctrl-A
        assert_eq!(il.cursor_pos, 0);

        il.keypress(0x05, &mut hist, &mut cq); // Ctrl-E
        assert_eq!(il.cursor_pos, 5);
    }

    #[test]
    fn history_cycling() {
        let mut il = InputLine::new(ptr::null_mut(), 80, 0x07, HistoryId::MainInput);
        let mut hist = HistorySet::new(10);
        let mut cq = CommandQueue::new();

        // Add some history manually
        hist.add(HistoryId::MainInput, "first", None);
        hist.add(HistoryId::MainInput, "second", None);

        // Press up arrow twice
        il.keypress(0x103, &mut hist, &mut cq); // Up
        assert_eq!(&il.input_buf[..il.max_pos], b"second");

        il.keypress(0x103, &mut hist, &mut cq); // Up
        assert_eq!(&il.input_buf[..il.max_pos], b"first");

        // Press down arrow
        il.keypress(0x102, &mut hist, &mut cq); // Down
        assert_eq!(&il.input_buf[..il.max_pos], b"second");
    }
}
