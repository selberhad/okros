//! Minimal ncurses wrapper for terminal capabilities
//!
//! Ported from: Curses.cc
//! MCL uses ncurses minimally - only for terminal setup and capability queries

use std::ffi::{CStr, CString};
use std::ptr;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn setupterm(
        term: *const libc::c_char,
        filedes: libc::c_int,
        errret: *mut libc::c_int,
    ) -> libc::c_int;

    fn newterm(
        term: *mut libc::c_char,
        outfd: *mut libc::FILE,
        infd: *mut libc::FILE,
    ) -> *mut libc::c_void;

    fn fopen(filename: *const libc::c_char, mode: *const libc::c_char) -> *mut libc::FILE;
    fn fclose(stream: *mut libc::FILE) -> libc::c_int;
    fn tigetstr(capname: *mut libc::c_char) -> *const libc::c_char;
}

// =============================================================================
// ACS capabilities storage
// =============================================================================

#[derive(Default, Debug, Clone)]
pub struct AcsCaps {
    pub smacs: Option<String>,
    pub rmacs: Option<String>,
}

static mut ACS_INITIALIZED: bool = false;
static mut ACS_CAPABILITIES: Option<AcsCaps> = None;

// =============================================================================
// Initialization
// =============================================================================

/// Initialize ncurses and query terminal capabilities
///
/// C++ equivalent (Curses.cc init_curses):
/// ```cpp
/// setupterm(term, STDOUT_FILENO, &err);
/// FILE *fp = fopen("/dev/null", "r+");
/// newterm(term, fp, fp);
/// ```
pub unsafe fn init_curses() -> Result<(), String> {
    if ACS_INITIALIZED {
        return Ok(());
    }

    let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
    let term_cstr = CString::new(term).map_err(|e| e.to_string())?;

    // Step 1: setupterm
    let mut errret = 0;
    setupterm(term_cstr.as_ptr(), libc::STDOUT_FILENO, &mut errret);
    if errret != 1 {
        return Err(format!("setupterm failed: {}", errret));
    }

    // Step 2: Open /dev/null for ncurses
    let devnull = CString::new("/dev/null").unwrap();
    let mode = CString::new("r+").unwrap();
    let fp = fopen(devnull.as_ptr(), mode.as_ptr());
    if fp.is_null() {
        return Err("Failed to open /dev/null".into());
    }

    // Step 3: newterm (initializes ncurses, populates ACS codes)
    let term_mut = term_cstr.into_raw();
    let screen = newterm(term_mut, fp, fp);
    if screen.is_null() {
        fclose(fp);
        let _ = CString::from_raw(term_mut);
        return Err("newterm failed".into());
    }

    // Re-take ownership to prevent leak
    let _ = CString::from_raw(term_mut);

    // Step 4: Query ACS capabilities
    let caps = AcsCaps {
        smacs: get_capability("smacs"),
        rmacs: get_capability("rmacs"),
    };

    ACS_CAPABILITIES = Some(caps);
    ACS_INITIALIZED = true;

    Ok(())
}

/// Get terminal capability string
///
/// C++ equivalent (Curses.cc):
/// ```cpp
/// char *str = tigetstr((char *)capname);
/// ```
unsafe fn get_capability(name: &str) -> Option<String> {
    let cap = CString::new(name).ok()?;
    let ptr = tigetstr(cap.as_ptr() as *mut _);

    if ptr.is_null() || ptr as isize == -1 {
        return None;
    }

    // Convert to String (lossy for non-UTF8 sequences)
    let bytes = CStr::from_ptr(ptr).to_bytes();
    String::from_utf8_lossy(bytes).into_owned().into()
}

/// Get ACS capabilities (smacs/rmacs)
pub fn get_acs_caps() -> AcsCaps {
    unsafe {
        if !ACS_INITIALIZED {
            // Initialize on first access
            let _ = init_curses();
        }

        ACS_CAPABILITIES.clone().unwrap_or_default()
    }
}

/// Get ACS character codes from ncurses
///
/// C++ equivalent (Curses.cc):
/// ```cpp
/// SPECIAL_CHARS[bc_vertical] = ACS_VLINE;
/// SPECIAL_CHARS[bc_horizontal] = ACS_HLINE;
/// // ...
/// ```
pub unsafe fn get_acs_codes() -> [u8; 8] {
    // Ensure ncurses is initialized first
    if !ACS_INITIALIZED {
        let _ = init_curses();
    }

    // Extract low byte (actual character code) from ncurses ACS values
    [
        (ncurses::ACS_VLINE() & 0xFF) as u8,      // vertical line
        (ncurses::ACS_HLINE() & 0xFF) as u8,      // horizontal line
        (ncurses::ACS_ULCORNER() & 0xFF) as u8,   // upper left corner
        (ncurses::ACS_URCORNER() & 0xFF) as u8,   // upper right corner
        (ncurses::ACS_LLCORNER() & 0xFF) as u8,   // lower left corner
        (ncurses::ACS_LRCORNER() & 0xFF) as u8,   // lower right corner
        (ncurses::ACS_CKBOARD() & 0xFF) as u8,    // checkerboard
        (ncurses::ACS_BULLET() & 0xFF) as u8,     // bullet
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_curses() {
        unsafe {
            let result = init_curses();
            assert!(result.is_ok(), "init_curses should succeed");
        }
    }

    #[test]
    fn test_get_acs_caps() {
        let caps = get_acs_caps();
        // Should have smacs/rmacs capabilities (or at least not crash)
        println!("smacs: {:?}", caps.smacs);
        println!("rmacs: {:?}", caps.rmacs);
    }

    #[test]
    fn test_get_acs_codes() {
        unsafe {
            let codes = get_acs_codes();
            // ACS codes should be non-zero
            println!("ACS_VLINE: {:#04x}", codes[0]);
            println!("ACS_HLINE: {:#04x}", codes[1]);
            assert!(codes[0] != 0, "VLINE should be non-zero");
            assert!(codes[1] != 0, "HLINE should be non-zero");
        }
    }
}
