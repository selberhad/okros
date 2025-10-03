use crate::scrollback::Attrib;

pub struct StatusLine {
    pub width: usize,
    text: String,
    pub color: u8,
}

impl StatusLine {
    pub fn new(width: usize, color: u8) -> Self {
        Self {
            width,
            text: String::new(),
            color,
        }
    }
    pub fn set_text<S: Into<String>>(&mut self, s: S) {
        self.text = s.into();
    }
    pub fn render(&self) -> Vec<Attrib> {
        let mut v = vec![((self.color as u16) << 8) | (b' ' as u16); self.width];
        for (i, b) in self.text.as_bytes().iter().enumerate().take(self.width) {
            v[i] = ((self.color as u16) << 8) | (*b as u16);
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_and_render() {
        let mut sl = StatusLine::new(8, 0x07);
        sl.set_text("READY");
        let v = sl.render();
        let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..5], b"READY");
    }
}
