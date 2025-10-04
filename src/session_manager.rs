// SessionManager - Connection lifecycle management for Session
//
// Ported from C++ Session.cc connection management (lines 237-390)
// Wraps Session (data pipeline) with connection state, interpreter hooks, and I/O

use crate::mccp::Decompressor;
use crate::mud::Mud;
use crate::plugins::stack::Interpreter;
use crate::session::{Session, SessionState};
use crate::socket::Socket;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

const CONNECT_TIMEOUT: i64 = 30; // seconds (C++ Session.cc:21)

/// SessionManager wraps Session with connection lifecycle management
/// Corresponds to C++ Session class (Session.cc)
pub struct SessionManager<D: Decompressor> {
    pub session: Session<D>,
    socket: Option<Socket>,
    mud_name: String, // Reference to MUD name (C++ has MUD& mud)
}

impl<D: Decompressor> SessionManager<D> {
    /// Create new SessionManager (C++ Session::Session constructor, lines 237-263)
    pub fn new(decomp: D, width: usize, height: usize, lines: usize, mud_name: String) -> Self {
        Self {
            session: Session::new(decomp, width, height, lines),
            socket: None,
            mud_name,
        }
    }

    /// Initialize session with MUD and interpreter (C++ Session constructor lines 251-262)
    /// Loads MUD init file and calls sys/connect hook
    pub fn initialize(&mut self, mud: &mut Mud, interp: &mut dyn Interpreter) {
        // Load MUD init file if not already loaded (C++ lines 251-254)
        if !mud.loaded {
            mud.loaded = true;
            let mut _out = String::new();
            interp.load_file(&mud.name, true);
        }

        // Set interpreter variable "mud" to MUD name (C++ line 256)
        interp.set_str("mud", &mud.name);

        // Call sys/connect hook (C++ line 257)
        let mut _out = String::new();
        interp.run_quietly("sys/connect", "", &mut _out, true);

        self.session.state = SessionState::Disconnected;
    }

    /// Connect to MUD (C++ Session::open, lines 296-310)
    pub fn open(&mut self, mud: &mut Mud) -> io::Result<()> {
        // Create socket and connect (C++ line 297: Socket::connect)
        mud.connect()?;

        // Take ownership of socket from Mud
        if let Some(sock) = mud.sock.take() {
            self.socket = Some(sock);
            self.session.state = SessionState::Connecting;
            self.session.stats.dial_time = current_time_unix();
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Failed to create socket",
            ))
        }
    }

    /// Disconnect from MUD (C++ Session::close, lines 313-321)
    pub fn close(&mut self, interp: &mut dyn Interpreter) {
        if self.session.state != SessionState::Disconnected {
            self.session.state = SessionState::Disconnected;

            // Call sys/loselink hook (C++ line 316)
            let mut _out = String::new();
            interp.run_quietly("sys/loselink", "", &mut _out, true);

            // Clear interpreter mud variable (C++ line 319)
            interp.set_str("mud", "");

            // Close socket
            self.socket = None;
        }
    }

    /// Send data to MUD with statistics tracking (C++ Session::writeMUD, lines 323-327)
    pub fn write_mud(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(ref mut sock) = self.socket {
            // Write to socket (C++ line 324: writeLine(s))
            let fd = sock.as_raw_fd();
            let written =
                unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len()) };

            if written >= 0 {
                // Track statistics (C++ lines 325-326)
                self.session.stats.bytes_written += written as usize;
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to MUD",
            ))
        }
    }

    /// Read data from MUD socket and feed to Session pipeline
    pub fn read_mud(&mut self) -> io::Result<usize> {
        if let Some(ref mut sock) = self.socket {
            let mut buf = [0u8; 4096];
            let fd = sock.as_raw_fd();
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };

            if n > 0 {
                let n = n as usize;
                self.session.stats.bytes_read += n;
                self.session.feed(&buf[..n]);
                Ok(n)
            } else if n == 0 {
                // EOF - connection closed
                self.session.state = SessionState::Disconnected;
                Ok(0)
            } else {
                Err(io::Error::last_os_error())
            }
        } else {
            Ok(0) // Not connected
        }
    }

    /// Time-based updates, connection timeout handling (C++ Session::idle, lines 330-359)
    pub fn idle(&mut self, interp: &mut dyn Interpreter) -> Option<String> {
        if self.session.state == SessionState::Connecting {
            let elapsed = current_time_unix() - self.session.stats.dial_time;
            let time_left = CONNECT_TIMEOUT - elapsed;

            if time_left <= 0 {
                // Timeout - close connection (C++ lines 334-335)
                self.close(interp);
                return Some(format!("Connection to {} timed out", self.mud_name));
            } else {
                // Show progress bar (C++ lines 337-351)
                // Simplified version - just return status message
                return Some(format!(
                    "Connecting to {} ({} seconds remaining)",
                    self.mud_name, time_left
                ));
            }
        }
        None
    }

    /// Check if socket is writable (connection established)
    pub fn check_writable(&mut self) -> io::Result<bool> {
        if let Some(ref mut sock) = self.socket {
            if self.session.state == SessionState::Connecting {
                sock.on_writable()?;
                if sock.state == crate::socket::ConnState::Connected {
                    self.establish_connection();
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Mark connection as established (C++ Session::establishConnection, lines 369-380)
    fn establish_connection(&mut self) {
        self.session.state = SessionState::Connected;
        self.session.stats.connect_time = current_time_unix();
        // Note: C++ sends mud.commands here, but that should be done by caller
    }

    /// Get current connection state
    pub fn state(&self) -> SessionState {
        self.session.state
    }

    /// Get socket file descriptor for select/poll
    pub fn socket_fd(&self) -> Option<i32> {
        self.socket.as_ref().map(|s| s.as_raw_fd())
    }

    /// Expand macro key to command text (C++ Session::expand_macros, lines 617-637)
    /// Returns Some(macro_text) if macro found, None otherwise
    /// Caller should add macro_text to interpreter command queue with EXPAND_ALL flags
    pub fn expand_macros(
        &self,
        key: i32,
        mud: &Mud,
        macros_disabled: bool,
        echo_callback: Option<&mut dyn FnMut(&str)>,
    ) -> Option<String> {
        // Check if macros globally disabled (C++ line 618)
        if macros_disabled {
            return None;
        }

        // Find macro from MUD (C++ line 622: mud.findMacro(key))
        if let Some(macro_def) = mud.find_macro(key) {
            // Echo if callback provided (C++ opt_echoinput check, lines 624-630)
            if let Some(echo) = echo_callback {
                // C++ format: SOFT_CR + ">> " + key_name + " -> " + macro_text + "\n"
                // Simplified: just show the expansion
                let msg = format!(">> Macro {} -> {}\n", key, macro_def.text);
                echo(&msg);
            }

            // Return macro text for interpreter to add (C++ line 632)
            // Note: C++ calls interpreter.add(m->text, EXPAND_ALL)
            // We return the text; caller handles queueing (Phase 2 feature)
            Some(macro_def.text.clone())
        } else {
            None
        }
    }
}

/// Get current Unix timestamp in seconds
fn current_time_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccp::PassthroughDecomp;

    #[test]
    fn session_manager_creation() {
        let mgr = SessionManager::new(PassthroughDecomp::new(), 80, 24, 200, "TestMUD".to_string());
        assert_eq!(mgr.state(), SessionState::Disconnected);
        assert!(mgr.socket.is_none());
    }

    #[test]
    fn session_manager_timeout() {
        let mut mgr =
            SessionManager::new(PassthroughDecomp::new(), 80, 24, 200, "TestMUD".to_string());
        mgr.session.state = SessionState::Connecting;
        mgr.session.stats.dial_time = current_time_unix() - 35; // 35 seconds ago

        // Create a no-op interpreter for testing
        struct NoOpInterp;
        impl Interpreter for NoOpInterp {
            fn run(&mut self, _function: &str, _arg: &str, _out: &mut String) -> bool {
                false
            }
        }
        let mut interp = NoOpInterp;

        let msg = mgr.idle(&mut interp);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("timed out"));
        assert_eq!(mgr.state(), SessionState::Disconnected);
    }

    #[test]
    fn expand_macros_found() {
        let mgr = SessionManager::new(PassthroughDecomp::new(), 80, 24, 200, "TestMUD".to_string());

        let mut mud = Mud::new("TestMUD", "127.0.0.1", 4000);
        mud.macro_list
            .push(crate::macro_def::Macro::new(1, "north"));
        mud.macro_list
            .push(crate::macro_def::Macro::new(2, "south"));

        // Test macro expansion without echo
        let result = mgr.expand_macros(1, &mud, false, None);
        assert_eq!(result, Some("north".to_string()));

        let result = mgr.expand_macros(2, &mud, false, None);
        assert_eq!(result, Some("south".to_string()));

        // Test non-existent macro
        let result = mgr.expand_macros(99, &mud, false, None);
        assert_eq!(result, None);
    }

    #[test]
    fn expand_macros_disabled() {
        let mgr = SessionManager::new(PassthroughDecomp::new(), 80, 24, 200, "TestMUD".to_string());

        let mut mud = Mud::new("TestMUD", "127.0.0.1", 4000);
        mud.macro_list
            .push(crate::macro_def::Macro::new(1, "north"));

        // Macros disabled - should return None even though macro exists
        let result = mgr.expand_macros(1, &mud, true, None);
        assert_eq!(result, None);
    }

    #[test]
    fn expand_macros_with_echo() {
        let mgr = SessionManager::new(PassthroughDecomp::new(), 80, 24, 200, "TestMUD".to_string());

        let mut mud = Mud::new("TestMUD", "127.0.0.1", 4000);
        mud.macro_list
            .push(crate::macro_def::Macro::new(1, "north"));

        // Test with echo callback
        let mut echoed = String::new();
        let result = mgr.expand_macros(
            1,
            &mud,
            false,
            Some(&mut |msg: &str| {
                echoed = msg.to_string();
            }),
        );

        assert_eq!(result, Some("north".to_string()));
        assert!(echoed.contains(">> Macro 1 -> north"));
    }
}
