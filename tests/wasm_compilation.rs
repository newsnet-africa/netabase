//! WASM compilation tests
//!
//! These tests verify that the crate compiles successfully for WASM targets.
//! Any issues that cannot be resolved should be documented in the README.

use std::process::Command;

/// Helper to run cargo commands
fn run_cargo_command(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new("cargo")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
}

/// Helper to check if wasm32 target is installed
fn is_wasm_target_installed() -> bool {
    let output = Command::new("rustup")
        .args(&["target", "list", "--installed"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("wasm32-unknown-unknown")
        }
        Err(_) => false,
    }
}

/// Test: WASM target compilation with wasm feature
#[test]
fn test_wasm_compilation() {
    println!("Checking WASM compilation...");

    // First check if wasm32 target is installed
    if !is_wasm_target_installed() {
        println!("⚠ wasm32-unknown-unknown target not installed");
        println!("Install with: rustup target add wasm32-unknown-unknown");
        println!("Skipping WASM compilation test");
        return;
    }

    println!("✓ wasm32-unknown-unknown target is installed");

    // Try to build for WASM with wasm feature
    let output = run_cargo_command(&[
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--no-default-features",
        "--features",
        "wasm",
    ])
    .expect("Failed to run cargo build for WASM");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        eprintln!("=== WASM Compilation Failed ===");
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
        eprintln!("================================");

        // Document specific known issues
        let mut known_issues = Vec::new();

        if stderr.contains("tempfile") || stderr.contains("TempDir") {
            known_issues.push("- Issue: tempfile crate not available in WASM (requires filesystem access)");
            known_issues.push("  Resolution: Use browser storage APIs or IndexedDB for temporary storage");
        }

        if stderr.contains("std::fs") {
            known_issues.push("- Issue: std::fs not available in WASM");
            known_issues.push("  Resolution: Use web_sys FileSystem API or feature-gate filesystem code");
        }

        if stderr.contains("libp2p") && stderr.contains("tcp") {
            known_issues.push("- Issue: TCP transport not available in WASM");
            known_issues.push("  Resolution: Use WebSocket, WebRTC, or WebTransport in WASM");
        }

        if stderr.contains("mdns") {
            known_issues.push("- Issue: mDNS not available in WASM (requires UDP multicast)");
            known_issues.push("  Resolution: Disable mDNS in WASM builds, use signaling servers");
        }

        if stderr.contains("sled") || stderr.contains("redb") {
            known_issues.push("- Issue: Native database engines not available in WASM");
            known_issues.push("  Resolution: Use IndexedDB or localStorage for WASM storage");
        }

        if !known_issues.is_empty() {
            eprintln!("\n=== Known WASM Issues ===");
            for issue in &known_issues {
                eprintln!("{}", issue);
            }
            eprintln!("=========================\n");
        }

        panic!(
            "WASM compilation failed. See above for known issues.\n\
             These issues should be documented in README.md under 'WASM Limitations'"
        );
    }

    println!("✓ WASM compilation successful");
}

/// Test: Check for WASM-incompatible dependencies
#[test]
fn test_check_wasm_incompatible_deps() {
    println!("Checking for WASM-incompatible dependencies...");

    // Read Cargo.toml to check dependencies
    let cargo_toml_path = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = std::fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml");

    let mut warnings = Vec::new();

    // Check for potentially problematic dependencies
    if cargo_toml.contains("tempfile") && !cargo_toml.contains("target.'cfg(not(target_arch = \"wasm32\"))'.dependencies") {
        warnings.push("- tempfile should be conditional on cfg(not(target_arch = \"wasm32\"))");
    }

    // Check that WASM feature properly configured
    if !cargo_toml.contains("[features]") {
        warnings.push("- No features defined");
    } else {
        if !cargo_toml.contains("wasm") {
            warnings.push("- No 'wasm' feature defined");
        }
        if !cargo_toml.contains("native") {
            warnings.push("- No 'native' feature defined");
        }
    }

    if !warnings.is_empty() {
        println!("⚠ Potential WASM compatibility issues:");
        for warning in &warnings {
            println!("{}", warning);
        }
    } else {
        println!("✓ No obvious WASM incompatibility issues found");
    }
}

/// Test: Check that WASM feature gates exist
#[test]
fn test_wasm_feature_gates_exist() {
    println!("Checking for WASM feature gates in source code...");

    // Check that wasm feature gates are used in code
    let src_files = vec![
        format!("{}/src/lib.rs", env!("CARGO_MANIFEST_DIR")),
    ];

    let mut has_wasm_cfg = false;

    for file_path in &src_files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            if content.contains("cfg(target_arch = \"wasm32\")") ||
               content.contains("cfg(not(target_arch = \"wasm32\"))") ||
               content.contains("#[cfg(feature = \"wasm\")]") ||
               content.contains("#[cfg(feature = \"native\")]") {
                has_wasm_cfg = true;
                break;
            }
        }
    }

    if has_wasm_cfg {
        println!("✓ WASM feature gates found in source code");
    } else {
        println!("⚠ No WASM feature gates found - may need conditional compilation");
    }
}

/// Test: Verify wasm-bindgen is available when needed
#[test]
fn test_wasm_bindgen_availability() {
    println!("Checking wasm-bindgen availability...");

    let cargo_toml_path = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = std::fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml");

    let has_wasm_bindgen = cargo_toml.contains("wasm-bindgen");
    let has_wasm_feature = cargo_toml.contains("features") && cargo_toml.contains("wasm");

    if has_wasm_feature && !has_wasm_bindgen {
        println!("⚠ WASM feature exists but wasm-bindgen not found");
        println!("  This may be intentional if using a different WASM runtime");
    } else if has_wasm_bindgen {
        println!("✓ wasm-bindgen is configured");
    } else {
        println!("⚠ No wasm-bindgen or WASM feature configured");
    }
}
