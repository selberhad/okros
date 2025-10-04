// Selection - Scrollable list widget
//
// Ported from mcl-cpp-reference/Selection.cc

use crate::input::{KeyCode, KeyEvent};
use crate::window::Window;
use std::ptr;

/// Base class for scrollable selection lists
/// Subclass and override get_data(), do_select(), do_choose() for custom behavior
pub struct Selection {
    window: Box<Window>,
    items: Vec<String>,
    colors: Vec<u8>,
    selection: i32, // Currently selected index (-1 = none)
}

impl Selection {
    /// Create new selection widget
    pub fn new(width: usize, height: usize, x: usize, y: usize) -> Self {
        Self {
            window: Window::new(ptr::null_mut(), width, height),
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

    /// Handle keypress - returns true if handled
    pub fn keypress(&mut self, event: KeyEvent) -> bool {
        if self.selection >= 0 {
            let count = self.items.len() as i32;
            let height = self.window.height as i32;

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

    /// Render the selection list
    pub fn render(&mut self) -> Vec<u8> {
        // TODO: Full rendering implementation
        // For now, return placeholder
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_basic() {
        let mut sel = Selection::new(80, 24, 0, 0);
        sel.add_string("Item 1", 0);
        sel.add_string("Item 2", 0);
        sel.add_string("Item 3", 0);

        assert_eq!(sel.count(), 3);
        assert_eq!(sel.get_selection(), 0);
    }

    #[test]
    fn selection_navigation() {
        let mut sel = Selection::new(80, 24, 0, 0);
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
        let mut sel = Selection::new(80, 24, 0, 0);
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
}
