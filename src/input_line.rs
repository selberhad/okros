// InputLine - Bottom input prompt window
//
// Ported from: mcl-cpp-reference/InputLine.cc (MainInputLine subset)
//
// C++ pattern: MainInputLine : public InputLine : public Window
// Rust pattern: InputLine owns Window

use crate::window::Window;

/// InputLine displays user input at bottom of screen
pub struct InputLine {
    pub win: Box<Window>,
    buf: Vec<u8>,
    pub cursor: usize,
    color: u8,
}

impl InputLine {
    /// Create InputLine as child of parent
    pub fn new(parent: *mut Window, width: usize, color: u8) -> Self {
        let mut win = Window::new(parent, width, 1); // height = 1 row
        win.color = color;
        win.clear();

        Self {
            win,
            buf: Vec::new(),
            cursor: 0,
            color,
        }
    }

    pub fn insert(&mut self, b: u8) {
        if self.buf.len() < self.win.width {
            self.buf.insert(self.cursor, b);
            self.cursor += 1;
            self.redraw();
            self.win.dirty = true;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buf.remove(self.cursor);
            self.redraw();
            self.win.dirty = true;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.win.dirty = true; // Cursor moved, need to redraw
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.buf.len() {
            self.cursor += 1;
            self.win.dirty = true; // Cursor moved, need to redraw
        }
    }

    pub fn home(&mut self) {
        self.cursor = 0;
        self.win.dirty = true;
    }

    pub fn end(&mut self) {
        self.cursor = self.buf.len();
        self.win.dirty = true;
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.cursor = 0;
        self.redraw();
        self.win.dirty = true;
    }

    /// Redraw window: fill canvas with input text
    pub fn redraw(&mut self) {
        let width = self.win.width;

        // Fill with spaces in input color
        let blank = ((self.color as u16) << 8) | (b' ' as u16);
        for a in &mut self.win.canvas {
            *a = blank;
        }

        // Write input buffer text
        for (i, b) in self.buf.iter().enumerate().take(width) {
            self.win.canvas[i] = ((self.color as u16) << 8) | (*b as u16);
        }

        // Update cursor position in window
        self.win.cursor_x = self.cursor;
        self.win.cursor_y = 0;
    }

    pub fn take_line(&mut self) -> Vec<u8> {
        let s = self.buf.clone();
        self.clear();
        s
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
    fn edit_and_render() {
        let mut il = InputLine::new(ptr::null_mut(), 10, 0x07);
        il.insert(b'a');
        il.insert(b'b');
        il.insert(b'c');
        il.move_left();
        il.backspace(); // remove 'b'
        let text: Vec<u8> = il.win.canvas.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..2], b"ac");
        assert_eq!(il.cursor, 1);
        let line = il.take_line();
        assert_eq!(line, b"ac");
    }
}
