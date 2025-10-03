use crate::scrollback::Attrib;

pub struct InputLine {
    pub width: usize,
    buf: Vec<u8>,
    pub cursor: usize,
    pub color: u8,
}

impl InputLine {
    pub fn new(width: usize, color: u8) -> Self { Self{ width, buf: Vec::new(), cursor: 0, color } }
    pub fn insert(&mut self, b: u8) { if self.buf.len() < self.width { self.buf.insert(self.cursor, b); self.cursor+=1; } }
    pub fn backspace(&mut self) { if self.cursor>0 { self.cursor-=1; self.buf.remove(self.cursor); } }
    pub fn move_left(&mut self) { if self.cursor>0 { self.cursor-=1; } }
    pub fn move_right(&mut self) { if self.cursor<self.buf.len() { self.cursor+=1; } }
    pub fn home(&mut self) { self.cursor = 0; }
    pub fn end(&mut self) { self.cursor = self.buf.len(); }
    pub fn clear(&mut self) { self.buf.clear(); self.cursor=0; }
    pub fn render(&self) -> Vec<Attrib> {
        let mut v = vec![((self.color as u16) << 8) | (b' ' as u16); self.width];
        for (i, b) in self.buf.iter().enumerate().take(self.width) { v[i] = ((self.color as u16) << 8) | (*b as u16); }
        v
    }
    pub fn take_line(&mut self) -> Vec<u8> { let s = self.buf.clone(); self.clear(); s }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_and_render() {
        let mut il = InputLine::new(10, 0x07);
        il.insert(b'a'); il.insert(b'b'); il.insert(b'c');
        il.move_left(); il.backspace(); // remove 'b'
        let v = il.render();
        let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..2], b"ac");
        assert_eq!(il.cursor, 1);
        let line = il.take_line();
        assert_eq!(line, b"ac");
    }
}

