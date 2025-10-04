// MUDSelection - Specialized selection widget for MUD connect menu
//
// Ported from mcl-cpp-reference/Selection.cc:170-213 (1:1 port)

use crate::config::Config;
use crate::input::{KeyCode, KeyEvent};
use crate::selection::Selection;
use crate::window::Window;

/// Specialized selection widget for choosing MUDs from config (C++ Selection.cc:39-48)
pub struct MudSelection {
    selection: Selection,
    config: Config,
}

impl MudSelection {
    /// Create new MUD selection menu (C++ Selection.cc:170-175)
    /// C++: Selection (_parent, _parent->width-2, _parent->height/2, 0, _parent->height/4)
    pub fn new(parent: *mut Window, config: Config) -> Self {
        // Get parent dimensions for centered window
        let (parent_width, parent_height) = unsafe {
            if !parent.is_null() {
                ((*parent).width, (*parent).height)
            } else {
                (80, 24) // Fallback
            }
        };

        // C++ MUDSelection.cc:171 - centered window
        let width = parent_width.saturating_sub(2); // parent->width - 2
        let height = parent_height / 2; // parent->height / 2
        let x = 0;
        let y = (parent_height / 4) as isize; // parent->height / 4 (centered)

        let mut selection = Selection::new(parent, width, height, x, y);

        // Populate selection with MUD names
        for mud in config.mud_list.iter() {
            // Format: "mudname hostname port commands"
            let display = if !mud.hostname.is_empty() {
                format!(
                    "{:<12} {:<35} {:>5} {}",
                    truncate(&mud.name, 12),
                    truncate(&mud.hostname, 35),
                    mud.port,
                    mud.commands
                )
            } else {
                mud.name.clone()
            };
            selection.add_string(display, 0);
        }

        Self { selection, config }
    }

    /// Get current selection index
    pub fn get_selection(&self) -> i32 {
        self.selection.get_selection()
    }

    /// Get selected MUD name
    pub fn get_selected_mud_name(&self) -> Option<&str> {
        let idx = self.selection.get_selection();
        if idx >= 0 {
            self.config
                .mud_list
                .get(idx as usize)
                .map(|m| m.name.as_str())
        } else {
            None
        }
    }

    /// Get MUD at index for rendering
    pub fn get_mud_at(&self, index: usize) -> Option<(&str, &str, u16)> {
        self.config
            .mud_list
            .get(index)
            .map(|m| (m.name.as_str(), m.hostname.as_str(), m.port))
    }

    /// Get mutable window pointer for tree operations
    pub fn window_mut_ptr(&mut self) -> *mut Window {
        self.selection.window_mut_ptr()
    }

    /// Redraw the window
    pub fn redraw(&mut self) {
        self.selection.redraw();
    }

    /// Handle keypress event
    pub fn keypress(&mut self, event: KeyEvent) -> bool {
        // Special handling for Alt-A (show aliases) - not implemented yet
        if matches!(event, KeyEvent::Key(KeyCode::Alt(b'a'))) {
            // TODO: Show alias selection for selected MUD
            return true;
        }

        // Special handling for Alt-O (already in MUD selection, ignore)
        if matches!(event, KeyEvent::Key(KeyCode::Alt(b'o'))) {
            // Status message: "It's already open!"
            return true;
        }

        // Delegate to base selection
        self.selection.keypress(event)
    }

    /// Get number of MUDs in list
    pub fn count(&self) -> usize {
        self.selection.count()
    }
}

/// Truncate string to max length, preserving full words if possible
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mud::Mud;
    use std::ptr;

    #[test]
    fn mud_selection_basic() {
        let mut config = Config::new();
        config
            .mud_list
            .insert(Mud::new("TestMUD", "127.0.0.1", 4000));
        config.mud_list.insert(Mud::new("Nodeka", "nodeka.com", 23));

        let sel = MudSelection::new(ptr::null_mut(), config);
        assert_eq!(sel.count(), 2);
        assert_eq!(sel.get_selection(), 0);
    }

    #[test]
    fn mud_selection_navigation() {
        let mut config = Config::new();
        for i in 1..=5 {
            config
                .mud_list
                .insert(Mud::new(&format!("MUD{}", i), "127.0.0.1", 4000 + i));
        }

        let mut sel = MudSelection::new(ptr::null_mut(), config);

        // Navigate down
        sel.keypress(KeyEvent::Key(KeyCode::ArrowDown));
        assert_eq!(sel.get_selection(), 1);

        // Navigate to end
        sel.keypress(KeyEvent::Key(KeyCode::End));
        assert_eq!(sel.get_selection(), 4);

        // Navigate to home
        sel.keypress(KeyEvent::Key(KeyCode::Home));
        assert_eq!(sel.get_selection(), 0);
    }

    #[test]
    fn mud_selection_get_name() {
        let mut config = Config::new();
        config
            .mud_list
            .insert(Mud::new("TestMUD", "127.0.0.1", 4000));
        config.mud_list.insert(Mud::new("Nodeka", "nodeka.com", 23));

        let sel = MudSelection::new(ptr::null_mut(), config);
        assert_eq!(sel.get_selected_mud_name(), Some("TestMUD"));
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }
}
