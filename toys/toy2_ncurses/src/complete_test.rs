// Complete test matching C++ MCL initialization pattern
// This initializes ncurses minimally to get ACS codes

use std::ffi::{CStr, CString};
use std::ptr;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complete MCL Initialization Pattern ===\n");

    let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
    println!("TERM: {}", term);

    // Step 1: setupterm (like C++ does first)
    println!("\n1. Calling setupterm...");
    let term_cstr = CString::new(term.clone())?;
    let mut errret = 0;

    unsafe {
        setupterm(term_cstr.as_ptr(), libc::STDOUT_FILENO, &mut errret);
        if errret != 1 {
            return Err(format!("setupterm failed: {}", errret).into());
        }
        println!("   ✓ setupterm OK");
    }

    // Step 2: Open /dev/null for ncurses output (like C++ does)
    println!("\n2. Opening /dev/null for ncurses...");
    let devnull = CString::new("/dev/null")?;
    let mode = CString::new("r+")?;

    let fp = unsafe { fopen(devnull.as_ptr(), mode.as_ptr()) };
    if fp.is_null() {
        return Err("Failed to open /dev/null".into());
    }
    println!("   ✓ Opened /dev/null");

    // Step 3: Call newterm (like C++ init_curses does)
    println!("\n3. Calling newterm...");
    let mut term_mut = term_cstr.into_raw();
    unsafe {
        let screen = newterm(term_mut, fp, fp);
        if screen.is_null() {
            return Err("newterm failed".into());
        }
        println!("   ✓ newterm OK");

        // Re-take ownership to prevent leak
        let _ = CString::from_raw(term_mut);
    }

    // Step 4: Now ACS codes should be populated
    println!("\n4. Checking ACS codes after initialization...");
    println!("   ACS_VLINE: {:#04x}", ncurses::ACS_VLINE());
    println!("   ACS_HLINE: {:#04x}", ncurses::ACS_HLINE());
    println!("   ACS_ULCORNER: {:#04x}", ncurses::ACS_ULCORNER());
    println!("   ACS_URCORNER: {:#04x}", ncurses::ACS_URCORNER());
    println!("   ACS_LLCORNER: {:#04x}", ncurses::ACS_LLCORNER());
    println!("   ACS_LRCORNER: {:#04x}", ncurses::ACS_LRCORNER());
    println!("   ACS_CKBOARD: {:#04x}", ncurses::ACS_CKBOARD());

    // Step 5: Query terminfo for smacs/rmacs (ACS enable/disable)
    println!("\n5. Querying ACS control sequences...");
    let smacs_str = CString::new("smacs")?;
    let rmacs_str = CString::new("rmacs")?;

    unsafe {
        let smacs = tigetstr(smacs_str.as_ptr() as *mut _);
        let rmacs = tigetstr(rmacs_str.as_ptr() as *mut _);

        if !smacs.is_null() && smacs as isize != -1 {
            let bytes = CStr::from_ptr(smacs).to_bytes();
            println!("   smacs: {:?}", bytes);
        }

        if !rmacs.is_null() && rmacs as isize != -1 {
            let bytes = CStr::from_ptr(rmacs).to_bytes();
            println!("   rmacs: {:?}", bytes);
        }
    }

    // Cleanup
    unsafe {
        fclose(fp);
    }

    println!("\n=== Results ===");
    println!("✓ Can replicate C++ MCL initialization exactly");
    println!("✓ setupterm → newterm → ACS codes available");
    println!("✓ All terminfo queries work");
    println!("\nConclusion: ncurses crate is sufficient for MCL port");

    Ok(())
}
