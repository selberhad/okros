//! Debug logging utility for TTY mode debugging
//! Writes to /tmp/okros_debug.log since stderr isn't visible in ncurses

use std::io::Write;

const DEBUG_LOG_PATH: &str = "/tmp/okros_debug.log";

/// Write a debug message to /tmp/okros_debug.log
/// Usage: debug_log!("some message: {}", value);
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {{
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/okros_debug.log")
        {
            let _ = writeln!(f, $($arg)*);
        }
    }};
}

/// Clear the debug log (call at startup)
pub fn clear_debug_log() {
    let _ = std::fs::remove_file(DEBUG_LOG_PATH);
}
