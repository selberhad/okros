// Test pancurses crate for MCL-style usage
// Pancurses is higher-level, but can we access low-level terminfo?

use std::io::{self, Write};
use std::ffi::CString;

// We still need raw FFI for terminfo since pancurses doesn't expose it
extern "C" {
    fn setupterm(
        term: *const libc::c_char,
        filedes: libc::c_int,
        errret: *mut libc::c_int,
    ) -> libc::c_int;

    fn tigetstr(capname: *mut libc::c_char) -> *const libc::c_char;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MCL-Style pancurses Usage Test ===\n");

    // 1. Test terminfo access (pancurses doesn't wrap setupterm)
    println!("1. Testing low-level terminfo access...");
    let term = std::env::var("TERM").unwrap_or_else(|_| "xterm".to_string());
    let term_cstr = CString::new(term.clone())?;

    let mut errret = 0;
    unsafe {
        let result = setupterm(term_cstr.as_ptr(), libc::STDOUT_FILENO, &mut errret);
        if errret != 1 {
            eprintln!("setupterm failed: {}", errret);
            println!("   ✗ setupterm not accessible through pancurses");
        } else {
            println!("   ✓ setupterm works (via raw FFI)");
        }
    }

    // 2. Test tigetstr
    println!("\n2. Testing tigetstr...");
    let cap = CString::new("smacs")?;
    unsafe {
        let ptr = tigetstr(cap.as_ptr() as *mut _);
        if ptr.is_null() || ptr as isize == -1 {
            println!("   ✗ tigetstr not accessible");
        } else {
            println!("   ✓ tigetstr works (via raw FFI)");
        }
    }

    // 3. Test ACS characters
    println!("\n3. Testing ACS character access...");
    // pancurses doesn't expose ACS_* constants directly
    // We'd need to init pancurses first, but that takes over the terminal
    println!("   ⚠ pancurses requires full init to access ACS codes");
    println!("   ⚠ Can't access ACS codes without ncurses window management");

    // 4. ANSI escape output (works regardless of ncurses)
    println!("\n4. Testing direct ANSI escape output...");
    let mut stdout = io::stdout();
    write!(stdout, "\x1b[32m")?;
    write!(stdout, "Green text")?;
    write!(stdout, "\x1b[0m")?;
    writeln!(stdout)?;
    println!("   ✓ Direct ANSI output works");

    println!("\n=== Test Results ===");
    println!("✗ pancurses: Higher-level abstraction");
    println!("✗ Requires manual FFI for setupterm/tigetstr");
    println!("✗ ACS codes hidden behind window initialization");
    println!("✗ More complex for MCL's minimal ncurses needs");
    println!("\nConclusion: pancurses adds unnecessary abstraction");
    println!("for MCL's use case (no actual window management).");

    Ok(())
}
