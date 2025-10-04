// Screen - Top-level window that renders to terminal
//
// Ported from: mcl-cpp-reference/Screen.cc
//
// C++ inheritance: Screen : public Window
// Rust pattern: Screen owns a Window, delegates to it

use crate::curses::AcsCaps;
use crate::screen::{diff_to_ansi, DiffOptions};
use crate::scrollback::Attrib;
use crate::window::Window;
use std::io::{self, Write};
use std::ptr;

/// Screen - Root window that renders to physical terminal (C++ Screen.cc:39-69)
pub struct Screen {
    pub window: Box<Window>,
    last_screen: Vec<Attrib>,
    scr_x: usize, // Scrolling region
    scr_y: usize,
    scr_w: usize,
    scr_h: usize,
    using_virtual: bool, // /dev/vcsa vs TTY (always false on macOS)
}

impl Screen {
    /// Create new screen with terminal dimensions (C++ Screen.cc:39-69)
    pub fn new(width: usize, height: usize) -> Self {
        let mut window = Window::new(ptr::null_mut(), width, height);
        window.color = 0x07;
        window.clear();

        // TTY mode (macOS/non-Linux) - C++ Screen.cc:52-59
        let last_screen = vec![0u16; width * height];

        Self {
            window,
            last_screen,
            scr_x: 0,
            scr_y: 0,
            scr_w: 0,
            scr_h: 0,
            using_virtual: false,
        }
    }

    /// Set scrolling region (C++ Screen.h setScrollingRegion)
    pub fn set_scrolling_region(&mut self, x: usize, y: usize, w: usize, h: usize) {
        self.scr_x = x;
        self.scr_y = y;
        self.scr_w = w;
        self.scr_h = h;
    }

    /// Refresh screen: Window::refresh() then refreshTTY() (C++ Screen.cc:105-110)
    pub fn refresh(&mut self, caps: &AcsCaps) -> bool {
        // Call Window::refresh() to composite tree (C++ Screen.cc:84)
        self.window.refresh();

        // Always render to terminal (status/input change every frame)
        // C++ Screen.cc:109
        self.refresh_tty(caps);

        true
    }

    /// Render composited canvas to terminal via ANSI (C++ Screen.cc:183-299)
    fn refresh_tty(&mut self, caps: &AcsCaps) {
        let width = self.window.width;
        let height = self.window.height;

        // Generate ANSI escape codes by diffing last_screen vs canvas
        let ansi = diff_to_ansi(
            &self.last_screen,
            &self.window.canvas,
            &DiffOptions {
                width,
                height,
                cursor_x: self.window.cursor_x,
                cursor_y: self.window.cursor_y,
                smacs: caps.smacs.as_deref(),
                rmacs: caps.rmacs.as_deref(),
                set_bg_always: true,
            },
        );

        // Write to stdout (C++ Screen.cc:295)
        let mut out = io::stdout();
        let _ = out.write_all(ansi.as_bytes());
        let _ = out.flush();

        // Update last_screen for next diff (C++ Screen.cc:299)
        self.last_screen.copy_from_slice(&self.window.canvas);
    }

    /// Get mutable window reference
    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    /// Get window reference
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Insert child window (delegate to Window)
    pub fn insert(&mut self, child: *mut Window) {
        self.window.insert(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_creation() {
        let screen = Screen::new(80, 24);
        assert_eq!(screen.window.width, 80);
        assert_eq!(screen.window.height, 24);
        assert_eq!(screen.last_screen.len(), 80 * 24);
    }

    #[test]
    fn screen_refresh() {
        let mut screen = Screen::new(10, 5);
        let caps = AcsCaps::default();

        // Mark window dirty
        screen.window.dirty = true;

        // Refresh should work
        let refreshed = screen.refresh(&caps);
        assert!(refreshed);
        assert!(!screen.window.dirty);
    }
}
