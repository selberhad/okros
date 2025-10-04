pub type Attrib = u16;

pub struct Scrollback {
    pub width: usize,
    pub height: usize,
    lines: usize,
    pub(crate) buf: Vec<Attrib>,
    canvas_off: usize,
    pub viewpoint: usize,
    pub top_line: usize,
    pub(crate) rows_filled: usize,
    frozen: bool,
    pub(crate) total_lines_written: usize, // Monotonic counter for headless mode
}

impl Scrollback {
    pub fn new(width: usize, height: usize, lines: usize) -> Self {
        Self {
            width,
            height,
            lines,
            buf: vec![0; width * lines],
            canvas_off: 0,
            viewpoint: 0,
            top_line: 0,
            rows_filled: 0,
            frozen: false,
            total_lines_written: 0,
        }
    }
    pub fn set_frozen(&mut self, f: bool) {
        self.frozen = f;
    }

    /// Move viewpoint (C++ OutputWindow::moveViewpoint, lines 65-120)
    /// Returns true if reached boundary (for "quit scrollback" feature)
    pub fn move_viewpoint_lines(&mut self, amount: isize) -> bool {
        let scroll_start = 0;
        let scroll_end = self.canvas_off;

        // C++ lines 100-103: Check boundaries BEFORE moving
        if amount < 0 && self.viewpoint == scroll_start {
            // Already at beginning (C++ line 100-101)
            return false;
        } else if amount > 0 && self.viewpoint == scroll_end {
            // At end - return true to quit scrollback mode (C++ line 102-103)
            return true;
        }

        // Calculate new viewpoint (C++ lines 106-111)
        let new_vp = if amount < 0 {
            self.viewpoint
                .saturating_sub((-amount as usize) * self.width)
        } else {
            self.viewpoint + (amount as usize) * self.width
        };

        // Clamp to valid range
        self.viewpoint = new_vp.clamp(scroll_start, scroll_end);

        false
    }

    /// Page up (C++ move_page_up: height/2)
    pub fn page_up(&mut self) -> bool {
        self.move_viewpoint_lines(-(self.height as isize / 2))
    }

    /// Page down (C++ move_page_down: height/2)
    pub fn page_down(&mut self) -> bool {
        self.move_viewpoint_lines(self.height as isize / 2)
    }

    /// Line up (C++ move_line_up: -1)
    pub fn line_up(&mut self) -> bool {
        self.move_viewpoint_lines(-1)
    }

    /// Line down (C++ move_line_down: 1)
    pub fn line_down(&mut self) -> bool {
        self.move_viewpoint_lines(1)
    }

    /// Home (C++ move_home: to beginning)
    pub fn home(&mut self) -> bool {
        // Return false if already at home (C++ lines 100-101)
        if self.viewpoint == 0 {
            return false;
        }
        self.viewpoint = 0;
        false
    }
    pub fn canvas_ptr(&self) -> usize {
        self.canvas_off
    }
    pub fn print_line(&mut self, bytes: &[u8], color: u8) {
        let screen_span = self.width * self.height;
        let max_canvas = self.width * (self.lines - self.height);
        if self.canvas_off >= max_canvas {
            const COPY: usize = 250;
            let copy = COPY.min(self.lines - self.height);
            let shift = copy * self.width;
            self.buf.copy_within(shift.., 0);
            self.canvas_off -= shift;
            if self.viewpoint >= shift {
                self.viewpoint -= shift
            } else {
                self.viewpoint = 0
            }
            self.top_line += copy;
            let tail = self.buf.len() - shift;
            for a in &mut self.buf[tail..] {
                *a = 0;
            }
        }
        let start = if self.rows_filled < self.height {
            let s = self.viewpoint + self.rows_filled * self.width;
            self.rows_filled += 1;
            s
        } else {
            self.canvas_off += self.width;
            if !self.frozen {
                if self.viewpoint + screen_span < self.canvas_off {
                    self.viewpoint = self.canvas_off - screen_span;
                }
            }
            self.viewpoint + (self.height - 1) * self.width
        };
        for a in &mut self.buf[start..start + self.width] {
            *a = ((color as u16) << 8) | b' ' as u16;
        }
        for (i, b) in bytes.iter().take(self.width).enumerate() {
            self.buf[start + i] = ((color as u16) << 8) | (*b as u16);
        }
        self.total_lines_written += 1; // Increment monotonic counter
    }

    /// Print line with per-character colors (like C++ SET_COLOR stream)
    pub fn print_line_colored(&mut self, pairs: &[(u8, u8)]) {
        let screen_span = self.width * self.height;
        let max_canvas = self.width * (self.lines - self.height);
        if self.canvas_off >= max_canvas {
            const COPY: usize = 250;
            let copy = COPY.min(self.lines - self.height);
            let shift = copy * self.width;
            self.buf.copy_within(shift.., 0);
            self.canvas_off -= shift;
            if self.viewpoint >= shift {
                self.viewpoint -= shift
            } else {
                self.viewpoint = 0
            }
            self.top_line += copy;
            let tail = self.buf.len() - shift;
            for a in &mut self.buf[tail..] {
                *a = 0;
            }
        }
        let start = if self.rows_filled < self.height {
            let s = self.viewpoint + self.rows_filled * self.width;
            self.rows_filled += 1;
            s
        } else {
            self.canvas_off += self.width;
            if !self.frozen {
                if self.viewpoint + screen_span < self.canvas_off {
                    self.viewpoint = self.canvas_off - screen_span;
                }
            }
            self.viewpoint + (self.height - 1) * self.width
        };

        // Fill with spaces first (use default color 0x07)
        for a in &mut self.buf[start..start + self.width] {
            *a = (0x07u16 << 8) | b' ' as u16;
        }

        // Write characters with their individual colors
        for (i, (ch, color)) in pairs.iter().take(self.width).enumerate() {
            self.buf[start + i] = ((*color as u16) << 8) | (*ch as u16);
        }

        self.total_lines_written += 1;
    }
    pub fn viewport_slice(&self) -> &[Attrib] {
        &self.buf[self.viewpoint..self.viewpoint + self.width * self.height]
    }

    /// Get total number of lines written (for testing)
    pub fn total_lines(&self) -> usize {
        self.total_lines_written
    }

    /// Get recent scrollback lines (for headless mode)
    /// Returns last N lines from scrollback, accounting for circular buffer
    pub fn recent_lines(&self, count: usize) -> Vec<Attrib> {
        // How many lines are actually in the buffer
        let lines_in_buffer = self.total_lines_written.min(self.lines);
        let rows_to_return = count.min(lines_in_buffer);

        // Current write position (where the next line would go)
        let current_line = if self.rows_filled < self.height {
            // Still filling initial viewport
            self.rows_filled
        } else {
            // Canvas has scrolled; calculate from canvas_off
            self.canvas_off / self.width
        };

        // Start position for the requested lines (working backwards from current)
        let start_line = if current_line >= rows_to_return {
            current_line - rows_to_return
        } else {
            // Wrap around in circular buffer
            self.lines - (rows_to_return - current_line)
        };

        // Flatten the circular buffer into a linear vec
        let mut result = Vec::with_capacity(rows_to_return * self.width);
        for i in 0..rows_to_return {
            let line_idx = (start_line + i) % self.lines;
            let offset = line_idx * self.width;
            result.extend_from_slice(&self.buf[offset..offset + self.width]);
        }

        result
    }

    pub fn move_viewpoint_page(&mut self, down: bool) {
        let d = (self.height / 2).max(1) * self.width;
        if down {
            self.viewpoint = (self.viewpoint + d).min(self.canvas_off);
        } else {
            self.viewpoint = self.viewpoint.saturating_sub(d);
        }
    }
    pub fn move_viewpoint_line(&mut self, down: bool) {
        let d = self.width;
        if down {
            self.viewpoint = (self.viewpoint + d).min(self.canvas_off);
        } else {
            self.viewpoint = self.viewpoint.saturating_sub(d);
        }
    }
    pub fn highlight_view(&self, line_off: usize, x: usize, len: usize) -> Vec<Attrib> {
        let mut v = self.viewport_slice().to_vec();
        if line_off < self.height && x < self.width {
            let start = line_off * self.width + x;
            let end = (start + len).min(self.height * self.width);
            for a in &mut v[start..end] {
                let ch = *a & 0x00FF;
                let mut color = (((*a) >> 8) as u8) & !(0x80);
                let fg = color & 0x0F;
                let bg = (color & 0xF0) >> 4;
                color = (fg << 4) | bg;
                *a = ((color as u16) << 8) | ch;
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleared_tail() {
        let mut sb = Scrollback::new(5, 2, 10);
        sb.print_line(b"abc", 0x10);
        let v = sb.viewport_slice();
        let bytes: Vec<u8> = v[0..5].iter().map(|a| (*a & 0xFF) as u8).collect();
        assert_eq!(&bytes, b"abc  ");
    }
    #[test]
    fn view_bounds_saturate() {
        let mut sb = Scrollback::new(5, 2, 20);
        for _ in 0..8 {
            sb.print_line(b"aaaaa", 0);
        }
        for _ in 0..10 {
            sb.move_viewpoint_page(false);
        }
        assert_eq!(sb.viewpoint, 0);
        for _ in 0..50 {
            sb.move_viewpoint_page(true);
        }
        assert_eq!(sb.viewpoint, sb.canvas_ptr());
    }
    #[test]
    fn follow_tail_and_freeze() {
        let mut sb = Scrollback::new(4, 2, 16);
        sb.print_line(b"1111", 0);
        sb.print_line(b"2222", 0);
        sb.print_line(b"3333", 0);
        let v = sb.viewport_slice().to_vec();
        let bottom: String =
            String::from_utf8(v[4..8].iter().map(|a| (*a & 0xFF) as u8).collect()).unwrap();
        assert_eq!(bottom, "3333");
        let vp = sb.viewpoint;
        sb.set_frozen(true);
        sb.print_line(b"4444", 0);
        assert_eq!(sb.viewpoint, vp);
    }
    #[test]
    fn page_up_and_down() {
        let mut sb = Scrollback::new(80, 24, 1000);

        // Fill buffer with some lines
        for i in 0..100 {
            let line = format!("Line {}", i);
            sb.print_line(line.as_bytes(), 0x07);
        }

        // Freeze and go up a page
        sb.set_frozen(true);
        let quit = sb.page_up(); // Should move up 12 lines (24/2)
        assert!(!quit); // Not at boundary

        let old_vp = sb.viewpoint;
        sb.page_down(); // Should move down 12 lines
        assert!(sb.viewpoint > old_vp);
    }

    #[test]
    fn line_up_and_down() {
        let mut sb = Scrollback::new(80, 24, 1000);

        for i in 0..50 {
            let line = format!("Line {}", i);
            sb.print_line(line.as_bytes(), 0x07);
        }

        sb.set_frozen(true);
        let old_vp = sb.viewpoint;

        // Move up one line
        sb.line_up();
        assert_eq!(sb.viewpoint, old_vp - 80);

        // Move down one line
        sb.line_down();
        assert_eq!(sb.viewpoint, old_vp);
    }

    #[test]
    fn home_scrollback() {
        let mut sb = Scrollback::new(80, 24, 1000);

        for i in 0..100 {
            let line = format!("Line {}", i);
            sb.print_line(line.as_bytes(), 0x07);
        }

        sb.set_frozen(true);
        sb.home();
        assert_eq!(sb.viewpoint, 0);

        // Home at beginning should return false (already there)
        let at_boundary = sb.home();
        assert!(!at_boundary);
    }

    #[test]
    fn page_down_at_end_returns_quit() {
        let mut sb = Scrollback::new(80, 24, 1000);

        for i in 0..50 {
            let line = format!("Line {}", i);
            sb.print_line(line.as_bytes(), 0x07);
        }

        // Freeze and ensure we're at the very end (canvas_off)
        sb.set_frozen(true);
        sb.viewpoint = sb.canvas_off; // Manually position at end

        // At end, page down should return true (quit scrollback mode)
        let quit = sb.page_down();
        assert!(quit);
    }

    #[test]
    fn highlight_clips() {
        let mut sb = Scrollback::new(3, 2, 6);
        sb.print_line(b"abc", 0x21);
        sb.print_line(b"def", 0x21);
        let v = sb.viewport_slice().to_vec();
        let hl = sb.highlight_view(0, 2, 10);
        assert_eq!(hl.len(), v.len());
        assert_eq!(v[0], hl[0]);
        assert_eq!(v[1], hl[1]);
        for idx in 2..hl.len() {
            assert_ne!((v[idx] >> 8) as u8, (hl[idx] >> 8) as u8);
        }
    }
    #[test]
    fn viewpoint_invariants_under_mixed_moves() {
        let mut sb = Scrollback::new(5, 3, 50);
        for i in 0..40u8 {
            let ch = b'A' + (i % 26);
            sb.print_line(&[ch, ch, ch, ch, ch], 0);
        }
        for i in 0..200 {
            match i % 4 {
                0 => sb.move_viewpoint_line(false),
                1 => sb.move_viewpoint_line(true),
                2 => sb.move_viewpoint_page(false),
                _ => sb.move_viewpoint_page(true),
            }
            assert!(sb.viewpoint <= sb.canvas_ptr());
            let slice = sb.viewport_slice();
            assert_eq!(slice.len(), sb.width * sb.height);
        }
    }
    #[test]
    fn compaction_top_line_increments_by_block() {
        let mut sb = Scrollback::new(4, 2, 8);
        for _ in 0..20 {
            sb.print_line(b"xxxx", 0);
        }
        assert_eq!(sb.top_line % 6, 0);
        assert!(sb.top_line >= 6);
    }
}
