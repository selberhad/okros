use crate::scrollback::{Attrib, Scrollback};
use crate::window::Window;

/// OutputWindow - Window that displays scrollback buffer
/// Ported from: mcl-cpp-reference/OutputWindow.cc
///
/// C++ pattern: OutputWindow : public Window, overrides scroll()
/// Rust pattern: OutputWindow owns Window, implements redraw()
pub struct OutputWindow {
    pub win: Box<Window>,
    pub sb: Scrollback,
    color: u8,
}

impl OutputWindow {
    /// Create OutputWindow as child of parent (C++ OutputWindow.cc:8-19)
    pub fn new(parent: *mut Window, width: usize, height: usize, lines: usize, color: u8) -> Self {
        let mut win = Window::new(parent, width, height);
        win.color = color;
        win.clear();

        Self {
            win,
            sb: Scrollback::new(width, height, lines),
            color,
        }
    }

    /// Print line to scrollback and mark dirty (C++ OutputWindow prints to canvas)
    pub fn print_line(&mut self, bytes: &[u8], color: u8) {
        self.sb.print_line(bytes, color);
        self.redraw();
    }

    /// Redraw window: blit scrollback viewport to canvas (C++ Window::redraw pattern)
    pub fn redraw(&mut self) {
        let view = self.sb.viewport_slice();
        self.win.blit(view);
    }

    /// Get viewport for direct rendering
    pub fn viewport(&self) -> &[Attrib] {
        &self.win.canvas
    }

    /// Get mutable window pointer for tree operations
    pub fn window_mut_ptr(&mut self) -> *mut Window {
        self.win.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{diff_to_ansi, DiffOptions};

    #[test]
    fn prints_lines_and_renders_diff() {
        use std::ptr;

        let mut ow = OutputWindow::new(ptr::null_mut(), 5, 2, 20, 0);
        ow.print_line(b"hello", 0);
        ow.print_line(b"world", 0);
        let v = ow.viewport();
        let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..5], b"hello");
        assert_eq!(&text[5..10], b"world");
        // Render diff from blank to current
        let prev = vec![0u16; v.len()];
        let s = diff_to_ansi(
            &prev,
            v,
            &DiffOptions {
                width: 5,
                height: 2,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(s.contains("hello"));
        // bottom-right cell is skipped by renderer, so only 'worl' present
        assert!(s.contains("worl"));
    }
}
