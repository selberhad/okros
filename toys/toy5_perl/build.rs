// build.rs - Find Perl and set up linking

use std::process::Command;

fn main() {
    // Use homebrew perl if available, otherwise system perl
    let perl_cmd = if std::path::Path::new("/opt/homebrew/bin/perl").exists() {
        "/opt/homebrew/bin/perl"
    } else {
        "perl"
    };

    eprintln!("Using perl: {}", perl_cmd);

    // Get Perl's archlib CORE directory
    let output = Command::new(perl_cmd)
        .args(&["-MConfig", "-e", "print \"$Config{archlibexp}/CORE\""])
        .output()
        .expect("Failed to run perl -MConfig");

    if !output.status.success() {
        panic!("perl command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    }

    let perl_core = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 from perl")
        .trim()
        .to_string();

    if perl_core.is_empty() {
        panic!("perl command returned empty path");
    }

    eprintln!("Perl CORE directory: {}", perl_core);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=native={}", perl_core);
    println!("cargo:rustc-link-lib=dylib=perl");
}
