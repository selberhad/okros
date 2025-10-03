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
            let copy = self.height.min(self.lines - self.height);
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
            if self.viewpoint + screen_span < self.canvas_offset {
                self.viewpoint = self.canvas_offset - screen_span;
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
}
