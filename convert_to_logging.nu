#!/usr/bin/env nu

# Script to convert println!/eprintln! to proper logging

def main [] {
    print "Converting println!/eprintln! to log macros..."

    # Find all Rust files in src with println! or eprintln!
    let files = (
        glob src/**/*.rs
        | each { |file|
            let content = (open $file | into string)
            if ($content | str contains "println!") or ($content | str contains "eprintln!") {
                $file
            } else {
                null
            }
        }
        | compact
    )

    print $"Found ($files | length) files to update"

    for file in $files {
        print $"Processing: ($file)"

        # Read file content
        let content = (open $file | into string)

        # Check if already has log import
        let has_log_import = ($content | str contains "use log::")

        # Build new content
        mut new_content = $content

        # Add log import if needed (after first use statement)
        if not $has_log_import {
            if ($content | str contains "use ") {
                # Find the position after first use block
                $new_content = ($new_content | str replace --regex "(?m)(^use [^;]+;)" "$1\nuse log::{debug, info, warn, error};")
            }
        }

        # Replace println! with info! or debug!
        # Use info! for important events (emojis), debug! for others
        $new_content = ($new_content | str replace --all --regex 'println!\("🎧' 'info!("🎧')
        $new_content = ($new_content | str replace --all --regex 'println!\("🔍' 'info!("🔍')
        $new_content = ($new_content | str replace --all --regex 'println!\("🤝' 'info!("🤝')
        $new_content = ($new_content | str replace --all --regex 'println!\("📥' 'info!("📥')
        $new_content = ($new_content | str replace --all --regex 'println!\("📤' 'info!("📤')
        $new_content = ($new_content | str replace --all --regex 'println!\("✅' 'info!("✅')
        $new_content = ($new_content | str replace --all --regex 'println!\("Bootstrapped' 'info!("Bootstrapped')
        $new_content = ($new_content | str replace --all --regex 'println!\("New external address' 'info!("New external address')

        # Replace remaining println! with debug!
        $new_content = ($new_content | str replace --all "println!" "debug!")

        # Replace eprintln! with warn! or error!
        # Use error! for critical failures, warn! for recoverable issues
        $new_content = ($new_content | str replace --all --regex 'eprintln!\("Failed to send' 'warn!("Failed to send')
        $new_content = ($new_content | str replace --all --regex 'eprintln!\("Failed to dial' 'warn!("Failed to dial')
        $new_content = ($new_content | str replace --all --regex 'eprintln!\("Failed to bootstrap' 'warn!("Failed to bootstrap')
        $new_content = ($new_content | str replace --all --regex 'eprintln!\("Connection error' 'warn!("Connection error')
        $new_content = ($new_content | str replace --all "eprintln!" "warn!")

        # Write updated content
        $new_content | save -f $file
        print $"  ✓ Updated ($file)"
    }

    print $"\n✨ Done! Updated ($files | length) files"
}
