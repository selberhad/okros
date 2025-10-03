# Toy 5: Perl Interpreter Embedding (Raw FFI)

## Status: ✅ SUCCESS

**Perl FFI working perfectly with modern threaded Perl!**

## What Worked

1. ✅ Linking works (build.rs finds libperl)
2. ✅ Function declarations compile and link
3. ✅ perl_alloc() and perl_construct() work
4. ✅ **perl_parse() works with PERL_SYS_INIT3!**
5. ✅ perl_eval_pv executes Perl code
6. ✅ Variable get/set (strings and integers) working

## The Problem (SOLVED!)

**Root cause**: MCL was written for Perl 5.10 or earlier (2007-2009 era). Modern Perl (5.34+) is compiled with threading support (`useithreads='define'`), which requires additional initialization.

**Error code 9**: Indicates syntax/parsing errors when `PERL_SYS_INIT3` is missing

## The Solution

Modern Perl requires `PERL_SYS_INIT3()` and `PERL_SYS_TERM()` for threaded builds:

```rust
// Before perl_alloc()
perl_sys_init3(&mut argc, &mut argv, &mut env);

// ... normal perl_alloc/construct/parse/run ...

// After perl_free()
perl_sys_term();
```

**Working initialization sequence**:
1. `PERL_SYS_INIT3()` - System init for threaded Perl
2. `perl_alloc()` - Allocate interpreter
3. `perl_construct()` - Construct interpreter
4. `perl_parse(interp, None, 3, ["", "-e", "0"], NULL)` - Parse with bootstrap args
5. `perl_eval_pv()` - Execute Perl code
6. `perl_destruct()` - Destruct interpreter
7. `perl_free()` - Free interpreter
8. `PERL_SYS_TERM()` - System cleanup

## Test Results

```
perl_parse returned: 0 ✅
Evaluating: "2 + 2"
✓ Result value: 4

Setting $test_var = "Hello from Rust!"
Verifying: Hello from Rust! ✅

Setting $num_var = 42
Verifying: 42 ✅
```

## Files

- `build.rs` - Build system (finds libperl via perl -MConfig)
- `src/perl_ffi_test.rs` - Rust FFI test (WORKS!)
- `test2.c` - Pure C test with PERL_SYS_INIT3 (reference)
- `LEARNINGS.md` - Complete FFI pattern documentation

## Recommendation

**✅ Perl port is VIABLE** - Use raw FFI with PERL_SYS_INIT3 for modern Perl!
