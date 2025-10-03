use std::io;
use std::os::fd::RawFd;

pub const READ: i16 = libc::POLLIN;
pub const WRITE: i16 = libc::POLLOUT;

#[derive(Debug, Clone, Copy)]
pub struct Ready {
    pub revents: i16,
}

pub fn poll_fds(fds: &[(RawFd, i16)], timeout_ms: i32) -> io::Result<Vec<(RawFd, Ready)>> {
    let mut pfds: Vec<libc::pollfd> = fds
        .iter()
        .map(|(fd, ev)| libc::pollfd {
            fd: *fd,
            events: *ev,
            revents: 0,
        })
        .collect();
    let rc = unsafe {
        libc::poll(
            pfds.as_mut_ptr(),
            (pfds.len() as u64).try_into().unwrap(),
            timeout_ms,
        )
    };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut out = Vec::new();
    for p in pfds {
        if p.revents != 0 {
            out.push((p.fd, Ready { revents: p.revents }));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::fd::{FromRawFd, IntoRawFd};

    #[test]
    fn poll_pipe_readable() {
        let mut fds = [0; 2];
        unsafe {
            libc::pipe(fds.as_mut_ptr());
        }
        let r = fds[0];
        let w = fds[1];
        // write to pipe so it's readable
        let mut f = unsafe { std::fs::File::from_raw_fd(w) };
        f.write_all(b"x").unwrap();
        // Leak file descriptor back so drop doesn't close both
        let _ = f.into_raw_fd();
        let ready = poll_fds(&[(r, READ)], 100).unwrap();
        assert!(!ready.is_empty());
        unsafe {
            libc::close(r);
            libc::close(w);
        }
    }
}
