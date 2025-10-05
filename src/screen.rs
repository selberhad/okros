// Screen - Top-level window that renders to terminal
//
// Ported from: mcl-cpp-reference/Screen.cc
//
// C++ inheritance: Screen : public Window
// Rust pattern: Screen owns a Window, delegates to it

use crate::curses::AcsCaps;
use crate::scrollback::Attrib;
use crate::window::Window;
use std::io::{self, Write};
use std::ptr;

const FG_BOLD: u8 = 1 << 7;

fn reverse_color_conv_table(idx: u8) -> u8 {
    match idx & 0x07 {
        0 => 0,
        1 => 4,
        2 => 2,
        3 => 6,
        4 => 1,
        5 => 5,
        6 => 3,
        _ => 7,
    }
}

pub fn get_color_code(color: u8, set_bg: bool) -> String {
    let fg = 30 + reverse_color_conv_table(color & 0x07) as i32;
    let bold = (color & FG_BOLD) != 0;
    let bg = 40 + reverse_color_conv_table((color >> 4) & 0x07) as i32;
    if fg == 37 && bg == 40 && !bold {
        return "\u{1b}[0m".to_string();
    }
    let bg_part = if set_bg {
        format!("{};", bg)
    } else {
        String::new()
    };
    if bold {
        format!("\u{1b}[1;{}{}m", bg_part, fg)
    } else {
        format!("\u{1b}[0;{}{}m", bg_part, fg)
    }
}

fn vt_home() -> &'static str {
    "\u{1b}[H"
}
fn vt_goto(y1: usize, x1: usize) -> String {
    format!("\u{1b}[{};{}H", y1, x1)
}

/// Convert a row of Attrib cells to an ANSI-formatted string (for headless mode)
/// Preserves all color information as escape sequences
pub fn attrib_row_to_ansi(row: &[Attrib]) -> String {
    let mut out = String::new();
    let mut current_color: Option<u8> = None;

    for &attr in row {
        let color = (attr >> 8) as u8;
        let ch = (attr & 0xFF) as u8;

        // Emit color change if needed
        if current_color != Some(color) {
            out.push_str(&get_color_code(color, true));
            current_color = Some(color);
        }

        // Emit character (replace control chars with space)
        out.push(if ch >= 32 { ch as char } else { ' ' });
    }

    // Reset at end of line if we changed colors
    if current_color.is_some() && current_color != Some(0x07) {
        out.push_str("\x1b[0m");
    }

    out.trim_end().to_string()
}

pub struct DiffOptions<'a> {
    pub width: usize,
    pub height: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub smacs: Option<&'a str>,
    pub rmacs: Option<&'a str>,
    pub set_bg_always: bool,
}
impl<'a> Default for DiffOptions<'a> {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cursor_x: 0,
            cursor_y: 0,
            smacs: None,
            rmacs: None,
            set_bg_always: true,
        }
    }
}

pub fn diff_to_ansi(prev: &[Attrib], next: &[Attrib], opt: &DiffOptions) -> String {
    assert_eq!(prev.len(), next.len());
    assert_eq!(prev.len(), opt.width * opt.height);
    let mut out = String::new();
    out.push_str(vt_home());
    let mut saved_color: i32 = -1;
    let mut last_x = 0usize; // 0-based like C++
    let mut last_y = 0usize; // 0-based like C++
    let mut acs = false;
    for y in 0..opt.height {
        for x in 0..opt.width {
            if y == opt.height - 1 && x == opt.width - 1 {
                continue;
            }
            let idx = y * opt.width + x;
            if prev[idx] == next[idx] {
                continue;
            }
            let color = (next[idx] >> 8) as u8;
            let ch = (next[idx] & 0xFF) as u8;
            if (color as i32) != saved_color {
                out.push_str(&get_color_code(color, opt.set_bg_always));
                saved_color = color as i32;
            }
            // Are we there yet? (C++ Screen.cc:256-271)
            if x != last_x || y != last_y {
                // Optimization: print skipped char if adjacent & same color
                if last_y == y
                    && last_x == x - 1
                    && idx > 0
                    && (next[idx - 1] >> 8) as i32 == saved_color
                {
                    print_character(&mut out, (next[idx - 1] & 0xFF) as u8, &mut acs, opt);
                } else {
                    out.push_str(&vt_goto(y + 1, x + 1));
                }
            }
            last_y = y;
            last_x = x + 1;
            if last_x >= opt.width {
                last_x = 0;
                last_y += 1;
            }
            print_character(&mut out, ch, &mut acs, opt);
        }
    }
    out.push_str(&vt_goto(opt.cursor_y + 1, opt.cursor_x + 1));
    if acs {
        if let Some(r) = opt.rmacs {
            out.push_str(r);
        }
    }
    out
}

/// Print character with ACS handling (C++ Screen::printCharacter)
#[inline]
fn print_character(out: &mut String, ch: u8, acs: &mut bool, opt: &DiffOptions) {
    if ch >= 0xEC && ch < 0xEC + 8 {
        if !*acs {
            if let Some(s) = opt.smacs {
                out.push_str(s);
            }
            *acs = true;
        }
        out.push('#');
    } else {
        if *acs {
            if let Some(r) = opt.rmacs {
                out.push_str(r);
            }
            *acs = false;
        }
        out.push(if ch >= 32 { ch as char } else { ' ' });
    }
}

pub fn plan_scroll_up(
    last: &[Attrib],
    next: &[Attrib],
    width: usize,
    height: usize,
    reg_y: usize,
    reg_h: usize,
) -> Option<usize> {
    if reg_y + reg_h > height {
        return None;
    }
    for n in 1..reg_h {
        let mut ok = true;
        for row in 0..(reg_h - n) {
            let ly = reg_y + row + n;
            let ny = reg_y + row;
            let lo = ly * width;
            let no = ny * width;
            if &last[lo..lo + width] != &next[no..no + width] {
                ok = false;
                break;
            }
        }
        if ok {
            return Some(n);
        }
    }
    None
}

pub fn emit_scroll_ansi(
    width: usize,
    height: usize,
    reg_y: usize,
    reg_h: usize,
    lines: usize,
) -> String {
    let y1 = reg_y + 1;
    let y2 = reg_y + reg_h;
    let mut s = String::new();
    s.push_str(&format!("\u{1b}[{};{}r", y1, y2));
    s.push_str(&format!("\u{1b}[{};{}H", y2, 1));
    for _ in 0..lines {
        s.push('\n');
    }
    s.push_str(&format!("\u{1b}[{};{}r", 1, height));
    s
}

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
        // refreshTTY only if something changed (C++ Screen.cc:187)
        if self.window.refresh() {
            self.refresh_tty(caps);
            true
        } else {
            false
        }
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
    fn cell(ch: u8, color: u8) -> Attrib {
        ((color as u16) << 8) | ch as u16
    }
    #[test]
    fn color_change_and_reset() {
        let w = 3;
        let h = 1;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(b'X', 0x80);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(s.contains("\u{1b}[1;"));
        let code = get_color_code(0x07, true);
        assert_eq!(code, "\u{1b}[0m");
    }
    #[test]
    fn skip_bottom_right() {
        let w = 2;
        let h = 2;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[w * h - 1] = cell(b'Z', 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(!s.contains("\u{1b}[2;2H"));
    }
    #[test]
    fn minimal_cursoring() {
        let w = 3;
        let h = 1;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0);
        next[1] = cell(b'B', 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(s.contains("\u{1b}[1;1H"));
        assert!(!s.contains("\u{1b}[1;2H"));
    }
    #[test]
    fn planner_detects_up_by_one() {
        let w = 4;
        let h = 4;
        let ry = 1;
        let rh = 2;
        let mut last = vec![cell(b'.', 0); w * h];
        let mut next = last.clone();
        for x in 0..w {
            last[(ry + 0) * w + x] = cell(b'A', 0);
        }
        for x in 0..w {
            last[(ry + 1) * w + x] = cell(b'B', 0);
        }
        for x in 0..w {
            next[(ry + 0) * w + x] = cell(b'B', 0);
        }
        let n = plan_scroll_up(&last, &next, w, h, ry, rh);
        assert_eq!(n, Some(1));
        let ansi = emit_scroll_ansi(w, h, ry, rh, 1);
        assert!(ansi.contains("\u{1b}[2;3r"));
    }
    #[test]
    fn begins_with_home_and_ends_with_cursor_goto() {
        let w = 2;
        let h = 2;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(b'X', 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 1,
                cursor_y: 1,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(s.starts_with("\u{1b}[H"));
        assert!(s.ends_with("\u{1b}[2;2H"));
    }
    #[test]
    fn control_chars_render_as_spaces() {
        let w = 2;
        let h = 1;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(1, 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(s.contains(" "));
    }
    #[test]
    fn acs_toggle_wraps_specials() {
        let w = 2;
        let h = 1;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(0xEC, 0);
        next[1] = cell(b'X', 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: Some("[SM]"),
                rmacs: Some("[RM]"),
                set_bg_always: true,
            },
        );
        let start = s.find("[SM]").unwrap();
        let end = s.find("[RM]").unwrap();
        assert!(start < end);
    }
    #[test]
    fn acs_two_specials_then_normal() {
        let w = 4;
        let h = 1;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(0xEC, 0);
        next[1] = cell(0xED, 0);
        next[2] = cell(b'Z', 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: Some("[SM]"),
                rmacs: Some("[RM]"),
                set_bg_always: true,
            },
        );
        let i_sm = s.find("[SM]").unwrap();
        let i_rm = s.find("[RM]").unwrap();
        let i_z = s.find('Z').unwrap();
        assert!(i_sm < i_rm && i_rm < i_z);
    }
    #[test]
    fn minimal_goto_across_wrap() {
        let w = 2;
        let h = 2;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0);
        next[1] = cell(b'B', 0);
        next[2] = cell(b'C', 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: true,
            },
        );
        assert!(s.contains("\u{1b}[1;1H"));
        assert!(!s.contains("\u{1b}[1;2H"));
        assert!(!s.contains("\u{1b}[2;1H"));
    }
    #[test]
    fn bottom_right_special_no_acs_toggle() {
        let w = 2;
        let h = 2;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[w * h - 1] = cell(0xEC, 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: Some("[SM]"),
                rmacs: Some("[RM]"),
                set_bg_always: true,
            },
        );
        assert!(!s.contains("[SM]"));
        assert!(!s.contains("[RM]"));
    }
    #[test]
    fn final_cursor_then_rmacs_order() {
        let w = 3;
        let h = 2;
        let prev = vec![cell(b' ', 0); w * h];
        let mut next = prev.clone();
        next[w + 0] = cell(0xEC, 0);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 2,
                cursor_y: 0,
                smacs: Some("[SM]"),
                rmacs: Some("[RM]"),
                set_bg_always: true,
            },
        );
        let goto = format!("\u{1b}[{};{}H", 1, 3);
        let i_g = s.rfind(&goto).unwrap();
        let i_rm = s.rfind("[RM]").unwrap();
        assert!(i_g < i_rm);
        assert!(s.ends_with("[RM]"));
    }
    #[test]
    fn no_bg_when_disabled() {
        let w = 2;
        let h = 1;
        let prev = vec![cell(b' ', 0x00); w * h];
        let mut next = prev.clone();
        next[0] = cell(b'X', 0x01);
        let s = diff_to_ansi(
            &prev,
            &next,
            &DiffOptions {
                width: w,
                height: h,
                cursor_x: 0,
                cursor_y: 0,
                smacs: None,
                rmacs: None,
                set_bg_always: false,
            },
        );
        assert!(!s.contains(";40;"));
    }

    #[test]
    fn attrib_row_basic() {
        let row = vec![cell(b'H', 0x07), cell(b'i', 0x07)];
        let s = super::attrib_row_to_ansi(&row);
        assert_eq!(s, "\u{1b}[0mHi");
    }

    #[test]
    fn attrib_row_color_change() {
        let row = vec![cell(b'A', 0x02), cell(b'B', 0x01), cell(b'C', 0x02)];
        let s = super::attrib_row_to_ansi(&row);
        assert!(s.contains("\u{1b}[")); // Should have ANSI escapes
        assert!(s.contains('A'));
        assert!(s.contains('B'));
        assert!(s.contains('C'));
    }

    #[test]
    fn attrib_row_trims_trailing_spaces() {
        let row = vec![cell(b'X', 0x07), cell(b' ', 0x07), cell(b' ', 0x07)];
        let s = super::attrib_row_to_ansi(&row);
        assert_eq!(s.trim_end(), s); // Should already be trimmed
        assert!(!s.ends_with(' '));
    }

    #[test]
    fn attrib_row_control_chars_as_spaces() {
        let row = vec![cell(1, 0x07), cell(b'A', 0x07), cell(0, 0x07)];
        let s = super::attrib_row_to_ansi(&row);
        // Control chars become spaces, but trailing spaces are trimmed
        assert!(s.starts_with("\u{1b}[0m A"));
        assert!(s.contains('A'));
    }

    #[test]
    fn screen_creation() {
        let screen = Screen::new(80, 24);
        assert_eq!(screen.window.width, 80);
        assert_eq!(screen.window.height, 24);
        assert_eq!(screen.last_screen.len(), 80 * 24);
    }

    #[test]
    fn screen_refresh() {
        use crate::curses::AcsCaps;
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
