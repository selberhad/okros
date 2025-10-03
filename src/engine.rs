use crate::mccp::Decompressor;
use crate::session::Session;
use crate::scrollback::Attrib;

pub struct SessionEngine<D: Decompressor> {
    pub session: Session<D>,
    attached: bool,
}

impl<D: Decompressor> SessionEngine<D> {
    pub fn new(decomp: D, width: usize, height: usize, lines: usize) -> Self {
        Self { session: Session::new(decomp, width, height, lines), attached: true }
    }

    pub fn detach(&mut self) { self.attached = false; }
    pub fn attach(&mut self) { self.attached = true; }
    pub fn is_attached(&self) -> bool { self.attached }

    pub fn feed_inbound(&mut self, chunk: &[u8]) {
        // Even if detached, we continue processing and buffering into scrollback
        self.session.feed(chunk);
    }

    pub fn viewport_text(&self) -> Vec<String> {
        let width = self.session.scrollback.width;
        let height = self.session.scrollback.height;
        let slice = self.session.scrollback.viewport_slice();
        let mut out = Vec::with_capacity(height);
        for row in 0..height {
            let off = row * width;
            let row_slice = &slice[off .. off + width];
            let mut bytes: Vec<u8> = row_slice.iter().map(|a| (a & 0xFF) as u8).collect();
            // trim trailing spaces
            while bytes.last() == Some(&b' ') { bytes.pop(); }
            out.push(String::from_utf8_lossy(&bytes).to_string());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccp::PassthroughDecomp;

    #[test]
    fn engine_detached_buffers_and_attach_reads() {
        let mut eng = SessionEngine::new(PassthroughDecomp::new(), 10, 3, 100);
        eng.detach();
        eng.feed_inbound(b"abc\n");
        assert!(!eng.is_attached());
        eng.attach();
        let rows = eng.viewport_text();
        assert!(rows.iter().any(|r| r == "abc"));
    }
}

