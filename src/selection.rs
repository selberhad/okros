// Selection - Scrollable list widget
//
// Ported from mcl-cpp-reference/Selection.cc (1:1 port)

use crate::input::{KeyCode, KeyEvent};
use crate::window::Window;

/// Base class for scrollable selection lists (C++ Selection.cc:7-37)
/// Subclass and override get_data(), do_select(), do_choose() for custom behavior
pub struct Selection {
    pub win: Box<Window>,
    items: Vec<String>,
    colors: Vec<u8>,
    selection: i32, // Currently selected index (-1 = none)
}

impl Selection {
    /// Create new selection widget (C++ Selection.cc:7-9)
    /// Note: C++ uses Bordered style, we'll draw border manually for now
    pub fn new(parent: *mut Window, width: usize, height: usize, x: isize, y: isize) -> Self {
        let mut win = Window::new(parent, width, height);
        win.parent_x = x;
        win.parent_y = y;
        win.color = 0x17; // bg_blue | fg_white

        Self {
            win,
            items: Vec::new(),
            colors: Vec::new(),
            selection: -1,
        }
    }

    /// Add string to list
    pub fn add_string(&mut self, s: impl Into<String>, color: u8) {
        self.items.push(s.into());
        self.colors.push(color);

        let count = self.items.len() as i32;

        // If first item, auto-select it
        if self.selection == -1 && count > 0 {
            self.selection = 0;
            self.do_select(self.selection);
        }
        // Move selection down if we had the last one selected
        else if count == self.selection + 1 {
            self.selection += 1;
            self.do_select(self.selection);
        }
    }

    /// Prepend string to list (add at beginning)
    pub fn prepend_string(&mut self, s: impl Into<String>, color: u8) {
        self.items.insert(0, s.into());
        self.colors.insert(0, color);

        // Move selection down if we had the last one selected
        let count = self.items.len() as i32;
        if count == self.selection + 1 {
            self.selection += 1;
            self.do_select(self.selection);
        }
    }

    /// Get count of items
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Get current selection index
    pub fn get_selection(&self) -> i32 {
        self.selection
    }

    /// Set selection index
    pub fn set_selection(&mut self, n: i32) {
        self.selection = n.min(self.items.len() as i32);
    }

    /// Set count (adjusts selection if needed)
    pub fn set_count(&mut self, count: usize) {
        let count = count as i32;
        if self.selection >= count {
            self.selection = count - 1;
        } else if self.selection == -1 && count > 0 {
            self.selection = 0;
        }
    }

    /// Get data at index (override in subclass for custom formatting)
    pub fn get_data(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|s| s.as_str())
    }

    /// Handle selection bar moved (override in subclass)
    pub fn do_select(&mut self, _index: i32) {
        // Override in subclass
    }

    /// Handle item chosen (override in subclass)
    pub fn do_choose(&mut self, _index: i32, _key: i32) {
        // Override in subclass - default is to close
    }

    /// Redraw window (C++ Selection.cc:38-66)
    pub fn redraw(&mut self) {
        // Set blue background color (C++ Selection.cc:41-42)
        let bg_blue_fg_white = 0x17u16; // bg_blue (1) | fg_white (7)
        let bg_green_fg_black = 0x20u16; // bg_green (2) | fg_black (0)

        // Clear with blue background (C++ Selection.cc:42)
        let blank = (bg_blue_fg_white << 8) | (b' ' as u16);
        for a in &mut self.win.canvas {
            *a = blank;
        }

        // Draw border (C++ Selection uses Bordered style which creates Border window)
        // Top border
        let width = self.win.width;
        let height = self.win.height;
        self.win.canvas[0] = (bg_blue_fg_white << 8) | (b'+' as u16);
        for x in 1..width - 1 {
            self.win.canvas[x] = (bg_blue_fg_white << 8) | (b'-' as u16);
        }
        self.win.canvas[width - 1] = (bg_blue_fg_white << 8) | (b'+' as u16);

        // Left and right borders
        for y in 1..height - 1 {
            self.win.canvas[y * width] = (bg_blue_fg_white << 8) | (b'|' as u16);
            self.win.canvas[y * width + width - 1] = (bg_blue_fg_white << 8) | (b'|' as u16);
        }

        // Bottom border
        self.win.canvas[(height - 1) * width] = (bg_blue_fg_white << 8) | (b'+' as u16);
        for x in 1..width - 1 {
            self.win.canvas[(height - 1) * width + x] = (bg_blue_fg_white << 8) | (b'-' as u16);
        }
        self.win.canvas[(height - 1) * width + width - 1] = (bg_blue_fg_white << 8) | (b'+' as u16);

        // Calculate top line for scrolling (C++ Selection.cc:47-48)
        // Content area is inside border, so height-2 rows available
        let count = self.items.len() as i32;
        let content_height = (height - 2) as i32;
        let mut top = 0.max(self.selection - content_height / 2);
        top = 0.max(count - content_height).min(top);

        // Draw items inside border (C++ Selection.cc:50-63)
        for y in 0..content_height {
            let idx = (y + top) as usize;
            if idx >= self.items.len() {
                break;
            }

            // Determine color for this line (C++ Selection.cc:52-60)
            let color = if y + top == self.selection {
                // Selected line - green background (C++ Selection.cc:53)
                bg_green_fg_black
            } else {
                // Check if item has custom color (C++ Selection.cc:55-60)
                let item_color = self.colors.get(idx).copied().unwrap_or(0);
                if item_color != 0 {
                    item_color as u16
                } else {
                    bg_blue_fg_white
                }
            };

            // Write line to canvas inside border (offset by 1 for border)
            // Copy data to avoid borrow conflict
            let data_bytes: Vec<u8> = self.get_data(idx).unwrap_or("").as_bytes().to_vec();
            let content_y = (y + 1) as usize; // +1 for top border
            let content_width = width - 2; // -2 for left/right borders
            for x in 0..content_width {
                let ch = if x < data_bytes.len() {
                    data_bytes[x]
                } else {
                    b' '
                };
                self.win.canvas[content_y * width + x + 1] = (color << 8) | (ch as u16);
                // +1 for left border
            }
        }

        self.win.dirty = false; // C++ Selection.cc:65
    }

    /// Handle keypress - returns true if handled
    pub fn keypress(&mut self, event: KeyEvent) -> bool {
        self.win.dirty = true; // C++ Selection.cc:69

        if self.selection >= 0 {
            let count = self.items.len() as i32;
            let height = self.win.height as i32;

            match event {
                KeyEvent::Key(KeyCode::ArrowUp) => {
                    self.selection = 0.max(self.selection - 1);
                    self.do_select(self.selection);
                }
                KeyEvent::Key(KeyCode::ArrowDown) => {
                    self.selection = (self.selection + 1).min(count - 1);
                    self.do_select(self.selection);
                }
                KeyEvent::Key(KeyCode::PageUp) => {
                    self.selection = 0.max(self.selection - height / 2);
                    self.do_select(self.selection);
                }
                KeyEvent::Key(KeyCode::PageDown) => {
                    self.selection = (self.selection + height / 2).min(count - 1);
                    self.do_select(self.selection);
                }
                KeyEvent::Key(KeyCode::Home) => {
                    self.selection = 0;
                    self.do_select(self.selection);
                }
                KeyEvent::Key(KeyCode::End) => {
                    self.selection = count - 1;
                    self.do_select(self.selection);
                }
                KeyEvent::Byte(b'\n')
                | KeyEvent::Byte(b'\r')
                | KeyEvent::Key(KeyCode::ArrowRight) => {
                    self.do_choose(self.selection, 0);
                }
                KeyEvent::Key(KeyCode::Escape) => {
                    return false; // Close widget
                }
                // Letter jump: find first item starting with this letter
                KeyEvent::Byte(ch @ b' '..=127) => {
                    if count == 0 {
                        return true;
                    }

                    // Start search from next item, wrap around
                    let start = if let Some(data) = self.get_data(self.selection as usize) {
                        if data.as_bytes().first() == Some(&ch) {
                            (self.selection + 1) as usize
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                    for i in 0..count as usize {
                        let idx = (start + i) % (count as usize);
                        if let Some(data) = self.get_data(idx) {
                            if data.as_bytes().first() == Some(&ch) {
                                self.selection = idx as i32;
                                break;
                            }
                        }
                    }
                }
                _ => return false,
            }
            true
        } else {
            // No items
            matches!(
                event,
                KeyEvent::Byte(b'\n') | KeyEvent::Byte(b'\r') | KeyEvent::Key(KeyCode::Escape)
            )
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
    fn selection_basic() {
        let mut sel = Selection::new(ptr::null_mut(), 80, 24, 0, 0);
        sel.add_string("Item 1", 0);
        sel.add_string("Item 2", 0);
        sel.add_string("Item 3", 0);

        assert_eq!(sel.count(), 3);
        assert_eq!(sel.get_selection(), 0);
    }

    #[test]
    fn selection_navigation() {
        let mut sel = Selection::new(ptr::null_mut(), 80, 24, 0, 0);
        for i in 1..=10 {
            sel.add_string(format!("Item {}", i), 0);
        }
        sel.set_selection(0);

        // Arrow down
        sel.keypress(KeyEvent::Key(KeyCode::ArrowDown));
        assert_eq!(sel.get_selection(), 1);

        // Arrow up
        sel.keypress(KeyEvent::Key(KeyCode::ArrowUp));
        assert_eq!(sel.get_selection(), 0);

        // End
        sel.keypress(KeyEvent::Key(KeyCode::End));
        assert_eq!(sel.get_selection(), 9);

        // Home
        sel.keypress(KeyEvent::Key(KeyCode::Home));
        assert_eq!(sel.get_selection(), 0);
    }

    #[test]
    fn selection_letter_jump() {
        let mut sel = Selection::new(ptr::null_mut(), 80, 24, 0, 0);
        sel.add_string("Apple", 0);
        sel.add_string("Banana", 0);
        sel.add_string("Cherry", 0);
        sel.set_selection(0);

        // Jump to 'B'
        sel.keypress(KeyEvent::Byte(b'B'));
        assert_eq!(sel.get_selection(), 1);

        // Jump to 'C'
        sel.keypress(KeyEvent::Byte(b'C'));
        assert_eq!(sel.get_selection(), 2);

        // Jump to 'A'
        sel.keypress(KeyEvent::Byte(b'A'));
        assert_eq!(sel.get_selection(), 0);
    }

    #[test]
    fn selection_redraw_blue_background() {
        let mut sel = Selection::new(ptr::null_mut(), 20, 5, 0, 0);
        sel.add_string("Test Item", 0);
        sel.redraw();

        // Check that canvas has blue background (0x17)
        let bg_blue_fg_white = 0x17u16;
        for &attr in &sel.win.canvas {
            let color = (attr >> 8) as u8;
            // Should be either blue background or green selection
            assert!(color == 0x17 || color == 0x20);
        }
    }
}
