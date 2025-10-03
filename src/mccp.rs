pub trait Decompressor {
    fn receive(&mut self, input: &[u8]);
    fn pending(&self) -> bool;
    fn take_output(&mut self) -> Vec<u8>;
    fn error(&self) -> bool { false }
    fn response(&mut self) -> Option<Vec<u8>> { None }
}

pub struct PassthroughDecomp{ buf: Vec<u8> }
impl PassthroughDecomp{ pub fn new()->Self{ Self{ buf:Vec::new() } } }
impl Decompressor for PassthroughDecomp{ fn receive(&mut self,i:&[u8]){ self.buf.extend_from_slice(i) } fn pending(&self)->bool{ !self.buf.is_empty() } fn take_output(&mut self)->Vec<u8>{ std::mem::take(&mut self.buf) } }

pub mod telopt{ pub const IAC:u8=255; pub const WILL:u8=251; pub const DO:u8=253; pub const DONT:u8=254; pub const SB:u8=250; pub const SE:u8=240; pub const COMPRESS:u8=85; pub const COMPRESS2:u8=86; }

pub struct MccpStub{ residual:Vec<u8>, out:Vec<u8>, responses:Vec<u8>, got_v2:bool, compressing:bool, error:bool }
impl MccpStub{ pub fn new()->Self{ Self{ residual:Vec::new(), out:Vec::new(), responses:Vec::new(), got_v2:false, compressing:false, error:false } } }
impl Decompressor for MccpStub{
    fn receive(&mut self,input:&[u8]){ use telopt::*; self.residual.extend_from_slice(input); let mut i=0usize; while i<self.residual.len(){ let b=self.residual[i]; if !self.compressing { if b!=IAC { self.out.push(b); i+=1; continue; } if i+1>=self.residual.len(){ break; } let b1=self.residual[i+1]; if b1==IAC{ self.out.push(IAC); i+=2; continue; } if b1==WILL{ if i+2>=self.residual.len(){ break; } let opt=self.residual[i+2]; if opt==COMPRESS2{ self.responses.extend_from_slice(&[IAC,DO,COMPRESS2]); self.got_v2=true; i+=3; continue; } if opt==COMPRESS{ if self.got_v2{ self.responses.extend_from_slice(&[IAC,DONT,COMPRESS]); } else { self.responses.extend_from_slice(&[IAC,DO,COMPRESS]); } i+=3; continue; } } if b1==SB{ if i+4>=self.residual.len(){ break; } let opt=self.residual[i+2]; if (opt==COMPRESS && self.residual[i+3]==WILL && self.residual[i+4]==SE) || (opt==COMPRESS2 && self.residual[i+3]==IAC && self.residual[i+4]==SE){ self.compressing=true; i+=5; continue; } } self.out.push(b); i+=1; continue; } else { // compressing → pass-through in stub
            self.out.push(b); i+=1; }
        } if i>0{ self.residual.drain(0..i);} }
    fn pending(&self)->bool{ !self.error && !self.out.is_empty() }
    fn take_output(&mut self)->Vec<u8>{ std::mem::take(&mut self.out) }
    fn error(&self)->bool{ self.error }
    fn response(&mut self)->Option<Vec<u8>>{ if self.responses.is_empty(){None}else{Some(std::mem::take(&mut self.responses))} }
}

#[cfg(feature="mccp")]
pub struct MccpInflate{ residual:Vec<u8>, out:Vec<u8>, responses:Vec<u8>, got_v2:bool, compressing:bool, error:bool, comp:usize, uncomp:usize, dec:Option<flate2::Decompress> }
#[cfg(feature="mccp")]
impl MccpInflate{ pub fn new()->Self{ Self{ residual:Vec::new(), out:Vec::new(), responses:Vec::new(), got_v2:false, compressing:false, error:false, comp:0, uncomp:0, dec:None } } pub fn stats(&self)->(usize,usize){(self.comp,self.uncomp)} }
#[cfg(feature="mccp")]
impl Decompressor for MccpInflate{ fn receive(&mut self,input:&[u8]){ use telopt::*; self.residual.extend_from_slice(input); let mut i=0usize; while i<self.residual.len(){ let b=self.residual[i]; if !self.compressing { if b!=IAC { self.out.push(b); i+=1; continue; } if i+1>=self.residual.len(){ break; } let b1=self.residual[i+1]; if b1==IAC{ self.out.push(IAC); i+=2; continue; } if b1==WILL{ if i+2>=self.residual.len(){ break; } let opt=self.residual[i+2]; if opt==COMPRESS2{ self.responses.extend_from_slice(&[IAC,DO,COMPRESS2]); self.got_v2=true; i+=3; continue; } if opt==COMPRESS{ if self.got_v2{ self.responses.extend_from_slice(&[IAC,DONT,COMPRESS]); } else { self.responses.extend_from_slice(&[IAC,DO,COMPRESS]); } i+=3; continue; } } if b1==SB{ if i+4>=self.residual.len(){ break; } let opt=self.residual[i+2]; if (opt==COMPRESS && self.residual[i+3]==WILL && self.residual[i+4]==SE) || (opt==COMPRESS2 && self.residual[i+3]==IAC && self.residual[i+4]==SE){ self.compressing=true; self.dec=Some(flate2::Decompress::new(true)); i+=5; continue; } } self.out.push(b); i+=1; continue; } else { let dec=self.dec.as_mut().unwrap(); let in_data=&self.residual[i..]; let out_start=self.out.len(); self.out.resize(out_start + in_data.len().max(64), 0); let in_before=dec.total_in(); let out_before=dec.total_out(); let res=dec.decompress(in_data, &mut self.out[out_start..], flate2::FlushDecompress::None); match res{ Ok(status)=>{ let used=(dec.total_in()-in_before) as usize; let prod=(dec.total_out()-out_before) as usize; self.comp+=used; self.uncomp+=prod; i+=used; self.out.truncate(out_start+prod); if status==flate2::Status::StreamEnd { self.compressing=false; self.dec=None; } if used==0 && prod==0 { break; } }, Err(_)=>{ self.error=true; break; } } } } if i>0{ self.residual.drain(0..i);} }
    fn pending(&self)->bool{ !self.error && !self.out.is_empty() }
    fn take_output(&mut self)->Vec<u8>{ std::mem::take(&mut self.out) }
    fn error(&self)->bool{ self.error }
    fn response(&mut self)->Option<Vec<u8>>{ if self.responses.is_empty(){None}else{Some(std::mem::take(&mut self.responses))} }
}

