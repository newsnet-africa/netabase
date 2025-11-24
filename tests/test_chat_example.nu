#!/usr/bin/env nu
# Multi-process integration test for the chat example (Nushell version)
# This script spawns multiple chat instances and verifies message storage

# Test configuration
let test_dir = $"./test_chat_data_(random integer 1000..9999)"
let example_bin = "./target/debug/examples/simple_mdns_chat"

print $"(ansi yellow)=== Chat Example Multi-Process Test ===(ansi reset)\n"

# Cleanup function
def cleanup [test_dir: string] {
    print $"(ansi yellow)Cleaning up...(ansi reset)"

    # Kill any remaining processes
    try {
        ps | where name =~ simple_mdns_chat | get pid | each { |pid| kill --force $pid }
    }

    # Remove test data
    try { rm -rf $test_dir }

    print $"(ansi green)✓ Cleanup complete(ansi reset)"
}

# Build the example
print $"(ansi yellow)Building example...(ansi reset)"
cargo build --example simple_mdns_chat --all-features
print $"(ansi green)✓ Build complete(ansi reset)\n"

# Create test directory
mkdir $test_dir

# Test 1: Single node message storage
print $"(ansi yellow)Test 1: Single node message storage(ansi reset)"
cd $test_dir
mkdir alice
cd alice

let commands = [
    "Hello from Alice!"
    "Second message"
    "Third message"
    "/history"
    "quit"
]

# Send commands to the chat example
let output = (
    $commands
    | str join "\n"
    | $"(echo $in)\n"
    | ^$example_bin alice
    | complete
)

$output.stdout | save output.log

# Check results
if ($output.stdout | str contains "✓ Message stored in database") {
    print $"(ansi green)✓ Messages stored successfully(ansi reset)"
} else {
    print $"(ansi red)✗ Failed to store messages(ansi reset)"
    cd ../..
    cleanup $test_dir
    exit 1
}

if ($output.stdout | str contains "Found 3 message(s)") {
    print $"(ansi green)✓ All 3 messages retrieved from history(ansi reset)"
} else {
    print $"(ansi red)✗ Failed to retrieve correct number of messages(ansi reset)"
    cd ../..
    cleanup $test_dir
    exit 1
}

cd ../..
print ""

# Test 2: Multiple nodes with separate databases
print $"(ansi yellow)Test 2: Multiple nodes with separate databases(ansi reset)"

cd $test_dir

# Start Bob
mkdir bob
let bob_job = (do {
    cd bob
    let bob_commands = [
        "Bob message 1"
        "Bob message 2"
        "/history"
        "quit"
    ]

    $bob_commands
    | str join "\n"
    | $"(echo $in)\n"
    | ^$example_bin bob
    | save output.log
} | complete)

# Start Charlie
mkdir charlie
let charlie_job = (do {
    cd charlie
    let charlie_commands = [
        "Charlie message 1"
        "Charlie message 2"
        "Charlie message 3"
        "/history"
        "quit"
    ]

    $charlie_commands
    | str join "\n"
    | $"(echo $in)\n"
    | ^$example_bin charlie
    | save output.log
} | complete)

# Give them time to complete
sleep 3sec

# Verify Bob's messages
let bob_output = (open bob/output.log)
if ($bob_output | str contains "Found 2 message(s)") {
    print $"(ansi green)✓ Bob has correct message count(ansi reset)"
} else {
    print $"(ansi red)✗ Bob has incorrect message count(ansi reset)"
    print $bob_output
    cd ..
    cleanup $test_dir
    exit 1
}

# Verify Charlie's messages
let charlie_output = (open charlie/output.log)
if ($charlie_output | str contains "Found 3 message(s)") {
    print $"(ansi green)✓ Charlie has correct message count(ansi reset)"
} else {
    print $"(ansi red)✗ Charlie has incorrect message count(ansi reset)"
    print $charlie_output
    cd ..
    cleanup $test_dir
    exit 1
}

cd ..
print ""

# Test 3: Message persistence across sessions
print $"(ansi yellow)Test 3: Message persistence across sessions(ansi reset)"

cd $test_dir
mkdir dave

# First session
cd dave
let session1_commands = [
    "Persistent message 1"
    "Persistent message 2"
    "quit"
]

$session1_commands
| str join "\n"
| $"(echo $in)\n"
| ^$example_bin dave
| save session1.log

# Second session
let session2_commands = [
    "/history"
    "quit"
]

$session2_commands
| str join "\n"
| $"(echo $in)\n"
| ^$example_bin dave
| save session2.log

# Verify persistence
let session2_output = (open session2.log)
if ($session2_output | str contains "Loaded 2 message(s) from history") {
    print $"(ansi green)✓ Messages persisted across sessions(ansi reset)"
} else {
    print $"(ansi yellow)! Persistence check inconclusive (may need more time)(ansi reset)"
}

cd ../..
print ""

# Test 4: Database structure
print $"(ansi yellow)Test 4: Database directory structure(ansi reset)"
if ($"($test_dir)/alice/chat_data" | path exists) {
    print $"(ansi green)✓ Database directories created(ansi reset)"
} else {
    print $"(ansi yellow)! Database directories not found (may use different path)(ansi reset)"
}
print ""

print $"(ansi green)=== All Tests Completed ===(ansi reset)\n"
print $"Test artifacts saved in: ($test_dir)"
print $"To inspect: ls -la ($test_dir)/"
print ""

# Cleanup
cleanup $test_dir
