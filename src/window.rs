// Window - Base class for UI widgets
//
// Ported from: mcl-cpp-reference/Window.cc (721 lines)
//
// Architecture (C++ comment from Window.cc):
// - Windows form a tree structure via parent/child pointers
// - Each window has its own canvas (attribute buffer)
// - refresh() walks tree: redraw() if dirty, then draw_on_parent() to composite
// - Subclasses override redraw() to render their content

use crate::scrollback::Attrib;
use std::ptr;

/// Window tree node
pub struct Window {
    // Tree structure (C++ Window.cc:10-14)
    pub parent: *mut Window,
    pub next: *mut Window, // Next sibling
    pub prev: *mut Window, // Previous sibling
    pub child_first: *mut Window,
    pub child_last: *mut Window,

    // Geometry
    pub width: usize,
    pub height: usize,
    pub parent_x: isize, // Position in parent
    pub parent_y: isize,

    // Canvas
    pub canvas: Vec<Attrib>,
    clear_line: Vec<Attrib>, // For clearing rows

    // State
    pub visible: bool,
    pub dirty: bool,
    pub color: u8,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub focused: *mut Window,
}

impl Window {
    /// Create new window (C++ Window.cc:10-57)
    pub fn new(parent: *mut Window, width: usize, height: usize) -> Box<Self> {
        let clear_line = vec![((0x07u16) << 8) | (b' ' as u16); width];
        let canvas = vec![((0x07u16) << 8) | (b' ' as u16); width * height];

        let mut win = Box::new(Self {
            parent,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
            child_first: ptr::null_mut(),
            child_last: ptr::null_mut(),
            width,
            height,
            parent_x: 0,
            parent_y: 0,
            canvas,
            clear_line,
            visible: true,
            dirty: true,
            color: 0x07,
            cursor_x: 0,
            cursor_y: 0,
            focused: ptr::null_mut(),
        });

        // Insert into parent's child list (C++ Window.cc:55-56)
        if !parent.is_null() {
            unsafe {
                (*parent).insert(win.as_mut());
            }
        }

        win
    }

    /// Insert child window into linked list (C++ Window.cc:67-82)
    pub fn insert(&mut self, window: *mut Window) {
        unsafe {
            if !self.child_last.is_null() {
                (*self.child_last).next = window;
                (*window).prev = self.child_last;
            } else {
                self.child_first = window;
            }
            self.child_last = window;
        }
    }

    /// Remove child from linked list
    pub fn remove(&mut self, window: *mut Window) {
        unsafe {
            if (*window).prev.is_null() {
                self.child_first = (*window).next;
            } else {
                (*(*window).prev).next = (*window).next;
            }

            if (*window).next.is_null() {
                self.child_last = (*window).prev;
            } else {
                (*(*window).next).prev = (*window).prev;
            }

            (*window).next = ptr::null_mut();
            (*window).prev = ptr::null_mut();
        }
    }

    /// Show/hide window (C++ Window.cc:98-101)
    pub fn show(&mut self, vis: bool) {
        self.visible = vis;
        self.dirty = true;
    }

    /// Clear canvas (C++ Window.cc:342-351)
    pub fn clear(&mut self) {
        let fill = ((self.color as u16) << 8) | (b' ' as u16);
        for a in &mut self.canvas {
            *a = fill;
        }
        self.dirty = true;
    }

    /// Handle keypress (C++ Window.h:33 - virtual bool keypress(int key))
    /// Returns true if the key was handled
    /// Default implementation does nothing
    pub fn keypress(&mut self, _key: i32) -> bool {
        false
    }

    /// Copy source attribs to this canvas at position (C++ Window.cc:280-311)
    pub fn copy(&mut self, source: &[Attrib], w: usize, h: usize, x: isize, y: isize) {
        // Bounds check
        if y >= self.height as isize || x >= self.width as isize {
            return;
        }
        if x + w as isize <= 0 || y + h as isize <= 0 {
            return;
        }

        // Direct copy optimization
        if self.width == w && x == 0 {
            let y_start = if y < 0 {
                let offset = (-y) as usize * w;
                let h_adj = h - (-y) as usize;
                let size = (w * h_adj).min((self.height) * self.width);
                if size > 0 {
                    self.canvas[0..size].copy_from_slice(&source[offset..offset + size]);
                }
                return;
            } else {
                y as usize
            };

            let size = (w * h).min((self.height - y_start) * self.width);
            if size > 0 {
                self.canvas[y_start * self.width..y_start * self.width + size]
                    .copy_from_slice(&source[0..size]);
            }
        } else {
            // Row-by-row copy
            let y_start = 0.max(y) as usize;
            let y_end = self.height.min((y + h as isize) as usize);

            for y2 in y_start..y_end {
                let dst_off = y2 * self.width + x as usize;
                let src_off = (y2 as isize - y) as usize * w;
                let copy_width = w.min(self.width - x as usize);

                self.canvas[dst_off..dst_off + copy_width]
                    .copy_from_slice(&source[src_off..src_off + copy_width]);
            }
        }
    }

    /// Redraw this window's content - override in subclasses (C++ Window.cc:314-317)
    pub fn redraw(&mut self) {
        self.dirty = false;
    }

    /// Refresh window hierarchy (C++ Window.cc:320-350)
    pub fn refresh(&mut self) -> bool {
        let mut refreshed = false;

        // Don't do anything if hidden
        if !self.visible {
            if self.dirty {
                self.dirty = false;
                return true;
            }
            return false;
        }

        // Redraw if dirty
        if self.dirty {
            self.redraw();
            refreshed = true;
        }

        // Refresh children (C++ Window.cc:343-345)
        let mut child = self.child_first;
        while !child.is_null() {
            unsafe {
                refreshed = (*child).refresh() || refreshed;
                child = (*child).next;
            }
        }

        // Copy our canvas to parent (C++ Window.cc:347-348)
        self.draw_on_parent();

        refreshed
    }

    /// Copy this window's canvas onto parent (C++ Window.cc:513-516)
    pub fn draw_on_parent(&mut self) {
        if !self.parent.is_null() {
            unsafe {
                (*self.parent).copy(
                    &self.canvas,
                    self.width,
                    self.height,
                    self.parent_x,
                    self.parent_y,
                );
            }
        }
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.width.saturating_sub(1));
        self.cursor_y = y.min(self.height.saturating_sub(1));
    }

    // Compatibility methods for existing code
    pub fn put_char(&mut self, x: usize, y: usize, ch: u8, color: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let off = y * self.width + x;
        self.canvas[off] = ((color as u16) << 8) | (ch as u16);
        self.dirty = true;
    }

    pub fn clear_line(&mut self, y: usize, color: u8) {
        if y >= self.height {
            return;
        }
        let fill = ((color as u16) << 8) | (b' ' as u16);
        let off = y * self.width;
        for a in &mut self.canvas[off..off + self.width] {
            *a = fill;
        }
        self.dirty = true;
    }

    pub fn blit(&mut self, data: &[Attrib]) {
        if data.len() == self.canvas.len() {
            self.canvas.copy_from_slice(data);
            self.dirty = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_tree() {
        let mut root = Window::new(ptr::null_mut(), 80, 24);
        let mut child = Window::new(root.as_mut(), 40, 12);

        assert_eq!(root.width, 80);
        assert_eq!(child.width, 40);
        assert_eq!(child.parent, root.as_mut() as *mut Window);
    }

    #[test]
    fn window_copy() {
        let mut win = Window::new(ptr::null_mut(), 10, 5);
        win.clear();

        let source = vec![((0x0Fu16) << 8) | (b'X' as u16); 20];
        win.copy(&source, 4, 5, 0, 0);

        // Check first cell copied
        assert_eq!(win.canvas[0] & 0xFF, b'X' as u16);
    }
}
