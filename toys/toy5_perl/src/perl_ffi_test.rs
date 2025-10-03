// Test raw Perl C API FFI from Rust
// Matches C++ PerlEmbeddedInterpreter patterns

use std::ffi::{CStr, CString};
use std::ptr;

// =============================================================================
// Opaque types (Perl internal structures)
// =============================================================================

#[repr(C)]
pub struct PerlInterpreter {
    _private: [u8; 0],
}

#[repr(C)]
pub struct SV {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CV {
    _private: [u8; 0],
}

// =============================================================================
// Perl C API FFI declarations
// =============================================================================

extern "C" {
    // System initialization (required for threaded Perl on macOS)
    #[link_name = "Perl_sys_init3"]
    fn perl_sys_init3(
        argc: *mut libc::c_int,
        argv: *mut *mut *mut libc::c_char,
        env: *mut *mut *mut libc::c_char,
    );

    #[link_name = "Perl_sys_term"]
    fn perl_sys_term();

    // Interpreter lifecycle (no Perl_ prefix)
    fn perl_alloc() -> *mut PerlInterpreter;
    fn perl_construct(interp: *mut PerlInterpreter);
    fn perl_parse(
        interp: *mut PerlInterpreter,
        xsinit: Option<unsafe extern "C" fn(*mut PerlInterpreter)>,
        argc: libc::c_int,
        argv: *mut *mut libc::c_char,
        env: *mut *mut libc::c_char,
    ) -> libc::c_int;
    fn perl_run(interp: *mut PerlInterpreter) -> libc::c_int;
    fn perl_destruct(interp: *mut PerlInterpreter) -> libc::c_int;
    fn perl_free(interp: *mut PerlInterpreter);

    // Variable access (Perl_ prefix, takes interpreter as first param)
    #[link_name = "Perl_get_sv"]
    fn perl_get_sv(
        interp: *mut PerlInterpreter,
        name: *const libc::c_char,
        flags: libc::c_int,
    ) -> *mut SV;

    // Code evaluation (Perl_ prefix)
    #[link_name = "Perl_eval_pv"]
    fn perl_eval_pv(
        interp: *mut PerlInterpreter,
        code: *const libc::c_char,
        croak_on_error: libc::c_int,
    ) -> *mut SV;

    // String operations on SVs (Perl_sv_ prefix, takes interpreter)
    #[link_name = "Perl_sv_setpv"]
    fn sv_setpv(interp: *mut PerlInterpreter, sv: *mut SV, ptr: *const libc::c_char);

    #[link_name = "Perl_sv_setiv"]
    fn sv_setiv(interp: *mut PerlInterpreter, sv: *mut SV, num: libc::c_long);

    // Get value from SV
    #[link_name = "Perl_sv_2iv"]
    fn sv_2iv(interp: *mut PerlInterpreter, sv: *mut SV) -> libc::c_long;

    #[link_name = "Perl_sv_2pv"]
    fn sv_2pv(interp: *mut PerlInterpreter, sv: *mut SV, len: *mut libc::size_t) -> *const libc::c_char;

    // XS functions for DynaLoader
    fn boot_DynaLoader(interp: *mut PerlInterpreter, cv: *mut CV);

    #[link_name = "Perl_newXS"]
    fn newXS(
        interp: *mut PerlInterpreter,
        name: *const libc::c_char,
        subaddr: unsafe extern "C" fn(*mut PerlInterpreter, *mut CV),
        filename: *const libc::c_char,
    ) -> *mut CV;

    // Access environment
    #[link_name = "environ"]
    static ENVIRON: *const *const libc::c_char;
}

// Constants
const GV_ADD: libc::c_int = 0x01; // Create variable if it doesn't exist

// =============================================================================
// XS initialization callback (matches C++ xs_init)
// =============================================================================

unsafe extern "C" fn xs_init(interp: *mut PerlInterpreter) {
    let dynaloader_name = b"DynaLoader::boot_DynaLoader\0".as_ptr() as *const libc::c_char;
    let filename = b"perl_ffi_test.rs\0".as_ptr() as *const libc::c_char;

    newXS(interp, dynaloader_name, boot_DynaLoader, filename);
}

// =============================================================================
// Helper to set up Perl interpreter
// =============================================================================

unsafe fn init_perl() -> *mut PerlInterpreter {
    println!("=== Initializing Perl Interpreter ===\n");

    // Step 0: System init (required for threaded Perl on macOS)
    println!("0. PERL_SYS_INIT3()...");
    let mut argc: libc::c_int = 0;
    let mut argv: *mut *mut libc::c_char = ptr::null_mut();
    let mut env: *mut *mut libc::c_char = ptr::null_mut();
    perl_sys_init3(&mut argc, &mut argv, &mut env);
    println!("   ✓ System initialized");

    // Step 1: Allocate interpreter
    println!("\n1. perl_alloc()...");
    let interp = perl_alloc();
    if interp.is_null() {
        panic!("perl_alloc() returned null");
    }
    println!("   ✓ Interpreter allocated at {:?}", interp);

    // Step 2: Construct interpreter
    println!("\n2. perl_construct()...");
    perl_construct(interp);
    println!("   ✓ Interpreter constructed");

    // Step 3: Parse (initialize with args)
    println!("\n3. perl_parse()...");

    // Canonical bootstrap args: ["perl", "-e", "0"]
    // This tells Perl to "do nothing but start clean"
    let arg0 = CString::new("perl").unwrap();
    let arg1 = CString::new("-e").unwrap();
    let arg2 = CString::new("0").unwrap();

    let mut argv = vec![
        arg0.as_ptr() as *mut libc::c_char,
        arg1.as_ptr() as *mut libc::c_char,
        arg2.as_ptr() as *mut libc::c_char,
        ptr::null_mut(),
    ];

    println!("   Args: [\"\", \"-e\", \"0\"] (canonical bootstrap)");
    println!("   Using xs_init callback for DynaLoader");

    let result = perl_parse(
        interp,
        None,                   // Try without xs_init first
        3,                      // ✓ 3 args: "", "-e", "0"
        argv.as_mut_ptr(),
        ptr::null_mut(),        // ✓ NULL environ (cleaner)
    );

    println!("   perl_parse returned: {}", result);

    if result != 0 {
        panic!("perl_parse() failed with code {}", result);
    }
    println!("   ✓ Perl parsed and initialized");

    // Step 4: Get default variable $_ (like C++ does)
    println!("\n4. Getting $_ (default variable)...");
    let underscore = CString::new("_").unwrap();
    let default_var = perl_get_sv(interp, underscore.as_ptr(), GV_ADD);
    if default_var.is_null() {
        panic!("Failed to get $_ variable");
    }
    println!("   ✓ Got $_: {:?}", default_var);

    interp
}

unsafe fn cleanup_perl(interp: *mut PerlInterpreter) {
    println!("\n=== Cleaning Up Perl Interpreter ===\n");

    println!("1. perl_destruct()...");
    perl_destruct(interp);
    println!("   ✓ Interpreter destructed");

    println!("\n2. perl_free()...");
    perl_free(interp);
    println!("   ✓ Interpreter freed");

    println!("\n3. PERL_SYS_TERM()...");
    perl_sys_term();
    println!("   ✓ System terminated");
}

// =============================================================================
// Tests
// =============================================================================

unsafe fn test_eval(interp: *mut PerlInterpreter) {
    println!("\n=== Test: Eval Simple Expression ===\n");

    let code = CString::new("2 + 2").unwrap();
    println!("Evaluating: \"2 + 2\"");

    let result = perl_eval_pv(interp, code.as_ptr(), 0);

    if result.is_null() {
        println!("✗ perl_eval_pv returned null");
    } else {
        println!("✓ perl_eval_pv succeeded (returned SV: {:?})", result);

        // Extract the integer result
        let value = sv_2iv(interp, result);
        println!("✓ Result value: {}", value);
    }
}

unsafe fn test_set_variable(interp: *mut PerlInterpreter) {
    println!("\n=== Test: Set Perl Variable ===\n");

    println!("Getting $test_var...");
    let var_name = CString::new("test_var").unwrap();
    let sv = perl_get_sv(interp, var_name.as_ptr(), GV_ADD);

    if sv.is_null() {
        println!("✗ perl_get_sv returned null");
        return;
    }
    println!("✓ Got SV: {:?}", sv);

    println!("\nSetting $test_var = \"Hello from Rust!\"");
    let value = CString::new("Hello from Rust!").unwrap();
    sv_setpv(interp, sv, value.as_ptr());
    println!("✓ Variable set");

    // Verify by evaluating it
    println!("\nVerifying with eval: print $test_var");
    let code = CString::new("print $test_var").unwrap();
    perl_eval_pv(interp, code.as_ptr(), 0);
    println!();
}

unsafe fn test_set_int_variable(interp: *mut PerlInterpreter) {
    println!("\n=== Test: Set Integer Variable ===\n");

    println!("Getting $num_var...");
    let var_name = CString::new("num_var").unwrap();
    let sv = perl_get_sv(interp, var_name.as_ptr(), GV_ADD);

    if sv.is_null() {
        println!("✗ perl_get_sv returned null");
        return;
    }
    println!("✓ Got SV: {:?}", sv);

    println!("\nSetting $num_var = 42");
    sv_setiv(interp, sv, 42);
    println!("✓ Variable set");

    // Verify
    println!("\nVerifying with eval: print $num_var");
    let code = CString::new("print $num_var").unwrap();
    perl_eval_pv(interp, code.as_ptr(), 0);
    println!();
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("=== Perl FFI Test (Raw C API) ===\n");

    unsafe {
        // Initialize
        let interp = init_perl();

        // Run tests
        test_eval(interp);
        test_set_variable(interp);
        test_set_int_variable(interp);

        // Cleanup
        cleanup_perl(interp);
    }

    println!("\n=== All Tests Complete ===");
}
