use std::os::fd::RawFd;

bitflags::bitflags! {
    pub struct Interest: i16 {
        const READ = libc::POLLIN;
        const WRITE = libc::POLLOUT;
    }
}

pub trait Selectable {
    fn fd(&self) -> RawFd;
    fn interest(&self) -> Interest;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interest_bits() {
        let both = Interest::READ | Interest::WRITE;
        assert!(both.contains(Interest::READ));
        assert!(both.contains(Interest::WRITE));
    }
}

