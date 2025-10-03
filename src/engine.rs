use crate::mccp::Decompressor;
use crate::session::Session;
use crate::scrollback::Attrib;
use std::cell::RefCell;

pub struct SessionEngine<D: Decompressor> {
    pub session: Session<D>,
    attached: bool,
    ansi_cache: RefCell<Option<Vec<String>>>,
    read_cursor: RefCell<usize>,  // Track which lines have been read in headless mode
}

impl<D: Decompressor> SessionEngine<D> {
    pub fn new(decomp: D, width: usize, height: usize, lines: usize) -> Self {
        Self { session: Session::new(decomp, width, height, lines), attached: true, ansi_cache: RefCell::new(None), read_cursor: RefCell::new(0) }
    }

    pub fn detach(&mut self) { self.attached = false; }
    pub fn attach(&mut self) { self.attached = true; }
    pub fn is_attached(&self) -> bool { self.attached }

    pub fn feed_inbound(&mut self, chunk: &[u8]) {
        // Even if detached, we continue processing and buffering into scrollback
        self.session.feed(chunk);
        // Invalidate ANSI cache since buffer changed
        *self.ansi_cache.borrow_mut() = None;
    }

    /// Returns viewport as ANSI-formatted strings (preserves colors)
    /// Uses caching to avoid repeated conversion overhead
    /// NOTE: For TTY mode - use get_scrollback() for headless mode
    pub fn viewport_text(&self) -> Vec<String> {
        // Check cache first
        if let Some(cached) = self.ansi_cache.borrow().as_ref() {
            return cached.clone();
        }

        // Convert Attrib buffer to ANSI strings
        let width = self.session.scrollback.width;
        let height = self.session.scrollback.height;
        let slice = self.session.scrollback.viewport_slice();
        let mut out = Vec::with_capacity(height);

        for row in 0..height {
            let off = row * width;
            let row_slice = &slice[off .. off + width];
            let line = crate::screen::attrib_row_to_ansi(row_slice);
            out.push(line);
        }

        // Cache result
        *self.ansi_cache.borrow_mut() = Some(out.clone());
        out
    }

    /// Returns only NEW lines since last read (for headless mode)
    /// Advances read cursor automatically - won't return same line twice
    pub fn get_new_lines(&self) -> Vec<String> {
        let total_lines_written = self.session.scrollback.total_lines_written;
        let cursor = *self.read_cursor.borrow();

        // No new lines since last read
        if cursor >= total_lines_written {
            return Vec::new();
        }

        // How many new lines since cursor
        let new_line_count = total_lines_written - cursor;

        // Just get the most recent N lines from scrollback using recent_lines
        // This handles wrapping for us
        let lines = self.session.scrollback.recent_lines(new_line_count);
        let width = self.session.scrollback.width;
        let row_count = lines.len() / width;

        let mut out = Vec::with_capacity(row_count);
        for row in 0..row_count {
            let off = row * width;
            let row_slice = &lines[off .. off + width];
            let line = crate::screen::attrib_row_to_ansi(row_slice);
            out.push(line);
        }

        // Advance cursor to current position
        *self.read_cursor.borrow_mut() = total_lines_written;

        // Include current incomplete line if it exists (for prompts without newlines/GA/EOR)
        let current = self.session.current_line();
        if !current.is_empty() {
            out.push(String::from_utf8_lossy(current).to_string());
        }

        out
    }

    /// Peek at recent lines without advancing cursor (for debugging)
    pub fn peek_recent(&self, lines: usize) -> Vec<String> {
        let width = self.session.scrollback.width;
        let slice = self.session.scrollback.recent_lines(lines);
        let row_count = slice.len() / width;
        let mut out = Vec::with_capacity(row_count);

        for row in 0..row_count {
            let off = row * width;
            let row_slice = &slice[off .. off + width];
            let line = crate::screen::attrib_row_to_ansi(row_slice);
            out.push(line);
        }

        // Include current incomplete line (same as get_new_lines)
        let current = self.session.current_line();
        if !current.is_empty() {
            out.push(String::from_utf8_lossy(current).to_string());
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccp::PassthroughDecomp;

    #[test]
    fn engine_detached_buffers_and_attach_reads() {
        let mut eng = SessionEngine::new(PassthroughDecomp::new(), 10, 3, 100);
        eng.detach();
        eng.feed_inbound(b"abc\n");
        assert!(!eng.is_attached());
        eng.attach();
        let rows = eng.viewport_text();
        assert!(rows.iter().any(|r| r.contains("abc")));
    }

    #[test]
    fn engine_viewport_text_preserves_ansi_colors() {
        let mut eng = SessionEngine::new(PassthroughDecomp::new(), 20, 3, 100);
        eng.feed_inbound(b"\x1b[31mRed\x1b[0m\n");
        let rows = eng.viewport_text();
        // Should contain ANSI escape sequences
        assert!(rows.iter().any(|r| r.contains("\x1b[")));
        assert!(rows.iter().any(|r| r.contains("Red")));
    }

    #[test]
    fn engine_ansi_cache_invalidated_on_feed() {
        let mut eng = SessionEngine::new(PassthroughDecomp::new(), 10, 3, 100);
        eng.feed_inbound(b"Line1\n");
        let rows1 = eng.viewport_text();
        let rows2 = eng.viewport_text();
        // Cache hit - should be identical
        assert_eq!(rows1, rows2);

        // Feed new data - should invalidate cache
        eng.feed_inbound(b"Line2\n");
        let rows3 = eng.viewport_text();
        assert_ne!(rows1, rows3);
        assert!(rows3.iter().any(|r| r.contains("Line2")));
    }
}

