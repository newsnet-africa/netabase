//! Build verification tests
//!
//! These tests verify that all auxiliary binaries compile correctly:
//! - Examples compile
//! - Doctests compile
//! - Benchmarks compile (when they exist)
//!
//! These tests use `cargo` commands to verify compilation.

use std::process::Command;

/// Helper function to run cargo commands
fn run_cargo_command(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new("cargo")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
}

/// Test that all examples compile successfully
#[test]
fn test_examples_compile() {
    println!("Checking that examples compile...");

    let output = run_cargo_command(&["build", "--examples", "--all-features"])
        .expect("Failed to run cargo build --examples");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Examples failed to compile");
    }

    println!("✓ All examples compiled successfully");
}

/// Test that documentation compiles successfully
///
/// Note: We use `cargo doc` instead of `cargo test --doc --no-run` because
/// the --no-run flag is not supported for doctests.
#[test]
fn test_doctests_compile() {
    println!("Checking that documentation compiles...");

    // Use cargo doc to verify documentation compiles
    let output = run_cargo_command(&["doc", "--all-features", "--no-deps"])
        .expect("Failed to run cargo doc");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Documentation failed to compile");
    }

    println!("✓ Documentation compiled successfully");
}

/// Test that benchmarks compile successfully (if they exist)
#[test]
fn test_benchmarks_compile() {
    println!("Checking that benchmarks compile...");

    let output = run_cargo_command(&["bench", "--no-run", "--all-features"])
        .expect("Failed to run cargo bench --no-run");

    // Note: cargo bench will exit with error if there are no benchmarks,
    // so we just check that the command ran (not necessarily successfully)
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("no bench target") || stderr.contains("no benchmarks") {
        println!("⚠ No benchmarks found (this is acceptable)");
    } else if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", stderr);
        panic!("Benchmarks failed to compile");
    } else {
        println!("✓ All benchmarks compiled successfully");
    }
}

/// Test that the native feature builds correctly
#[test]
fn test_native_feature_builds() {
    println!("Checking that native feature builds...");

    let output = run_cargo_command(&["build", "--features", "native"])
        .expect("Failed to run cargo build --features native");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Native feature failed to build");
    }

    println!("✓ Native feature builds successfully");
}

/// Test that tests compile (without running them)
#[test]
fn test_all_tests_compile() {
    println!("Checking that all tests compile...");

    let output = run_cargo_command(&["test", "--no-run", "--all-features"])
        .expect("Failed to run cargo test --no-run");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Tests failed to compile");
    }

    println!("✓ All tests compiled successfully");
}

/// Test that the library builds with minimal features
#[test]
fn test_minimal_build() {
    println!("Checking minimal build (no default features)...");

    let output = run_cargo_command(&["build", "--no-default-features"])
        .expect("Failed to run cargo build --no-default-features");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Minimal build (no default features) failed");
    }

    println!("✓ Minimal build successful");
}

/// Test that individual features build correctly
#[test]
fn test_individual_features() {
    let features = vec!["libp2p", "sled", "redb"];

    for feature in features {
        println!("Checking feature: {}", feature);

        let output = run_cargo_command(&[
            "build",
            "--no-default-features",
            "--features",
            feature,
        ])
        .expect(&format!("Failed to run cargo build for feature {}", feature));

        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Feature '{}' failed to build", feature);
        }

        println!("✓ Feature '{}' builds successfully", feature);
    }
}

/// Test that clippy doesn't find critical issues
#[test]
#[ignore] // Run separately with: cargo test test_clippy_check -- --ignored
fn test_clippy_check() {
    println!("Running clippy checks...");

    let output = run_cargo_command(&[
        "clippy",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])
    .expect("Failed to run cargo clippy");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Clippy found issues");
    }

    println!("✓ Clippy check passed");
}

/// Test that cargo check passes for the project
#[test]
fn test_cargo_check() {
    println!("Running cargo check...");

    let output = run_cargo_command(&["check", "--all-features"])
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Cargo check failed");
    }

    println!("✓ Cargo check passed");
}
