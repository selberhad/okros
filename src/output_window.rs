use crate::scrollback::{Attrib, Scrollback};
use crate::window::Window;

pub struct OutputWindow {
    pub win: Window,
    pub sb: Scrollback,
    color: u8,
}

impl OutputWindow {
    pub fn new(width: usize, height: usize, lines: usize, color: u8) -> Self {
        let mut win = Window::new(width, height);
        win.clear(color);
        Self {
            win,
            sb: Scrollback::new(width, height, lines),
            color,
        }
    }

    pub fn print_line(&mut self, bytes: &[u8], color: u8) {
        self.sb.print_line(bytes, color);
        let view = self.sb.viewport_slice();
        self.win.blit(view);
    }

    pub fn viewport(&self) -> &[Attrib] {
        &self.win.canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{diff_to_ansi, DiffOptions};

    #[test]
    fn prints_lines_and_renders_diff() {
        let mut ow = OutputWindow::new(5, 2, 20, 0);
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
