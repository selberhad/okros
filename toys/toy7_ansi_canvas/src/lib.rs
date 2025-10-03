// Toy 7: ANSI Canvas Diff

pub type Attrib = u16; // (color << 8) | byte

const FG_BOLD: u8 = 1 << 7; // mirrors fg_bold bit placement from reference
pub const SC_BASE: u8 = 0xEC;
pub const SC_END: u8 = SC_BASE + 8; // arbitrary window like reference

fn reverse_color_conv_table(idx: u8) -> u8 {
    match idx & 0x07 {
        0 => 0, 1 => 4, 2 => 2, 3 => 6, 4 => 1, 5 => 5, 6 => 3, _ => 7,
    }
}

pub fn get_color_code(color: u8, set_bg: bool) -> String {
    let fg = 30 + reverse_color_conv_table(color & 0x07) as i32;
    let bold = (color & FG_BOLD) != 0;
    let bg = 40 + reverse_color_conv_table((color >> 4) & 0x07) as i32;
    if fg == 37 && bg == 40 && !bold {
        return "\u{1b}[0m".to_string();
    }
    let bg_part = if set_bg { format!("{};", bg) } else { String::new() };
    if bold {
        format!("\u{1b}[1;{}{}m", bg_part, fg)
    } else {
        format!("\u{1b}[0;{}{}m", bg_part, fg)
    }
}

fn vt_home() -> &'static str { "\u{1b}[H" }
fn vt_goto(y1: usize, x1: usize) -> String { format!("\u{1b}[{};{}H", y1, x1) }

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
        Self { width: 0, height: 0, cursor_x: 0, cursor_y: 0, smacs: None, rmacs: None, set_bg_always: true }
    }
}

pub fn diff_to_ansi(prev: &[Attrib], next: &[Attrib], opt: &DiffOptions) -> String {
    assert_eq!(prev.len(), next.len());
    assert_eq!(prev.len(), opt.width * opt.height);
    let mut out = String::new();
    out.push_str(vt_home());
    let mut saved_color: i32 = -1;
    let mut last_x = 1usize;
    let mut last_y = 1usize;
    let mut acs_enabled = false;

    for y in 0..opt.height {
        for x in 0..opt.width {
            if y == opt.height - 1 && x == opt.width - 1 { continue; }
            let idx = y * opt.width + x;
            if prev[idx] == next[idx] { continue; }
            let color = (next[idx] >> 8) as u8;
            let ch = (next[idx] & 0xFF) as u8;
            if (color as i32) != saved_color {
                out.push_str(&get_color_code(color, opt.set_bg_always));
                saved_color = color as i32;
            }

            if !(last_y == y + 1 && last_x == x + 1 && saved_color == color as i32) {
                out.push_str(&vt_goto(y + 1, x + 1));
            }
            last_y = y + 1;
            last_x = x + 2;
            if last_x > opt.width { last_x = 1; last_y += 1; }

            if ch >= SC_BASE && ch < SC_END {
                if !acs_enabled {
                    if let Some(s) = opt.smacs { out.push_str(s); }
                    acs_enabled = true;
                }
                // For tests, print placeholder for special char
                out.push('#');
            } else {
                if acs_enabled {
                    if let Some(r) = opt.rmacs { out.push_str(r); }
                    acs_enabled = false;
                }
                out.push(if ch >= 32 { ch as char } else { ' ' });
            }
        }
    }

    out.push_str(&vt_goto(opt.cursor_y + 1, opt.cursor_x + 1));
    if acs_enabled {
        if let Some(r) = opt.rmacs { out.push_str(r); }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(ch: u8, color: u8) -> Attrib { ((color as u16) << 8) | ch as u16 }

    #[test]
    fn color_change_emits_code() {
        let w = 3; let h = 2;
        let mut prev = vec![cell(b' ', 0); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0x00); // same color
        next[1] = cell(b'B', 0x10); // bg change
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        // Expect at least two color codes present (count 'm' terminators)
        let color_codes = s.chars().filter(|&c| c == 'm').count();
        assert!(color_codes >= 2, "expected >=2 color code sequences, got {} in: {}", color_codes, s);
    }

    #[test]
    fn acs_toggle_wraps_specials() {
        let w=2; let h=1;
        let prev = vec![cell(b' ', 0); w*h];
        let mut next = prev.clone();
        next[0] = cell(SC_BASE, 0);
        next[1] = cell(b'X', 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:Some("[SM]"), rmacs:Some("[RM]"), set_bg_always:true });
        let start = s.find("[SM]").expect("smacs present");
        let end = s.find("[RM]").expect("rmacs present");
        assert!(start < end);
    }

    #[test]
    fn begins_with_vt_home_and_ends_with_cursor_goto() {
        let w=2; let h=2;
        let prev = vec![cell(b' ', 0); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'X', 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:1, cursor_y:1, smacs:None, rmacs:None, set_bg_always:true });
        assert!(s.starts_with("\u{1b}[H"));
        assert!(s.ends_with("\u{1b}[2;2H"));
    }

    #[test]
    fn skip_bottom_right_cell() {
        let w=3; let h=3;
        let prev = vec![cell(b' ', 0); w*h];
        let mut next = prev.clone();
        next[w*h-1] = cell(b'Z', 0); // bottom-right
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        // should not move to bottom-right
        assert!(!s.contains("\u{1b}[3;3H"), "unexpected write to bottom-right: {}", s);
    }

    #[test]
    fn white_on_black_maps_to_reset() {
        // When color is fg=white(7), bg=black(0), no bold → 0m, but our reverse map makes fg 37/bg 40.
        // get_color_code should collapse that to CSI 0m.
        let code = get_color_code(0x07, true);
        assert_eq!(code, "\u{1b}[0m");
    }
}
