pub type Attrib = u16;

pub struct Scrollback {
    pub width: usize,
    pub height: usize,
    lines: usize,
    buf: Vec<Attrib>,
    canvas_off: usize,
    pub viewpoint: usize,
    pub top_line: usize,
    rows_filled: usize,
    frozen: bool,
}

impl Scrollback {
    pub fn new(width: usize, height: usize, lines: usize) -> Self {
        Self{ width, height, lines, buf: vec![0; width*lines], canvas_off:0, viewpoint:0, top_line:0, rows_filled:0, frozen:false }
    }
    pub fn set_frozen(&mut self, f: bool){ self.frozen = f; }
    pub fn canvas_ptr(&self)->usize{ self.canvas_off }
    pub fn print_line(&mut self, bytes:&[u8], color:u8){
        let screen_span=self.width*self.height; let max_canvas=self.width*(self.lines-self.height);
        if self.canvas_off>=max_canvas { const COPY:usize=250; let copy=COPY.min(self.lines-self.height); let shift=copy*self.width; self.buf.copy_within(shift..,0); self.canvas_off-=shift; if self.viewpoint>=shift{ self.viewpoint-=shift } else { self.viewpoint=0 } self.top_line+=copy; let tail=self.buf.len()-shift; for a in &mut self.buf[tail..]{ *a=0; } }
        let start = if self.rows_filled<self.height { let s=self.viewpoint + self.rows_filled*self.width; self.rows_filled+=1; s } else { self.canvas_off+=self.width; if !self.frozen { if self.viewpoint + screen_span < self.canvas_off { self.viewpoint = self.canvas_off - screen_span; } } self.viewpoint + (self.height-1)*self.width };
        for a in &mut self.buf[start..start+self.width]{ *a = ((color as u16) << 8) | b' ' as u16; }
        for (i,b) in bytes.iter().take(self.width).enumerate(){ self.buf[start+i] = ((color as u16) << 8) | (*b as u16); }
    }
    pub fn viewport_slice(&self)->&[Attrib]{ &self.buf[self.viewpoint .. self.viewpoint + self.width*self.height] }
    pub fn move_viewpoint_page(&mut self, down: bool){ let d=(self.height/2).max(1)*self.width; if down { self.viewpoint = (self.viewpoint + d).min(self.canvas_off); } else { self.viewpoint = self.viewpoint.saturating_sub(d); } }
    pub fn move_viewpoint_line(&mut self, down: bool){ let d=self.width; if down { self.viewpoint = (self.viewpoint + d).min(self.canvas_off); } else { self.viewpoint = self.viewpoint.saturating_sub(d); } }
    pub fn highlight_view(&self, line_off: usize, x: usize, len: usize) -> Vec<Attrib> { let mut v=self.viewport_slice().to_vec(); if line_off<self.height && x<self.width { let start=line_off*self.width + x; let end=(start+len).min(self.height*self.width); for a in &mut v[start..end]{ let ch=*a & 0x00FF; let mut color=(((*a)>>8) as u8) & !(0x80); let fg=color & 0x0F; let bg=(color & 0xF0)>>4; color=(fg<<4)|bg; *a=((color as u16)<<8)|ch; } } v }
}

#[cfg(test)]
mod tests{ use super::*;
    #[test] fn cleared_tail(){ let mut sb=Scrollback::new(5,2,10); sb.print_line(b"abc",0x10); let v=sb.viewport_slice(); let bytes:Vec<u8>=v[0..5].iter().map(|a| (*a&0xFF) as u8).collect(); assert_eq!(&bytes,b"abc  "); }
}

