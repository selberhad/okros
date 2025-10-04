// InputBox - Modal dialog with text input
//
// Ported from: mcl-cpp-reference/InputBox.cc
//
// C++ pattern: InputBox : public Window with virtual execute()
// Rust pattern: InputBox owns Window and InputLine, uses callback for execute

use crate::history::HistoryId;
use crate::input::{KeyCode, KeyEvent};
use crate::input_line::InputLine;
use crate::window::Window;

/// Callback type for InputBox execute
/// NOTE: Send bound removed to allow capturing raw pointers (e.g., *mut OutputWindow)
/// This is safe because the callback only runs on the main UI thread
pub type ExecuteCallback = Box<dyn FnMut(&str)>;

/// InputBox - Simple dialog box which prompts for input (C++ InputBox.h:3-24)
pub struct InputBox {
    win: Box<Window>,
    input: InputLine,
    prompt: String,
    execute_cb: Option<ExecuteCallback>,
    can_cancel: bool,
}

impl InputBox {
    /// Create new InputBox centered on parent (C++ InputBox.cc:23-32)
    pub fn new(
        parent: *mut Window,
        prompt: &str,
        history_id: HistoryId,
        execute_cb: ExecuteCallback,
    ) -> Self {
        // Calculate size from prompt (C++ line 24)
        // Width = prompt length + 4, Height = 7
        let width = prompt.len() + 4;
        let height = 7;

        // Calculate centering position (C++ Window.cc:25-33, xy_center = -999)
        let (parent_x, parent_y) = if !parent.is_null() {
            unsafe {
                let px = (*parent).width / 2 - width / 2;
                let py = (*parent).height / 2 - height / 2;
                (px as isize, py as isize)
            }
        } else {
            (0, 0)
        };

        // Create bordered window
        // C++ creates Border window and adjusts dimensions (Window.cc:36-42)
        // We'll draw border manually like Selection does
        let mut win = Window::new(parent, width, height);
        win.parent_x = parent_x;
        win.parent_y = parent_y;

        // Create InputLine inside border (C++ line 26-29)
        // Position: 1 char from left, row 3 (middle)
        // Width: 2 chars smaller than window width (for borders)
        let input = InputLine::new(win.as_mut(), width - 2, 0x07, history_id);

        // NOTE: InputLine positioning handled in redraw

        Self {
            win,
            input,
            prompt: prompt.to_string(),
            execute_cb: Some(execute_cb),
            can_cancel: true,
        }
    }

    /// Redraw window (C++ InputBox.cc:34-40)
    pub fn redraw(&mut self) {
        // Set color: blue background, white foreground (C++ line 35)
        // bg_blue|fg_white = 0x1F (blue=0x10, white=0x0F)
        self.win.set_color(0x1F);
        self.win.clear();

        // Draw border (similar to Selection - C++ uses Bordered style)
        self.draw_border();

        // Print prompt at position (1,1) inside border (C++ lines 37-38)
        self.win.gotoxy(2, 2);
        self.win.print(&self.prompt);

        // Position InputLine at (1, 3) inside border (C++ line 27-28)
        // InputLine redraw will be called separately

        self.win.dirty = false;
    }

    /// Draw border (adapted from Selection::redraw)
    fn draw_border(&mut self) {
        let width = self.win.width;
        let height = self.win.height;

        // Top border
        self.win.gotoxy(0, 0);
        self.win.print("+");
        for _ in 1..width - 1 {
            self.win.print("-");
        }
        self.win.print("+");

        // Left and right borders
        for y in 1..height - 1 {
            self.win.put_char(0, y, b'|', 0x1F);
            self.win.put_char(width - 1, y, b'|', 0x1F);
        }

        // Bottom border
        self.win.gotoxy(0, height - 1);
        self.win.print("+");
        for _ in 1..width - 1 {
            self.win.print("-");
        }
        self.win.print("+");
    }

    /// Handle keypress (C++ InputBox.cc:42-50)
    pub fn keypress(&mut self, key: KeyEvent) -> bool {
        // Check for Escape key (C++ lines 43-46)
        if matches!(key, KeyEvent::Key(KeyCode::Escape)) {
            if self.can_cancel {
                // Close the dialog by calling die()
                // In C++: die() deletes the window
                // In Rust: caller must handle dropping the Box
                self.win.die();
                return true;
            }
            return true;
        }

        // Check for Enter key - execute the input
        if matches!(key, KeyEvent::Byte(b'\n') | KeyEvent::Byte(b'\r')) {
            if let Some(mut cb) = self.execute_cb.take() {
                let text = self.input.get_input();
                cb(&text);
                // Don't restore callback - dialog should close after execute
            }
            self.win.die();
            return true;
        }

        // Pass to InputLine for editing
        // NOTE: InputLine::keypress in Rust needs CommandQueue, but we don't use it here
        // For now, we'll handle basic editing ourselves
        // TODO: Properly integrate InputLine keypress handling

        // Delegate to Window::keypress (C++ line 49)
        // This will dispatch to children
        false
    }

    /// Get window pointer (for event loop integration)
    pub fn window(&mut self) -> &mut Window {
        self.win.as_mut()
    }

    /// Get mutable InputLine reference
    pub fn input_line(&mut self) -> &mut InputLine {
        &mut self.input
    }

    /// Set whether dialog can be cancelled with Escape (C++ canCancel())
    pub fn set_can_cancel(&mut self, can_cancel: bool) {
        self.can_cancel = can_cancel;
    }

    /// Get the window Box for ownership transfer
    pub fn into_window(self) -> Box<Window> {
        self.win
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::sync::{Arc, Mutex};

    #[test]
    fn input_box_creation() {
        let root = Window::new(ptr::null_mut(), 80, 24);
        let executed = Arc::new(Mutex::new(String::new()));
        let executed_clone = executed.clone();

        let input_box = InputBox::new(
            root.as_ref() as *const _ as *mut _,
            "Enter name:",
            HistoryId::None,
            Box::new(move |text| {
                *executed_clone.lock().unwrap() = text.to_string();
            }),
        );

        assert_eq!(input_box.prompt, "Enter name:");
        assert!(input_box.can_cancel);
    }

    #[test]
    fn input_box_escape() {
        let root = Window::new(ptr::null_mut(), 80, 24);
        let mut input_box = InputBox::new(
            root.as_ref() as *const _ as *mut _,
            "Test:",
            HistoryId::None,
            Box::new(|_| {}),
        );

        // Escape should close if can_cancel is true
        assert!(input_box.keypress(KeyEvent::Key(KeyCode::Escape)));

        // Reset and test with can_cancel = false
        let root = Window::new(ptr::null_mut(), 80, 24);
        let mut input_box = InputBox::new(
            root.as_ref() as *const _ as *mut _,
            "Test:",
            HistoryId::None,
            Box::new(|_| {}),
        );
        input_box.set_can_cancel(false);

        // Escape should be handled but not close
        assert!(input_box.keypress(KeyEvent::Key(KeyCode::Escape)));
    }
}
