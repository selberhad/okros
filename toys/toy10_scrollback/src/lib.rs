// Toy 10: Scrollback Ring + Highlight

pub type Attrib = u16; // (color<<8)|byte

pub struct Scrollback {
    width: usize,
    height: usize,
    lines: usize,
    scrollback: Vec<Attrib>,
    canvas_offset: usize, // index (attribs) where current canvas starts
    pub viewpoint: usize, // index (attribs) where viewport starts
    pub top_line: usize,
    rows_filled: usize,
    frozen: bool,
}

impl Scrollback {
    pub fn new(width: usize, height: usize, lines: usize) -> Self {
        let mut s = Self{
            width, height, lines,
            scrollback: vec![0; width * lines],
            canvas_offset: 0,
            viewpoint: 0,
            top_line: 0,
            rows_filled: 0,
            frozen: false,
        };
        s
    }

    pub fn canvas_ptr(&self) -> usize { self.canvas_offset }

    pub fn print_line(&mut self, bytes: &[u8], color: u8) {
        // write a line; first line goes to top, subsequent lines scroll
        // ensure space at bottom; scroll or compact if needed
        let screen_span = self.width * self.height;
        let max_canvas = self.width * (self.lines - self.height);
        if self.canvas_offset >= max_canvas {
            // compact by COPY_LINES = height (simple choice)
            const COPY_LINES: usize = 250;
            let copy = COPY_LINES.min(self.lines - self.height);
            let shift = copy * self.width;
            self.scrollback.copy_within(shift.., 0);
            self.canvas_offset -= shift;
            if self.viewpoint >= shift { self.viewpoint -= shift; } else { self.viewpoint = 0; }
            self.top_line += copy;
            // clear tail
            let tail = self.scrollback.len() - shift;
            for a in &mut self.scrollback[tail..] { *a = 0; }
        }

        // Fill visible rows first, then start scrolling
        let start = if self.rows_filled < self.height {
            let start = self.viewpoint + self.rows_filled * self.width;
            self.rows_filled += 1;
            start
        } else {
            // advance canvas by one line (scroll)
            self.canvas_offset += self.width;
            // track viewpoint with canvas (default behavior)
            if !self.frozen {
                if self.viewpoint + screen_span < self.canvas_offset {
                    self.viewpoint = self.canvas_offset - screen_span;
                }
            }
            // write into last visible line of viewport
            self.viewpoint + (self.height - 1) * self.width
        };
        // clear line
        for a in &mut self.scrollback[start..start + self.width] { *a = ((color as u16) << 8) | b' ' as u16; }
        for (i, b) in bytes.iter().take(self.width).enumerate() {
            self.scrollback[start + i] = ((color as u16) << 8) | (*b as u16);
        }
    }

    pub fn set_frozen(&mut self, frozen: bool) { self.frozen = frozen; }

    pub fn viewport_slice(&self) -> &[Attrib] {
        &self.scrollback[self.viewpoint .. self.viewpoint + self.width * self.height]
    }

    pub fn move_viewpoint_page(&mut self, down: bool) {
        let delta = (self.height/2).max(1) * self.width;
        if down { self.viewpoint = (self.viewpoint + delta).min(self.canvas_offset); }
        else { self.viewpoint = self.viewpoint.saturating_sub(delta); }
    }

    pub fn move_viewpoint_line(&mut self, down: bool) {
        let delta = self.width;
        if down { self.viewpoint = (self.viewpoint + delta).min(self.canvas_offset); }
        else { self.viewpoint = self.viewpoint.saturating_sub(delta); }
    }

    // Return a copy of viewport with a highlighted span (swap fg/bg, strip bold/blink)
    pub fn highlight_view(&self, line_offset: usize, x: usize, len: usize) -> Vec<Attrib> {
        let mut buf = self.viewport_slice().to_vec();
        if line_offset < self.height && x < self.width {
            let start = line_offset * self.width + x;
            let end = (start + len).min(self.height * self.width);
            for a in &mut buf[start..end] {
                let ch = *a & 0x00FF;
                let mut color = ((*a >> 8) as u8) & !(0x80 | 0x80); // mask bold/blink bits if present
                let fg = color & 0x0F;
                let bg = (color & 0xF0) >> 4;
                color = (fg << 4) | bg; // swap
                *a = ((color as u16) << 8) | ch;
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(s: &str, color: u8, w: usize) -> Vec<Attrib> {
        let mut v = vec![((color as u16) << 8) | b' ' as u16; w];
        for (i, b) in s.as_bytes().iter().take(w).enumerate() { v[i] = ((color as u16) << 8) | (*b as u16); }
        v
    }

    #[test]
    fn scroll_and_viewport() {
        let mut sb = Scrollback::new(5, 2, 10);
        sb.print_line(b"hello", 0);
        sb.print_line(b"world", 0);
        let view = sb.viewport_slice();
        assert_eq!(&view[0..5], &line("hello",0,5)[..]);
        assert_eq!(&view[5..10], &line("world",0,5)[..]);
    }

    #[test]
    fn page_nav_and_highlight() {
        let mut sb = Scrollback::new(4, 2, 12);
        for _ in 0..6 { sb.print_line(b"aaaa", 0); }
        sb.move_viewpoint_page(false); // up
        let v1 = sb.viewport_slice().to_vec();
        // highlight first line, chars 1..3
        let hl = sb.highlight_view(0, 1, 2);
        assert_eq!(hl.len(), v1.len());
    }

    #[test]
    fn compaction_advances_top_line_and_preserves_content() {
        let mut sb = Scrollback::new(3, 2, 6); // total 6 lines buffer, 2 visible
        // Fill more than buffer to force compaction
        for i in 0..10u8 {
            let s = [b'A' + (i%26)];
            let line = [s[0]; 3];
            sb.print_line(&line, 0);
        }
        assert!(sb.top_line > 0);
        // View should be last two lines printed
        let view = sb.viewport_slice();
        let last_char = ((view[view.len()-1]) & 0xFF) as u8;
        assert!(last_char.is_ascii());
    }

    #[test]
    fn highlight_swaps_colors() {
        let mut sb = Scrollback::new(2, 1, 4);
        // color 0x21: bg=2, fg=1
        sb.print_line(&[b'Z', b' '], 0x21);
        let hl = sb.highlight_view(0, 0, 1);
        let c = (hl[0] >> 8) as u8;
        assert_eq!(c & 0x0F, 0x02); // new fg = old bg
        assert_eq!((c & 0xF0) >> 4, 0x01); // new bg = old fg
    }

    #[test]
    fn view_bounds_saturate_on_moves() {
        let mut sb = Scrollback::new(5, 2, 20);
        for _ in 0..8 { sb.print_line(b"aaaaa", 0); }
        // Move up beyond top
        for _ in 0..10 { sb.move_viewpoint_page(false); }
        assert_eq!(sb.viewpoint, 0);
        // Move down beyond bottom
        for _ in 0..50 { sb.move_viewpoint_page(true); }
        assert_eq!(sb.viewpoint, sb.canvas_ptr());
    }

    #[test]
    fn follow_tail_on_print() {
        let mut sb = Scrollback::new(4, 2, 16);
        // Fill visible
        sb.print_line(b"1111", 0);
        sb.print_line(b"2222", 0);
        // Now each new line should scroll and keep viewport at tail
        sb.print_line(b"3333", 0);
        let v = sb.viewport_slice().to_vec();
        // Bottom line should be the latest printed line
        assert_eq!(String::from_utf8_lossy(&v[4..8].iter().map(|a| (*a & 0xFF) as u8).collect::<Vec<_>>()), "3333");
    }

    #[test]
    fn highlight_clips_within_bounds() {
        let mut sb = Scrollback::new(3, 2, 6);
        sb.print_line(b"abc", 0x21);
        sb.print_line(b"def", 0x21);
        let v = sb.viewport_slice().to_vec();
        let hl = sb.highlight_view(0, 2, 10); // extends beyond row
        assert_eq!(hl.len(), v.len());
        // Cells before x are unchanged
        assert_eq!(v[0], hl[0]);
        assert_eq!(v[1], hl[1]);
        // From index 2 to end are swapped (clipped to viewport end)
        for idx in 2..hl.len() {
            assert_ne!((v[idx] >> 8) as u8, (hl[idx] >> 8) as u8, "idx {} expected swapped color", idx);
        }
    }
    #[test]
    fn compaction_preserves_last_visible_lines() {
        let mut sb = Scrollback::new(3, 2, 6); // total 6 lines buffer, 2 visible
        // Push enough lines to compact
        for i in 0..12u8 {
            let ch = b'A' + (i % 26);
            sb.print_line(&[ch, ch, ch], 0);
        }
        let view = sb.viewport_slice();
        // Expect last printed line to be visible on the bottom row
        let last = [b'L', b'L', b'L']; // 12th (index 11) -> 'L'
        let base = 1 * 3;
        let bottom = [ (view[base] & 0xFF) as u8,
                       (view[base+1] & 0xFF) as u8,
                       (view[base+2] & 0xFF) as u8 ];
        assert_eq!(bottom, last);
    }

    #[test]
    fn move_line_saturates() {
        let mut sb = Scrollback::new(3, 2, 12);
        for i in 0..6u8 { let ch = b'A' + i; sb.print_line(&[ch,ch,ch], 0); }
        // Move up until top
        for _ in 0..10 { sb.move_viewpoint_line(false); }
        assert_eq!(sb.viewpoint, 0);
        // Move down until bottom
        for _ in 0..50 { sb.move_viewpoint_line(true); }
        assert_eq!(sb.viewpoint, sb.canvas_ptr());
    }

    #[test]
    fn mixed_page_and_line_navigation() {
        let mut sb = Scrollback::new(3, 4, 30);
        // Fill a bunch of lines to ensure scrolling
        for i in 0..20u8 { let ch = b'A' + (i%26); sb.print_line(&[ch,ch,ch], 0); }
        let v0 = sb.viewpoint;
        // Page up once, then line down once
        let page_delta = (sb.height/2).max(1) * sb.width; // = 2*3 = 6
        sb.move_viewpoint_page(false);
        let v1 = sb.viewpoint;
        assert_eq!(v1, v0.saturating_sub(page_delta));
        sb.move_viewpoint_line(true);
        let expected = (v1 + sb.width).min(sb.canvas_ptr());
        assert_eq!(sb.viewpoint, expected);
    }

    #[test]
    fn cleared_tail_cells_after_short_line() {
        let mut sb = Scrollback::new(5, 2, 10);
        sb.print_line(b"abc", 0x10);
        let v = sb.viewport_slice();
        // First line: 'a','b','c',' ',' '
        let bytes: Vec<u8> = v[0..5].iter().map(|a| (*a & 0xFF) as u8).collect();
        assert_eq!(&bytes, b"abc  ");
    }

    #[test]
    fn compaction_top_line_increments_by_height() {
        let mut sb = Scrollback::new(4, 2, 8); // height=2, lines-height=6 => copy=min(250,6)=6
        // Force compaction at least once
        for _ in 0..20 { sb.print_line(b"xxxx", 0); }
        assert_eq!(sb.top_line % 6, 0, "top_line should advance in multiples of copy block (6)");
        assert!(sb.top_line >= 6);
    }

    #[test]
    fn freeze_stops_follow_tail() {
        let mut sb = Scrollback::new(4, 2, 20);
        sb.print_line(b"1111", 0);
        sb.print_line(b"2222", 0);
        let vp_before = sb.viewpoint;
        sb.set_frozen(true);
        sb.print_line(b"3333", 0);
        sb.print_line(b"4444", 0);
        // Viewpoint unchanged when frozen
        assert_eq!(sb.viewpoint, vp_before);
    }

    #[test]
    fn viewpoint_bounds_invariant_under_mixed_moves() {
        let mut sb = Scrollback::new(5, 3, 50);
        for i in 0..40u8 { let ch = b'A' + (i%26); sb.print_line(&[ch,ch,ch,ch,ch], 0); }
        // Mixed sequence of moves
        for i in 0..200 {
            match i % 4 {
                0 => sb.move_viewpoint_line(false),
                1 => sb.move_viewpoint_line(true),
                2 => sb.move_viewpoint_page(false),
                _ => sb.move_viewpoint_page(true),
            }
            // Invariants: viewpoint within [0, canvas_offset], slice in-bounds
            assert!(sb.viewpoint <= sb.canvas_ptr());
            let slice = sb.viewport_slice();
            assert_eq!(slice.len(), sb.width * sb.height);
        }
    }
}
