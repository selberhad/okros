use crate::scrollback::{Attrib, Scrollback};
use crate::window::Window;

/// Search highlight information (C++ OutputWindow.cc:37-42)
#[derive(Default)]
struct Highlight {
    line: i32,  // Line number to highlight (-1 = none)
    x: usize,   // X offset to start highlight
    len: usize, // Length of highlight
}

/// OutputWindow - Window that displays scrollback buffer
/// Ported from: mcl-cpp-reference/OutputWindow.cc
///
/// C++ pattern: OutputWindow : public Window, overrides scroll()
/// Rust pattern: OutputWindow owns Window, implements redraw()
pub struct OutputWindow {
    pub win: Box<Window>,
    pub sb: Scrollback,
    color: u8,
    highlight: Highlight,
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
            highlight: Highlight {
                line: -1, // -1 = no highlight
                x: 0,
                len: 0,
            },
        }
    }

    /// Print line to scrollback and mark dirty (C++ OutputWindow prints to canvas)
    pub fn print_line(&mut self, bytes: &[u8], color: u8) {
        self.sb.print_line(bytes, color);
        self.redraw();
    }

    /// Redraw window: blit scrollback viewport to canvas (C++ Window::redraw pattern)
    /// Updated to handle search highlighting (C++ OutputWindow::draw_on_parent lines 239-274)
    pub fn redraw(&mut self) {
        let view = self.sb.viewport_slice();

        // Check if we need to highlight search result (C++ lines 246-248)
        if self.highlight.line >= 0 {
            let viewpoint_line = (self.sb.viewpoint / self.sb.width) + self.sb.top_line;
            let highlight_line = self.highlight.line as usize;

            // Is highlighted line visible in viewport? (C++ lines 246-248)
            if highlight_line >= viewpoint_line && highlight_line < viewpoint_line + self.sb.height
            {
                let line_in_view = highlight_line - viewpoint_line;
                let start_offset = line_in_view * self.sb.width + self.highlight.x;
                let end_offset = start_offset + self.highlight.len;

                // Create modified view with inverted colors for highlight (C++ lines 251-264)
                let mut modified_view = view.to_vec();

                if end_offset <= modified_view.len() {
                    for attrib in &mut modified_view[start_offset..end_offset] {
                        // Invert colors: swap foreground and background (C++ lines 259-263)
                        let color = ((*attrib & 0xFF00) >> 8) as u8;
                        let bg = (color & 0x0F) << 4;
                        let fg = (color & 0xF0) >> 4;
                        *attrib = (*attrib & 0x00FF) | (((bg | fg) as u16) << 8);
                    }

                    self.win.blit(&modified_view);
                    return;
                }
            }
        }

        // Normal blit without highlighting
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

    /// Freeze scrollback (stop auto-scrolling)
    pub fn freeze(&mut self) {
        self.sb.set_frozen(true);
    }

    /// Unfreeze scrollback (resume auto-scrolling to bottom)
    pub fn unfreeze(&mut self) {
        self.sb.set_frozen(false);
        // Snap viewpoint to canvas position
        self.sb.viewpoint = self.sb.canvas_ptr();
    }

    /// Page up in scrollback (C++ ScrollbackController::keypress line 133-135)
    pub fn page_up(&mut self) -> bool {
        let quit = self.sb.page_up();
        self.redraw();
        quit
    }

    /// Page down in scrollback (C++ ScrollbackController::keypress line 137-139)
    pub fn page_down(&mut self) -> bool {
        let quit = self.sb.page_down();
        self.redraw();
        quit
    }

    /// Line up in scrollback (C++ ScrollbackController::keypress line 141-143)
    pub fn line_up(&mut self) -> bool {
        let quit = self.sb.line_up();
        self.redraw();
        quit
    }

    /// Line down in scrollback (C++ ScrollbackController::keypress line 145-147)
    pub fn line_down(&mut self) -> bool {
        let quit = self.sb.line_down();
        self.redraw();
        quit
    }

    /// Home in scrollback (C++ ScrollbackController::keypress line 149-151)
    pub fn home(&mut self) -> bool {
        let quit = self.sb.home();
        self.redraw();
        quit
    }

    /// Search for text in scrollback (C++ OutputWindow::search, lines 174-236)
    /// Returns Some(message) to display in status bar
    pub fn search(&mut self, text: &str, forward: bool) -> Option<String> {
        if text.is_empty() {
            return Some("Search string is empty".to_string());
        }

        let search_bytes = text.to_lowercase().into_bytes();
        let len = search_bytes.len();

        // Start search from current viewpoint (C++ line 176)
        // C++ uses cursor_y-1, but we'll search from the middle of the viewport
        let start_line = self.sb.viewpoint / self.sb.width + (self.sb.height / 2);

        // Search through all lines in scrollback
        let total_lines = if self.sb.canvas_off > 0 {
            self.sb.canvas_off / self.sb.width
        } else {
            0
        };

        let mut current_line = start_line;
        let mut found = false;
        let mut found_x = 0;
        let mut found_line = 0;

        // C++ does unbounded loop with manual break (lines 181-221)
        for _ in 0..total_lines {
            if current_line >= total_lines {
                break;
            }

            let line_offset = current_line * self.sb.width;
            if line_offset >= self.sb.buf.len() {
                break;
            }

            // Search current line (C++ lines 184-200)
            // Search from beginning to width-len
            if self.sb.width >= len {
                for x in 0..=(self.sb.width - len) {
                    let mut matches = true;

                    // Compare characters case-insensitively (C++ lines 189-195)
                    for (i, search_ch) in search_bytes.iter().enumerate() {
                        let buf_offset = line_offset + x + i;
                        if buf_offset >= self.sb.buf.len() {
                            matches = false;
                            break;
                        }
                        let buf_ch = (self.sb.buf[buf_offset] & 0xFF) as u8;
                        if buf_ch.to_ascii_lowercase() != *search_ch {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        found = true;
                        found_x = x;
                        found_line = current_line;
                        break;
                    }
                }
            }

            if found {
                break;
            }

            // Move to next line (C++ lines 206-220)
            if forward {
                current_line += 1;
            } else {
                if current_line == 0 {
                    break;
                }
                current_line -= 1;
            }
        }

        if !found {
            // Clear highlight
            self.highlight.line = -1;
            Some(format!("Search string '{}' not found", text))
        } else {
            // Set highlight (C++ lines 227-229)
            self.highlight.line = (found_line + self.sb.top_line) as i32;
            self.highlight.x = found_x;
            self.highlight.len = len;

            // Adjust viewpoint to show the found line (C++ lines 231-233)
            // Show on the second line rather than under status bar
            let target_viewpoint = if found_line > 0 {
                (found_line - 1) * self.sb.width
            } else {
                0
            };

            // Clamp to valid range
            self.sb.viewpoint = target_viewpoint.min(self.sb.canvas_off);

            self.redraw();
            Some(format!("Found string '{}'", text))
        }
    }

    /// Clear search highlight
    pub fn clear_highlight(&mut self) {
        self.highlight.line = -1;
        self.redraw();
    }

    /// Save scrollback to file (C++ OutputWindow::saveToFile, lines 301-322)
    /// Returns Some(message) for status bar
    pub fn save_to_file(&self, filename: &str, use_color: bool) -> Option<String> {
        use std::fs::File;
        use std::io::Write;

        // Open file for writing (C++ line 302)
        let mut file = match File::create(filename) {
            Ok(f) => f,
            Err(e) => {
                return Some(format!("Cannot open {} for writing: {}", filename, e));
            }
        };

        // Write header (C++ line 306)
        let timestamp = chrono::Local::now().format("%a %b %e %H:%M:%S %Y");
        if let Err(e) = writeln!(file, "Scrollback saved from okros at {}", timestamp) {
            return Some(format!("Write error: {}", e));
        }

        // Write scrollback content (C++ lines 308-318)
        // Iterate through all lines from scrollback start to canvas end
        let total_lines = if self.sb.canvas_off > 0 {
            self.sb.canvas_off / self.sb.width + self.sb.height
        } else {
            self.sb.height
        };

        let mut last_color = 255u8; // Invalid color to force first color code

        for line_num in 0..total_lines {
            let line_offset = line_num * self.sb.width;

            if line_offset >= self.sb.buf.len() {
                break;
            }

            let line_end = (line_offset + self.sb.width).min(self.sb.buf.len());

            for &attrib in &self.sb.buf[line_offset..line_end] {
                let ch = (attrib & 0xFF) as u8;
                let color = ((attrib >> 8) & 0xFF) as u8;

                // Output color code if changed and use_color is true (C++ lines 311-313)
                if use_color && color != last_color {
                    // Generate ANSI color code
                    let ansi_code = attrib_to_ansi_color(color);
                    if let Err(e) = write!(file, "{}", ansi_code) {
                        return Some(format!("Write error: {}", e));
                    }
                    last_color = color;
                }

                // Write character (C++ line 315)
                if let Err(e) = write!(file, "{}", ch as char) {
                    return Some(format!("Write error: {}", e));
                }
            }

            // Write newline (C++ line 317)
            if let Err(e) = writeln!(file) {
                return Some(format!("Write error: {}", e));
            }
        }

        Some(format!("Scrollback saved to {} successfully", filename))
    }
}

/// Convert attribute color byte to ANSI escape sequence
/// Simplified version of C++ Screen::getColorCode()
fn attrib_to_ansi_color(color: u8) -> String {
    let fg = color & 0x0F;
    let bg = (color >> 4) & 0x0F;

    // ANSI color codes: 30-37 for foreground, 40-47 for background
    // Mapping: 0=black, 1=red, 2=green, 3=yellow, 4=blue, 5=magenta, 6=cyan, 7=white
    // Bold (bright) colors use the bold attribute (1) + base color

    let mut codes = Vec::new();

    // Foreground color
    if fg & 0x08 != 0 {
        // Bright/bold
        codes.push("1".to_string());
        codes.push(format!("{}", 30 + (fg & 0x07)));
    } else {
        codes.push(format!("{}", 30 + fg));
    }

    // Background color
    codes.push(format!("{}", 40 + (bg & 0x07)));

    format!("\x1b[{}m", codes.join(";"))
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

    #[test]
    fn save_to_file_plain_text() {
        use std::fs;
        use std::ptr;

        let mut ow = OutputWindow::new(ptr::null_mut(), 10, 3, 20, 0x07);
        ow.print_line(b"line one", 0x07);
        ow.print_line(b"line two", 0x07);
        ow.print_line(b"line three", 0x07);

        // Save without color
        let filename = "/tmp/test_scrollback.txt";
        let result = ow.save_to_file(filename, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("successfully"));

        // Read file and verify contents
        let content = fs::read_to_string(filename).unwrap();
        assert!(content.contains("line one"));
        assert!(content.contains("line two"));
        assert!(content.contains("line three"));
        assert!(content.contains("Scrollback saved from okros"));

        // Should not contain ANSI codes
        assert!(!content.contains("\x1b["));

        fs::remove_file(filename).ok();
    }

    #[test]
    fn save_to_file_with_color() {
        use std::fs;
        use std::ptr;

        let mut ow = OutputWindow::new(ptr::null_mut(), 10, 2, 20, 0x07);
        ow.print_line(b"red text", 0x0C); // Red foreground
        ow.print_line(b"blue text", 0x09); // Blue foreground

        // Save with color
        let filename = "/tmp/test_scrollback_color.txt";
        let result = ow.save_to_file(filename, true);
        assert!(result.is_some());
        assert!(result.unwrap().contains("successfully"));

        // Read file and verify ANSI codes present
        let content = fs::read_to_string(filename).unwrap();
        assert!(content.contains("red text"));
        assert!(content.contains("blue text"));
        assert!(content.contains("\x1b[")); // Should have ANSI codes

        fs::remove_file(filename).ok();
    }
}
