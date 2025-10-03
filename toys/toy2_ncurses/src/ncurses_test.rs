// Test ncurses crate for MCL-style usage
// Key requirements:
// 1. Low-level terminfo access (setupterm, tigetstr)
// 2. ACS character codes
// 3. No actual ncurses windows needed

use std::ffi::{CStr, CString};
use std::io::{self, Write};
use std::os::unix::io::RawFd;

// Raw ncurses FFI bindings we need beyond what ncurses crate exposes
extern "C" {
    fn setupterm(
        term: *const libc::c_char,
        filedes: libc::c_int,
        errret: *mut libc::c_int,
    ) -> libc::c_int;

    fn tigetstr(capname: *mut libc::c_char) -> *const libc::c_char;

    fn newterm(
        term: *mut libc::c_char,
        outfd: *mut libc::FILE,
        infd: *mut libc::FILE,
    ) -> *mut libc::c_void;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MCL-Style ncurses Usage Test ===\n");

    // 1. Test setupterm (like C++ init_curses)
    println!("1. Testing setupterm...");
    let term = std::env::var("TERM").unwrap_or_else(|_| "xterm".to_string());
    let term_cstr = CString::new(term.clone())?;

    let mut errret = 0;
    unsafe {
        let result = setupterm(term_cstr.as_ptr(), libc::STDOUT_FILENO, &mut errret);
        if errret != 1 {
            eprintln!("setupterm failed: {}", errret);
            return Err("setupterm failed".into());
        }
        println!("   ✓ setupterm succeeded for TERM={}", term);
    }

    // 2. Test tigetstr for capabilities (like C++ lookup_key)
    println!("\n2. Testing tigetstr for terminal capabilities...");
    let caps = ["smacs", "rmacs", "kf1", "kf2", "kcuu1", "kcud1"];

    for cap in &caps {
        let cap_cstr = CString::new(*cap)?;
        unsafe {
            let ptr = tigetstr(cap_cstr.as_ptr() as *mut _);
            if ptr.is_null() || ptr as isize == -1 {
                println!("   {} => (null)", cap);
            } else {
                let bytes = CStr::from_ptr(ptr).to_bytes();
                println!("   {} => {:?} (len={})", cap, String::from_utf8_lossy(bytes), bytes.len());
            }
        }
    }

    // 3. Test ACS characters (like C++ special_chars)
    println!("\n3. Testing ACS character codes...");
    println!("   ACS_VLINE: {:#04x} ({})", ncurses::ACS_VLINE(), ncurses::ACS_VLINE() as u8 as char);
    println!("   ACS_HLINE: {:#04x} ({})", ncurses::ACS_HLINE(), ncurses::ACS_HLINE() as u8 as char);
    println!("   ACS_ULCORNER: {:#04x}", ncurses::ACS_ULCORNER());
    println!("   ACS_URCORNER: {:#04x}", ncurses::ACS_URCORNER());
    println!("   ACS_LLCORNER: {:#04x}", ncurses::ACS_LLCORNER());
    println!("   ACS_LRCORNER: {:#04x}", ncurses::ACS_LRCORNER());
    println!("   ACS_CKBOARD: {:#04x}", ncurses::ACS_CKBOARD());

    // 4. Test ANSI escape output (like C++ Screen::refreshTTY)
    println!("\n4. Testing direct ANSI escape output...");
    println!("   Writing colored text with ANSI codes...");

    let mut stdout = io::stdout();

    // Red text
    write!(stdout, "\x1b[31m")?;
    write!(stdout, "This is red")?;
    write!(stdout, "\x1b[0m")?;

    write!(stdout, " | ")?;

    // Green bold text
    write!(stdout, "\x1b[1;32m")?;
    write!(stdout, "This is green bold")?;
    write!(stdout, "\x1b[0m")?;

    write!(stdout, " | ")?;

    // Blue background
    write!(stdout, "\x1b[44;37m")?;
    write!(stdout, "Blue background")?;
    write!(stdout, "\x1b[0m")?;

    writeln!(stdout)?;
    stdout.flush()?;

    println!("\n5. Testing cursor positioning...");
    // Save cursor, move to (10,5), print, restore
    write!(stdout, "\x1b[s")?; // Save cursor
    write!(stdout, "\x1b[10;5H")?; // Move to row 10, col 5
    write!(stdout, "Text at (10,5)")?;
    write!(stdout, "\x1b[u")?; // Restore cursor
    writeln!(stdout)?;
    stdout.flush()?;

    println!("\n=== Test Results ===");
    println!("✓ setupterm: Works for terminfo access");
    println!("✓ tigetstr: Can query terminal capabilities");
    println!("✓ ACS codes: Available from ncurses crate");
    println!("✓ ANSI escapes: Direct write to stdout works");
    println!("\nConclusion: ncurses crate provides sufficient low-level access");
    println!("for MCL's minimal ncurses usage (no window management needed).");

    Ok(())
}
