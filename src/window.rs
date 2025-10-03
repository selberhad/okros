use crate::scrollback::Attrib;

#[derive(Debug)]
pub struct Window {
    pub width: usize,
    pub height: usize,
    pub canvas: Vec<Attrib>,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl Window {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, canvas: vec![0; width*height], cursor_x: 0, cursor_y: 0 }
    }

    pub fn clear(&mut self, color: u8) {
        let fill = ((color as u16) << 8) | (b' ' as u16);
        for a in &mut self.canvas { *a = fill; }
    }

    pub fn clear_line(&mut self, y: usize, color: u8) {
        if y >= self.height { return; }
        let fill = ((color as u16) << 8) | (b' ' as u16);
        let off = y * self.width;
        for a in &mut self.canvas[off..off+self.width] { *a = fill; }
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.width.saturating_sub(1));
        self.cursor_y = y.min(self.height.saturating_sub(1));
    }

    pub fn put_char(&mut self, x: usize, y: usize, ch: u8, color: u8) {
        if x >= self.width || y >= self.height { return; }
        let off = y * self.width + x;
        self.canvas[off] = ((color as u16) << 8) | (ch as u16);
    }

    pub fn blit(&mut self, data: &[Attrib]) {
        if data.len() == self.canvas.len() {
            self.canvas.copy_from_slice(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn put_and_clear() {
        let mut w = Window::new(3,2);
        w.clear(0x07);
        w.put_char(0,0,b'H',0x07);
        w.put_char(1,0,b'i',0x07);
        assert_eq!((w.canvas[0] & 0xFF) as u8, b'H');
        assert_eq!((w.canvas[1] & 0xFF) as u8, b'i');
        w.clear_line(0,0x07);
        assert_eq!((w.canvas[0] & 0xFF) as u8, b' ');
    }
}

