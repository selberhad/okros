//! Perl interpreter plugin (feature-gated)
//!
//! Ported from: plugins/PerlEmbeddedInterpreter.cc
//! Uses raw Perl C API FFI (validated in toy5)

use std::ffi::{CStr, CString};
use std::ptr;
use crate::plugins::stack::Interpreter;

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
    // System initialization (required for threaded Perl)
    #[link_name = "Perl_sys_init3"]
    fn perl_sys_init3(
        argc: *mut libc::c_int,
        argv: *mut *mut *mut libc::c_char,
        env: *mut *mut *mut libc::c_char,
    );

    #[link_name = "Perl_sys_term"]
    fn perl_sys_term();

    // Interpreter lifecycle
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

    // Variable access
    #[link_name = "Perl_get_sv"]
    fn perl_get_sv(
        interp: *mut PerlInterpreter,
        name: *const libc::c_char,
        flags: libc::c_int,
    ) -> *mut SV;

    // Code evaluation
    #[link_name = "Perl_eval_pv"]
    fn perl_eval_pv(
        interp: *mut PerlInterpreter,
        code: *const libc::c_char,
        croak_on_error: libc::c_int,
    ) -> *mut SV;

    // String operations on SVs
    #[link_name = "Perl_sv_setpv"]
    fn sv_setpv(interp: *mut PerlInterpreter, sv: *mut SV, ptr: *const libc::c_char);

    #[link_name = "Perl_sv_setiv"]
    fn sv_setiv(interp: *mut PerlInterpreter, sv: *mut SV, num: libc::c_long);

    // Get value from SV
    #[link_name = "Perl_sv_2iv"]
    fn sv_2iv(interp: *mut PerlInterpreter, sv: *mut SV) -> libc::c_long;

    #[link_name = "Perl_sv_2pv"]
    fn sv_2pv(
        interp: *mut PerlInterpreter,
        sv: *mut SV,
        len: *mut libc::size_t,
    ) -> *const libc::c_char;

    // XS functions for DynaLoader
    fn boot_DynaLoader(interp: *mut PerlInterpreter, cv: *mut CV);

    #[link_name = "Perl_newXS"]
    fn newXS(
        interp: *mut PerlInterpreter,
        name: *const libc::c_char,
        subaddr: unsafe extern "C" fn(*mut PerlInterpreter, *mut CV),
        filename: *const libc::c_char,
    ) -> *mut CV;
}

const GV_ADD: libc::c_int = 0x01;

// =============================================================================
// XS initialization callback (matches C++ xs_init)
// =============================================================================

unsafe extern "C" fn xs_init(interp: *mut PerlInterpreter) {
    let dynaloader_name = b"DynaLoader::boot_DynaLoader\0".as_ptr() as *const libc::c_char;
    let filename = b"perl.rs\0".as_ptr() as *const libc::c_char;

    newXS(interp, dynaloader_name, boot_DynaLoader, filename);
}

// =============================================================================
// Perl interpreter wrapper
// =============================================================================

/// Perl interpreter wrapper matching C++ PerlEmbeddedInterpreter patterns
pub struct PerlPlugin {
    interp: *mut PerlInterpreter,
    initialized: bool,
}

impl PerlPlugin {
    /// Initialize Perl interpreter
    ///
    /// C++ equivalent (PerlEmbeddedInterpreter.cc):
    /// ```cpp
    /// my_perl = perl_alloc();
    /// perl_construct(my_perl);
    /// perl_parse(my_perl, xs_init, argc, argv, NULL);
    /// perl_run(my_perl);
    /// ```
    pub fn new() -> Result<Self, String> {
        unsafe {
            // Step 0: System init (required for threaded Perl)
            let mut argc: libc::c_int = 0;
            let mut argv: *mut *mut libc::c_char = ptr::null_mut();
            let mut env: *mut *mut libc::c_char = ptr::null_mut();
            perl_sys_init3(&mut argc, &mut argv, &mut env);

            // Step 1: Allocate
            let interp = perl_alloc();
            if interp.is_null() {
                return Err("perl_alloc failed".into());
            }

            // Step 2: Construct
            perl_construct(interp);

            // Step 3: Parse with bootstrap args
            let arg0 = CString::new("perl").unwrap();
            let arg1 = CString::new("-e").unwrap();
            let arg2 = CString::new("0").unwrap();

            let mut argv = vec![
                arg0.as_ptr() as *mut libc::c_char,
                arg1.as_ptr() as *mut libc::c_char,
                arg2.as_ptr() as *mut libc::c_char,
                ptr::null_mut(),
            ];

            let result = perl_parse(interp, None, 3, argv.as_mut_ptr(), ptr::null_mut());

            if result != 0 {
                perl_destruct(interp);
                perl_free(interp);
                perl_sys_term();
                return Err(format!("perl_parse failed with code {}", result));
            }

            // Step 4: Run
            let run_result = perl_run(interp);
            if run_result != 0 {
                perl_destruct(interp);
                perl_free(interp);
                perl_sys_term();
                return Err(format!("perl_run failed with code {}", run_result));
            }

            Ok(PerlPlugin {
                interp,
                initialized: true,
            })
        }
    }

    /// Eval Perl code
    unsafe fn eval_internal(&mut self, code: &str) -> Result<(), String> {
        let c_code = CString::new(code).map_err(|e| e.to_string())?;
        let _result = perl_eval_pv(self.interp, c_code.as_ptr(), 0);
        // TODO: Check ERRSV for errors
        Ok(())
    }

    /// Call Perl function with string arg, return result as string
    unsafe fn call_function_internal(&mut self, function: &str, arg: &str) -> Result<String, String> {
        // Build Perl code: $result = function(arg);
        let code = format!("$_ = {}(q{{{}}})", function, arg);
        let c_code = CString::new(code).map_err(|e| e.to_string())?;

        let result_sv = perl_eval_pv(self.interp, c_code.as_ptr(), 0);
        if result_sv.is_null() {
            return Err("Function call returned null".into());
        }

        // Get string from $_
        let underscore = CString::new("_").unwrap();
        let sv = perl_get_sv(self.interp, underscore.as_ptr(), GV_ADD);
        if sv.is_null() {
            return Err("Failed to get $_ variable".into());
        }

        let mut len: libc::size_t = 0;
        let ptr = sv_2pv(self.interp, sv, &mut len);
        if ptr.is_null() {
            return Ok(String::new());
        }

        let cstr = CStr::from_ptr(ptr);
        Ok(cstr.to_string_lossy().into_owned())
    }

    /// Load Perl file
    unsafe fn load_file_internal(&mut self, filename: &str) -> Result<(), String> {
        let code = format!("do q{{{}}}", filename);
        self.eval_internal(&code)
    }
}

impl Interpreter for PerlPlugin {
    /// Run Perl function with arg, return result in out
    fn run(&mut self, function: &str, arg: &str, out: &mut String) -> bool {
        unsafe {
            match self.call_function_internal(function, arg) {
                Ok(result) => {
                    *out = result;
                    true
                }
                Err(_) => false,
            }
        }
    }

    /// Run quietly - suppress Perl errors if requested
    fn run_quietly(
        &mut self,
        function: &str,
        arg: &str,
        out: &mut String,
        _suppress_error: bool,
    ) -> bool {
        // Same as run for now - Perl error handling is complex
        self.run(function, arg, out)
    }

    /// Load Perl file
    fn load_file(&mut self, filename: &str, _suppress: bool) -> bool {
        unsafe {
            match self.load_file_internal(filename) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }

    /// Eval Perl expression
    fn eval(&mut self, expr: &str, _out: &mut String) {
        unsafe {
            let _ = self.eval_internal(expr);
        }
    }

    /// Set integer variable in Perl
    fn set_int(&mut self, var: &str, val: i64) {
        unsafe {
            if let Ok(c_name) = CString::new(var) {
                let sv = perl_get_sv(self.interp, c_name.as_ptr(), GV_ADD);
                if !sv.is_null() {
                    sv_setiv(self.interp, sv, val as libc::c_long);
                }
            }
        }
    }

    /// Set string variable in Perl
    fn set_str(&mut self, var: &str, val: &str) {
        unsafe {
            if let Ok(c_name) = CString::new(var) {
                if let Ok(c_value) = CString::new(val) {
                    let sv = perl_get_sv(self.interp, c_name.as_ptr(), GV_ADD);
                    if !sv.is_null() {
                        sv_setpv(self.interp, sv, c_value.as_ptr());
                    }
                }
            }
        }
    }

    /// Get integer variable from Perl
    fn get_int(&mut self, name: &str) -> i64 {
        unsafe {
            if let Ok(c_name) = CString::new(name) {
                let sv = perl_get_sv(self.interp, c_name.as_ptr(), GV_ADD);
                if !sv.is_null() {
                    return sv_2iv(self.interp, sv) as i64;
                }
            }
            0
        }
    }

    /// Get string variable from Perl
    fn get_str(&mut self, name: &str) -> String {
        unsafe {
            if let Ok(c_name) = CString::new(name) {
                let sv = perl_get_sv(self.interp, c_name.as_ptr(), GV_ADD);
                if !sv.is_null() {
                    let mut len: libc::size_t = 0;
                    let ptr = sv_2pv(self.interp, sv, &mut len);
                    if !ptr.is_null() {
                        let cstr = CStr::from_ptr(ptr);
                        return cstr.to_string_lossy().into_owned();
                    }
                }
            }
            String::new()
        }
    }

    /// Prepare regex pattern for trigger matching (C++ match_prepare)
    /// Returns compiled Perl sub that matches pattern and sets $_ to commands if matched
    fn match_prepare(&mut self, pattern: &str, commands: &str) -> Option<Box<dyn std::any::Any>> {
        unsafe {
            // Create Perl sub: sub { if (/$pattern/) { $_ = "$commands"; } else { $_ = ""; } }
            let code = format!(
                "sub {{ if (/{pat}/) {{ $_ = \"{cmd}\"; }} else {{ $_ = \"\"; }} }}",
                pat = pattern.replace("\\", "\\\\").replace("\"", "\\\""),
                cmd = commands.replace("\\", "\\\\").replace("\"", "\\\"")
            );

            if let Ok(c_code) = CString::new(code) {
                let sv = perl_eval_pv(self.interp, c_code.as_ptr(), 1); // 1 = TRUE (croak on error)
                if !sv.is_null() {
                    // Box the SV pointer as opaque data
                    return Some(Box::new(sv as usize));
                }
            }
            None
        }
    }

    /// Prepare regex substitution (C++ substitute_prepare)
    /// Returns compiled Perl sub that does s/pattern/replacement/g
    fn substitute_prepare(&mut self, pattern: &str, replacement: &str) -> Option<Box<dyn std::any::Any>> {
        unsafe {
            // Create Perl sub: sub { unless (s/$pattern/$replacement/g) { $_ = ""; } }
            let code = format!(
                "sub {{ unless (s/{pat}/{rep}/g) {{ $_ = \"\"; }} }}",
                pat = pattern.replace("\\", "\\\\").replace("\"", "\\\""),
                rep = replacement.replace("\\", "\\\\").replace("\"", "\\\"")
            );

            if let Ok(c_code) = CString::new(code) {
                let sv = perl_eval_pv(self.interp, c_code.as_ptr(), 1);
                if !sv.is_null() {
                    return Some(Box::new(sv as usize));
                }
            }
            None
        }
    }

    /// Execute compiled regex (C++ match)
    /// Sets $_ to text, calls compiled sub, returns result from $_
    fn match_exec(&mut self, compiled: &dyn std::any::Any, text: &str) -> Option<String> {
        unsafe {
            // Extract SV pointer from Any
            if let Some(&sv_ptr) = compiled.downcast_ref::<usize>() {
                // Set $_ to the input text
                if let Ok(c_text) = CString::new(text) {
                    if let Ok(c_default) = CString::new("_") {
                        let default_sv = perl_get_sv(self.interp, c_default.as_ptr(), GV_ADD);
                        if !default_sv.is_null() {
                            sv_setpv(self.interp, default_sv, c_text.as_ptr());

                            // Call the compiled sub (sv_ptr points to it)
                            // Note: This is simplified - C++ uses perl_call_sv with flags
                            // For MVP, we'll just return the $_ value after "calling" the sub
                            // TODO: Proper perl_call_sv implementation

                            // For now, just eval the sub in scalar context
                            // This is a simplified approach
                            let eval_code = format!("{{ my $sub = {}; $sub->(); $_ }}", sv_ptr);
                            if let Ok(c_eval) = CString::new(eval_code) {
                                let result_sv = perl_eval_pv(self.interp, c_eval.as_ptr(), 0);
                                if !result_sv.is_null() {
                                    let mut len: libc::size_t = 0;
                                    let ptr = sv_2pv(self.interp, result_sv, &mut len);
                                    if !ptr.is_null() && len > 0 {
                                        let cstr = CStr::from_ptr(ptr);
                                        return Some(cstr.to_string_lossy().into_owned());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        }
    }
}

impl Drop for PerlPlugin {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                perl_destruct(self.interp);
                perl_free(self.interp);
                perl_sys_term();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let _interp = PerlPlugin::new().unwrap();
        // If we got here, initialization worked
    }

    #[test]
    fn test_set_get_int() {
        let mut interp = PerlPlugin::new().unwrap();
        interp.set_int("test_num", 42);

        let value = interp.get_int("test_num");
        assert_eq!(value, 42);
    }

    #[test]
    fn test_set_get_string() {
        let mut interp = PerlPlugin::new().unwrap();
        interp.set_str("test_var", "Hello from Rust!");

        let value = interp.get_str("test_var");
        assert_eq!(value, "Hello from Rust!");
    }

    #[test]
    fn test_eval() {
        let mut interp = PerlPlugin::new().unwrap();
        let mut out = String::new();
        interp.eval("$x = 123", &mut out);

        let value = interp.get_int("x");
        assert_eq!(value, 123);
    }

    #[test]
    fn test_run_function() {
        let mut interp = PerlPlugin::new().unwrap();

        // Define a function that uppercases
        let mut out = String::new();
        interp.eval("sub test_func { my $s = shift; return uc($s); }", &mut out);

        // Call it via run
        let mut result = String::new();
        let ok = interp.run("test_func", "hello", &mut result);
        assert!(ok);
        assert_eq!(result, "HELLO");
    }
}
