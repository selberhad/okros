// StatusLine - Top status bar window
//
// Ported from: mcl-cpp-reference/StatusLine.cc
//
// C++ pattern: StatusLine : public Window
// Rust pattern: StatusLine owns Window

use crate::window::Window;

/// StatusLine displays status messages at top of screen
/// Ported from C++ StatusLine.cc:10-59
pub struct StatusLine {
    pub win: Box<Window>,
    text: String,
    color: u8,
}

impl StatusLine {
    /// Create StatusLine as child of parent (C++ StatusLine.cc:10-15)
    pub fn new(parent: *mut Window, width: usize, color: u8) -> Self {
        let mut win = Window::new(parent, width, 1); // height = 1 row
        win.color = color;
        win.clear();

        Self {
            win,
            text: String::new(),
            color,
        }
    }

    /// Set status text and mark dirty (C++ StatusLine.cc:40-48)
    pub fn set_text<S: Into<String>>(&mut self, s: S) {
        self.text = s.into();
        self.redraw();
        self.win.dirty = true;
    }

    /// Redraw window: fill canvas with text (C++ StatusLine.cc:50-59)
    pub fn redraw(&mut self) {
        let width = self.win.width;

        // Fill with spaces in status color
        let blank = ((self.color as u16) << 8) | (b' ' as u16);
        for a in &mut self.win.canvas {
            *a = blank;
        }

        // Write message text
        for (i, b) in self.text.as_bytes().iter().enumerate().take(width) {
            self.win.canvas[i] = ((self.color as u16) << 8) | (*b as u16);
        }
    }

    /// Get mutable window pointer for tree operations
    pub fn window_mut_ptr(&mut self) -> *mut Window {
        self.win.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn set_and_render() {
        let mut sl = StatusLine::new(ptr::null_mut(), 8, 0x07);
        sl.set_text("READY");
        let text: Vec<u8> = sl.win.canvas.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..5], b"READY");
    }
}
