use crate::ansi::{AnsiConverter, AnsiEvent};
use crate::mccp::Decompressor;
use crate::scrollback::Scrollback;
use crate::telnet::TelnetParser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Disconnected,
    Connecting,
    Connected,
}

/// Session statistics tracking (C++ Session.h:44-49)
#[derive(Debug, Clone, Copy, Default)]
pub struct SessionStats {
    pub bytes_written: usize,
    pub bytes_read: usize,
    pub connect_time: i64, // Unix timestamp
    pub dial_time: i64,    // Unix timestamp when connection started
}

/// Trigger callback: receives line text, returns commands to execute
pub type TriggerCallback = Box<dyn FnMut(&str) -> Vec<String> + Send>;

/// Replacement callback: receives line text, returns replacement text or None
pub type ReplacementCallback = Box<dyn FnMut(&str) -> Option<String> + Send>;

/// Prompt callback: receives prompt text (C++ Session::set_prompt calls sys/prompt hook)
/// Returns true to show prompt, false to hide it (opt_showprompt)
pub type PromptCallback = Box<dyn FnMut(&str) -> bool + Send>;

/// Output callback: sys/output hook (C++ Session::triggerCheck line 671)
/// Receives line text, returns modified text or None (None = no change)
pub type OutputCallback = Box<dyn FnMut(&str) -> Option<String> + Send>;

pub struct Session<D: Decompressor> {
    decomp: D,
    telnet: TelnetParser,
    ansi: AnsiConverter,

    // Output target (C++ Session.h:35 Window *window)
    // TTY mode: writes to OutputWindow.sb (single source of truth)
    // Headless/offline: writes to own scrollback
    output_window: *mut crate::output_window::OutputWindow,
    scrollback: Option<Scrollback>, // Only used when output_window is null

    cur_color: u8,
    line_buf: Vec<(u8, u8)>, // (char, color) pairs like C++ SET_COLOR stream
    prompt_events: usize,

    // Session state and statistics (C++ Session.h:27, 44-49)
    pub state: SessionState,
    pub stats: SessionStats,

    // Prompt buffering across reads (C++ Session.h:37 prompt[MAX_MUD_BUF])
    prompt_buffer: Vec<u8>,

    // Optional callbacks for trigger/replacement checking (C++ Session::triggerCheck lines 640-683)
    trigger_callback: Option<TriggerCallback>,
    replacement_callback: Option<ReplacementCallback>,

    // Optional prompt callback (C++ Session::set_prompt lines 361-367, calls sys/prompt hook)
    prompt_callback: Option<PromptCallback>,

    // Optional output hook callback (C++ Session::triggerCheck line 671, sys/output)
    output_callback: Option<OutputCallback>,
}

// SAFETY: Session is used in single-threaded context like C++ MCL
// The raw pointer is only used locally, never shared across threads
unsafe impl<D: Decompressor> Send for Session<D> {}

impl<D: Decompressor> Session<D> {
    /// Create Session with own scrollback (for headless/offline modes)
    pub fn new(decomp: D, width: usize, height: usize, lines: usize) -> Self {
        Self {
            decomp,
            telnet: TelnetParser::new(),
            ansi: AnsiConverter::new(),
            output_window: std::ptr::null_mut(),
            scrollback: Some(Scrollback::new(width, height, lines)),
            cur_color: 0x07,
            line_buf: Vec::new(),
            prompt_events: 0,
            state: SessionState::Disconnected,
            stats: SessionStats::default(),
            prompt_buffer: Vec::new(),
            trigger_callback: None,
            replacement_callback: None,
            prompt_callback: None,
            output_callback: None,
        }
    }

    /// Attach OutputWindow for TTY mode (C++ Session.h:35 Window *window)
    /// Once attached, Session writes directly to OutputWindow.sb instead of own scrollback
    pub fn attach_window(&mut self, window: *mut crate::output_window::OutputWindow) {
        self.output_window = window;
        // Drop own scrollback since we'll use OutputWindow's
        self.scrollback = None;
    }

    /// Write character to output (C++ Session::print → window->print)
    /// TTY mode: writes character-by-character to OutputWindow
    /// Headless mode: buffered line writing to scrollback
    fn print_char(&mut self, ch: u8) {
        if !self.output_window.is_null() {
            // TTY mode - write character immediately like C++ Window::print
            unsafe {
                (*self.output_window).print(&[ch], self.cur_color);
            }
        }
        // Headless mode: characters are buffered in line_buf, written on \n
    }

    /// Set trigger callback (C++ Session has MUD& and calls mud.checkActionMatch)
    pub fn set_trigger_callback(&mut self, callback: TriggerCallback) {
        self.trigger_callback = Some(callback);
    }

    /// Set replacement callback (C++ Session calls mud.checkReplacement)
    pub fn set_replacement_callback(&mut self, callback: ReplacementCallback) {
        self.replacement_callback = Some(callback);
    }

    /// Set prompt callback (C++ Session::set_prompt calls sys/prompt hook, lines 361-367)
    /// Callback receives prompt text, returns true to show, false to hide
    pub fn set_prompt_callback(&mut self, callback: PromptCallback) {
        self.prompt_callback = Some(callback);
    }

    /// Set output hook callback (C++ Session::triggerCheck calls sys/output, line 671)
    /// Callback receives line text, returns modified text or None
    pub fn set_output_callback(&mut self, callback: OutputCallback) {
        self.output_callback = Some(callback);
    }

    pub fn feed(&mut self, chunk: &[u8]) {
        self.decomp.receive(chunk);
        while self.decomp.pending() {
            let out = self.decomp.take_output();
            self.telnet.feed(&out);
            let prompt_count = self.telnet.drain_prompt_events();
            self.prompt_events += prompt_count;
            let app = self.telnet.take_app_out();
            for ev in self.ansi.feed(&app) {
                match ev {
                    AnsiEvent::SetColor(c) => self.cur_color = c,
                    AnsiEvent::Text(b'\n') => {
                        // C++ Session.cc:524-538 - Check triggers on complete line
                        let should_print = self.check_line_triggers();

                        // TTY mode: write newline immediately (C++ Window::print writes char-by-char)
                        // Already written character-by-character above, always visible
                        self.print_char(b'\n');

                        // Headless mode: write buffered line to scrollback (respecting gag)
                        if self.output_window.is_null() && should_print {
                            if let Some(ref mut sb) = self.scrollback {
                                sb.print_line_colored(&self.line_buf);
                            }
                        }

                        self.line_buf.clear();
                    }
                    AnsiEvent::Text(b'\r') => { /* discard \r like C++ Session.cc:541 */ }
                    AnsiEvent::Text(b) => {
                        // Write character immediately (C++ Window::print)
                        self.print_char(b);
                        // Also buffer for trigger checking
                        self.line_buf.push((b, self.cur_color));
                    }
                }
            }
            // Handle prompt events (GA/EOR) with multi-read buffering (C++ Session.cc:455-499, 596-602)
            if prompt_count > 0 {
                self.handle_prompt_event();
            }
        }
    }

    /// Handle prompt event (IAC GA/EOR) with multi-read buffering
    /// C++ Session.cc lines 455-499 (prompt detection) and 596-602 (buffering)
    fn handle_prompt_event(&mut self) {
        // Combine prompt_buffer (from previous reads) + current line_buf
        // C++ lines 479-485: if (prompt[0] || out[0]) { strcat(prompt, out_buf); set_prompt(...) }
        let mut full_prompt = self.prompt_buffer.clone();
        full_prompt.extend(self.line_buf.iter().map(|(ch, _)| *ch));

        let prompt_text = String::from_utf8_lossy(&full_prompt).to_string();

        // Call prompt callback (C++ set_prompt calls sys/prompt hook)
        let should_show = if let Some(ref mut callback) = self.prompt_callback {
            callback(&prompt_text)
        } else {
            true // Default: show prompt
        };

        // Note: Prompt characters were already written via print_char() as they arrived
        // prompt_event (GA/EOR) just signals completion, nothing more to print
        // In headless mode, write the buffered prompt to scrollback
        if should_show && !self.line_buf.is_empty() && self.output_window.is_null() {
            if let Some(ref mut sb) = self.scrollback {
                sb.print_line_colored(&self.line_buf);
            }
        }

        // Clear buffers for next prompt (C++ line 497: prompt[0] = NUL)
        self.prompt_buffer.clear();
        self.line_buf.clear();
    }

    /// Check trigger/replacement callbacks on current line (C++ Session::triggerCheck lines 640-683)
    /// Returns false if line should be gagged (not printed)
    fn check_line_triggers(&mut self) -> bool {
        if self.line_buf.is_empty() {
            return true;
        }

        // Extract plain text by stripping SET_COLOR markers (C++ Session.cc:656-664)
        let mut plain_text: String = self
            .line_buf
            .iter()
            .map(|(ch, _color)| *ch as char)
            .collect();

        // Check replacement first (can modify text)
        if let Some(ref mut callback) = self.replacement_callback {
            if let Some(replacement) = callback(&plain_text) {
                // If empty replacement, this is a gag - return false to skip printing
                if replacement.is_empty() {
                    return false;
                }
                plain_text = replacement.clone();
                // Replace line_buf with new text (preserve colors for now - C++ does full re-processing)
                self.line_buf.clear();
                for ch in replacement.bytes() {
                    self.line_buf.push((ch, self.cur_color));
                }
            }
        }

        // Check triggers (generate commands but don't affect line display)
        if let Some(ref mut callback) = self.trigger_callback {
            let _commands = callback(&plain_text);
            // TODO: Commands should be added to interpreter queue (C++ Session.cc:667)
            // For now, we just call the callback which can handle queueing externally
        }

        // Call sys/output hook (C++ Session.cc:671 - AFTER trigger/replacement)
        if let Some(ref mut callback) = self.output_callback {
            if let Some(modified) = callback(&plain_text) {
                // Hook modified the text (or gagged it)
                if modified.is_empty() {
                    return false; // Gag the line
                }
                // Replace line_buf with modified text
                self.line_buf.clear();
                for ch in modified.bytes() {
                    self.line_buf.push((ch, self.cur_color));
                }
            }
        }

        true // Print the line
    }

    pub fn drain_prompt_events(&mut self) -> usize {
        let n = self.prompt_events;
        self.prompt_events = 0;
        n
    }

    /// Get current incomplete line (not yet terminated by newline or prompt event)
    pub fn current_line(&self) -> Vec<u8> {
        self.line_buf.iter().map(|(ch, _)| *ch).collect()
    }

    /// Get current incomplete line with colors (for rendering)
    pub fn current_line_colored(&self) -> &[(u8, u8)] {
        &self.line_buf
    }

    /// Get scrollback viewport for headless mode
    /// Returns None in TTY mode (use OutputWindow instead)
    pub fn scrollback_viewport(&self) -> Option<&[crate::scrollback::Attrib]> {
        self.scrollback.as_ref().map(|sb| sb.viewport_slice())
    }

    /// Get mutable scrollback for headless mode
    /// Returns None in TTY mode (use OutputWindow instead)
    pub fn scrollback_mut(&mut self) -> Option<&mut Scrollback> {
        self.scrollback.as_mut()
    }

    /// Get immutable scrollback reference for headless mode
    /// Returns None in TTY mode (use OutputWindow instead)
    pub fn scrollback_ref(&self) -> Option<&Scrollback> {
        self.scrollback.as_ref()
    }

    /// Get total lines written to scrollback (for headless mode)
    pub fn total_lines(&self) -> usize {
        self.scrollback
            .as_ref()
            .map(|sb| sb.total_lines())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccp::PassthroughDecomp;

    #[test]
    fn session_pipeline_basic() {
        let mut ses = Session::new(PassthroughDecomp::new(), 5, 2, 20);
        ses.feed(b"Hello\nWorld\n");
        let v = ses.scrollback_viewport().unwrap();
        let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..5], b"Hello");
        assert_eq!(&text[5..10], b"World");
    }

    #[test]
    fn nodeka_menu_colors() {
        // Real Nodeka output with mid-line color changes
        // Line format: [red bg spaces][reset][white text][reset][red bg spaces]\n
        let nodeka_line =
            b"\x1b[41m \x1b[0m \x1b[1;37mWelcome to Nodeka\x1b[0m: \x1b[41m \x1b[0m\n\r";

        let mut ses = Session::new(PassthroughDecomp::new(), 80, 3, 100);
        ses.feed(nodeka_line);

        // Get the stored line
        let v = ses.scrollback_viewport().unwrap();

        // Extract text (should have "Welcome to Nodeka")
        let text: String = v[0..80].iter().map(|a| (a & 0xFF) as u8 as char).collect();

        assert!(
            text.contains("Welcome to Nodeka"),
            "Text should contain 'Welcome to Nodeka', got: {:?}",
            text
        );

        // Check that "Welcome" part has white color (0x87 or 0x07), NOT black-on-black (0x00)
        let welcome_start = text.find('W').expect("Should find 'W'");
        let welcome_color = (v[welcome_start] >> 8) as u8;

        assert_ne!(
            welcome_color & 0x0F,
            0x00,
            "Text color should NOT be black (0x00), got: 0x{:02x}",
            welcome_color
        );

        // NOW test the conversion to ANSI - this is what get_buffer uses
        let ansi_output = crate::screen::attrib_row_to_ansi(&v[0..80]);
        assert!(
            ansi_output.contains("Welcome to Nodeka"),
            "ANSI output should contain 'Welcome to Nodeka', got: {:?}",
            ansi_output
        );
    }
}
