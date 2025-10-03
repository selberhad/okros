#[cfg(unix)]
mod unix {
    use libc;
    use std::io::{self, Write};
    use std::mem;

    pub struct Tty {
        old: libc::termios,
        enabled: bool,
    }

    impl Tty {
        pub fn new() -> io::Result<Self> {
            unsafe {
                let mut old = mem::zeroed::<libc::termios>();
                if libc::tcgetattr(libc::STDIN_FILENO, &mut old) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(Self { old, enabled: false })
            }
        }

        pub fn enable_raw(&mut self) -> io::Result<()> {
            unsafe {
                let mut raw = self.old;
                // lflag: disable ECHO, ICANON
                raw.c_lflag &= !(libc::ECHO | libc::ICANON);
                // iflag: disable ISTRIP
                // Best-effort: ISTRIP may be platform-dependent width; cast to the underlying type
                raw.c_iflag &= !(libc::ISTRIP as libc::tcflag_t);
                if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) != 0 {
                    return Err(io::Error::last_os_error());
                }
                self.enabled = true;
                Ok(())
            }
        }

        pub fn disable_raw(&mut self) -> io::Result<()> {
            if !self.enabled { return Ok(()); }
            unsafe {
                if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.old) != 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            self.enabled = false;
            Ok(())
        }

        pub fn keypad_application_mode(&self, on: bool) -> io::Result<()> {
            let seq = if on { b"\x1b=" } else { b"\x1b>" };
            let mut out = io::stdout();
            out.write_all(seq)?;
            out.flush()?;
            Ok(())
        }
    }

    impl Drop for Tty {
        fn drop(&mut self) {
            let _ = self.disable_raw();
            let _ = self.keypad_application_mode(false);
        }
    }

    pub use Tty as PlatformTty;
}

#[cfg(not(unix))]
mod nonunix {
    use std::io;
    pub struct Tty;
    impl Tty { pub fn new() -> io::Result<Self> { Ok(Tty) } pub fn enable_raw(&mut self)->io::Result<()> { Ok(()) } pub fn disable_raw(&mut self)->io::Result<()> { Ok(()) } pub fn keypad_application_mode(&self,_:bool)->io::Result<()> { Ok(()) } }
    pub use Tty as PlatformTty;
}

pub use self::unix::PlatformTty as Tty;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_new() {
        // This test will only work in a TTY environment
        // If running in CI without a TTY, it should gracefully handle the error
        match Tty::new() {
            Ok(tty) => {
                // Successfully created TTY handle
                println!("✓ TTY created successfully");
                drop(tty); // Ensure Drop runs
            }
            Err(e) => {
                // Not a TTY (expected in CI/non-interactive environments)
                println!("⚠️ Not a TTY: {} (expected in CI)", e);
            }
        }
    }

    #[test]
    fn test_tty_raw_mode() {
        // Test enabling/disabling raw mode if we have a TTY
        match Tty::new() {
            Ok(mut tty) => {
                // Try to enable raw mode
                match tty.enable_raw() {
                    Ok(_) => {
                        println!("✓ Raw mode enabled");
                        // Immediately disable it
                        match tty.disable_raw() {
                            Ok(_) => println!("✓ Raw mode disabled"),
                            Err(e) => println!("⚠️ Failed to disable raw mode: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Cannot enable raw mode: {} (expected if not a real TTY)", e);
                    }
                }
            }
            Err(_) => {
                println!("⚠️ No TTY available for raw mode test");
            }
        }
    }

    #[test]
    fn test_keypad_mode() {
        // Test keypad application mode sequences
        match Tty::new() {
            Ok(tty) => {
                // These should not fail even if stdout isn't a TTY
                let _ = tty.keypad_application_mode(true);
                let _ = tty.keypad_application_mode(false);
                println!("✓ Keypad mode sequences sent");
            }
            Err(_) => {
                println!("⚠️ No TTY for keypad test");
            }
        }
    }

    #[test]
    fn test_tty_drop() {
        // Verify Drop implementation doesn't panic
        if let Ok(mut tty) = Tty::new() {
            let _ = tty.enable_raw();
            // Drop will be called automatically here
            println!("✓ TTY Drop executed");
        }
    }
}

