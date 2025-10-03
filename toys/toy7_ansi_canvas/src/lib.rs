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

// Experimental: detect simple upward scroll within a region by N lines
pub fn plan_scroll_up(last: &[Attrib], next: &[Attrib], width: usize, height: usize, reg_y: usize, reg_h: usize) -> Option<usize> {
    if reg_y + reg_h > height { return None; }
    for n in 1..reg_h { // try scroll up by n lines
        let mut ok = true;
        for row in 0..(reg_h - n) {
            let ly = reg_y + row + n;
            let ny = reg_y + row;
            let l_off = ly * width; let n_off = ny * width;
            if &last[l_off..l_off+width] != &next[n_off..n_off+width] { ok = false; break; }
        }
        if ok { return Some(n); }
    }
    None
}

pub fn emit_scroll_ansi(width: usize, height: usize, reg_y: usize, reg_h: usize, lines: usize) -> String {
    // CSI y1;y2 r, goto bottom-left, N newlines, reset region to full screen
    let y1 = reg_y + 1;
    let y2 = reg_y + reg_h;
    let mut s = String::new();
    s.push_str(&format!("\u{1b}[{};{}r", y1, y2));
    s.push_str(&format!("\u{1b}[{};{}H", y2, 1));
    for _ in 0..lines { s.push('\n'); }
    s.push_str(&format!("\u{1b}[{};{}r", 1, height));
    s
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

    #[test]
    fn same_frame_noop_only_cursor_moves() {
        let w = 3; let h = 2;
        let prev = vec![cell(b'X', 0x00); w*h];
        let next = prev.clone();
        let opt = DiffOptions{ width:w, height:h, cursor_x:2, cursor_y:1, smacs:None, rmacs:None, set_bg_always:true };
        let s = diff_to_ansi(&prev, &next, &opt);
        // Expect only home + final cursor goto
        assert!(s.starts_with("\u{1b}[H"));
        assert!(s.ends_with("\u{1b}[2;3H"));
        // No printable chars between
        let inner = &s["\u{1b}[H".len()..s.len()-"\u{1b}[2;3H".len()];
        assert!(inner.is_empty());
    }

    #[test]
    fn fg_change_without_bg_when_disabled() {
        let w=2; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'X', 0x01); // change only fg
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:false });
        // Should not contain explicit background code like 40;
        assert!(!s.contains(";40;"), "unexpected background code in: {}", s);
    }

    #[test]
    fn bold_flag_emits_csi_1() {
        let w=1; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'X', 0x80); // bold bit
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        assert!(s.contains("\u{1b}[1;"), "expected bold CSI in: {}", s);
    }

    #[test]
    fn acs_nesting_two_specials_then_normal() {
        let w=4; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(SC_BASE, 0);
        next[1] = cell(SC_BASE+1, 0);
        next[2] = cell(b'Z', 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:Some("[SM]"), rmacs:Some("[RM]"), set_bg_always:true });
        let i_sm = s.find("[SM]").expect("smacs present");
        let i_rm = s.find("[RM]").expect("rmacs present");
        let i_hash1 = s[i_sm..].find('#').map(|k| i_sm+k).unwrap();
        let i_hash2 = s[i_hash1+1..].find('#').map(|k| i_hash1+1+k).unwrap();
        let i_z = s.find('Z').expect("Z present");
        assert!(i_sm < i_hash1 && i_hash1 < i_hash2 && i_hash2 < i_rm && i_rm < i_z, "ordering incorrect: {}", s);
    }

    #[test]
    fn minimal_cursoring_for_adjacent_cells() {
        let w=3; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0);
        next[1] = cell(b'B', 0); // adjacent same color
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        assert!(s.contains("\u{1b}[1;1H"));
        assert!(!s.contains("\u{1b}[1;2H"), "unexpected second goto: {}", s);
    }

    #[test]
    fn scroll_region_planner_detects_simple_up_by_one() {
        let w=4; let h=4; let reg_y=1; let reg_h=2;
        let mut last = vec![cell(b'.',0); w*h];
        let mut next = last.clone();
        // Fill last region rows with A and B
        for x in 0..w { last[(reg_y+0)*w + x] = cell(b'A',0); }
        for x in 0..w { last[(reg_y+1)*w + x] = cell(b'B',0); }
        // next is last scrolled up by 1: row reg_y becomes B
        for x in 0..w { next[(reg_y+0)*w + x] = cell(b'B',0); }
        // bottom line after scroll would be blank; we don't need it for detection
        let n = plan_scroll_up(&last, &next, w, h, reg_y, reg_h);
        assert_eq!(n, Some(1));
        let ansi = emit_scroll_ansi(w,h,reg_y,reg_h,1);
        assert!(ansi.contains("\u{1b}[2;3r")); // region 2..3
        assert!(ansi.contains("\u{1b}[3;1H")); // goto bottom of region
        assert!(ansi.ends_with(&format!("\n\u{1b}[1;{}r", h)));
    }

    #[test]
    fn scroll_region_planner_returns_none_when_no_match() {
        let w=3; let h=3; let reg_y=0; let reg_h=2;
        let last = vec![cell(b'A',0); w*h];
        let next = vec![cell(b'B',0); w*h];
        assert_eq!(plan_scroll_up(&last, &next, w, h, reg_y, reg_h), None);
    }

    #[test]
    fn minimal_goto_across_line_wrap() {
        // Updates at (1,1)->(2,1)->(1,2) should use only one goto
        let w=2; let h=2;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0);
        next[1] = cell(b'B', 0);
        next[2] = cell(b'C', 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        // Expect one goto to 1;1 and no extra gotos to 1;2 or 2;1
        let count_11 = s.matches("\u{1b}[1;1H").count();
        assert!(count_11 >= 1);
        assert!(!s.contains("\u{1b}[1;2H"), "unexpected goto to 1;2: {}", s);
        assert!(!s.contains("\u{1b}[2;1H"), "unexpected goto to 2;1: {}", s);
    }

    #[test]
    fn bottom_right_special_does_not_toggle_acs() {
        let w=2; let h=2;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        // Set only bottom-right to a special ACS char
        next[w*h - 1] = cell(SC_BASE, 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:Some("[SM]"), rmacs:Some("[RM]"), set_bg_always:true });
        assert!(!s.contains("[SM]"), "should not enable ACS for bottom-right only: {}", s);
        assert!(!s.contains("[RM]"), "should not disable ACS for bottom-right only: {}", s);
        assert!(!s.contains("\u{1b}[2;2H"), "should not write bottom-right cell: {}", s);
    }

    #[test]
    fn control_chars_render_as_spaces() {
        let w=1; let h=2;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(0x01, 0x00); // control char -> space
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        // There should be a literal space emitted in the output
        assert!(s.contains(" "));
    }

    #[test]
    fn acs_reset_emitted_if_last_change_is_special() {
        let w=2; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(SC_BASE, 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:1, cursor_y:0, smacs:Some("[SM]"), rmacs:Some("[RM]"), set_bg_always:true });
        let idx_sm = s.rfind("[SM]").expect("smacs present");
        let idx_rm = s.rfind("[RM]").expect("rmacs present");
        assert!(idx_sm < idx_rm, "expected rmacs after smacs at end: {}", s);
    }

    #[test]
    fn single_color_code_reused_across_adjacent_writes() {
        let w=2; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0x12);
        next[1] = cell(b'B', 0x12);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        let m_count = s.chars().filter(|&c| c=='m').count();
        assert_eq!(m_count, 1, "expected single color code, got {}: {}", m_count, s);
    }

    #[test]
    fn single_color_code_reused_across_separated_writes() {
        let w=3; let h=1;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0x22);
        next[2] = cell(b'B', 0x22);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        let m_count = s.chars().filter(|&c| c=='m').count();
        assert_eq!(m_count, 1, "expected single color code across separated writes, got {}: {}", m_count, s);
    }

    #[test]
    fn final_cursor_goto_then_rmacs_after_bottom_row_acs() {
        let w=3; let h=2;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        // Set a special at (x=1,y=2), not bottom-right
        next[w + 0] = cell(SC_BASE, 0);
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:2, cursor_y:0, smacs:Some("[SM]"), rmacs:Some("[RM]"), set_bg_always:true });
        // Ensure cursor goto appears before final rmacs, and rmacs is last
        let goto = "\u{1b}[1;3H"; // cursor_y=0 -> row 1, cursor_x=2 -> col 3
        let idx_goto = s.rfind(goto).expect("cursor goto present");
        let idx_rm = s.rfind("[RM]").expect("rmacs present");
        assert!(idx_goto < idx_rm, "expected goto before rmacs: {}", s);
        assert!(s.ends_with("[RM]"));
    }

    #[test]
    fn color_code_reused_across_multiple_rows() {
        let w=2; let h=2;
        let prev = vec![cell(b' ', 0x00); w*h];
        let mut next = prev.clone();
        next[0] = cell(b'A', 0x33);
        next[w+1] = cell(b'B', 0x33); // different row, same color
        let s = diff_to_ansi(&prev, &next, &DiffOptions{ width:w, height:h, cursor_x:0, cursor_y:0, smacs:None, rmacs:None, set_bg_always:true });
        let m_count = s.chars().filter(|&c| c=='m').count();
        assert_eq!(m_count, 1, "expected single color code across rows, got {}: {}", m_count, s);
    }
}
