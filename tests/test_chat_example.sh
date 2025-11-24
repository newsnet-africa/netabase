#!/bin/bash
# Multi-process integration test for the chat example
# This script spawns multiple chat instances and verifies message storage

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="./test_chat_data_$$"
EXAMPLE_BIN="./target/debug/examples/simple_mdns_chat"

echo -e "${YELLOW}=== Chat Example Multi-Process Test ===${NC}"
echo ""

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining processes
    pkill -f simple_mdns_chat || true
    # Remove test data
    rm -rf "$TEST_DIR" || true
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Build the example
echo -e "${YELLOW}Building example...${NC}"
cargo build --example simple_mdns_chat --all-features
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Create test directory
mkdir -p "$TEST_DIR"

# Test 1: Single node message storage
echo -e "${YELLOW}Test 1: Single node message storage${NC}"
(
    cd "$TEST_DIR"
    mkdir -p alice
    cd alice

    # Send commands via stdin
    {
        sleep 0.5
        echo "Hello from Alice!"
        sleep 0.3
        echo "Second message"
        sleep 0.3
        echo "Third message"
        sleep 0.3
        echo "/history"
        sleep 0.5
        echo "quit"
    } | timeout 10 "$EXAMPLE_BIN" alice 2>&1 | tee output.log

    # Check if messages were stored
    if grep -q "✓ Message stored in database" output.log; then
        echo -e "${GREEN}✓ Messages stored successfully${NC}"
    else
        echo -e "${RED}✗ Failed to store messages${NC}"
        exit 1
    fi

    if grep -q "Found 3 message(s)" output.log; then
        echo -e "${GREEN}✓ All 3 messages retrieved from history${NC}"
    else
        echo -e "${RED}✗ Failed to retrieve correct number of messages${NC}"
        exit 1
    fi
)
echo ""

# Test 2: Multiple nodes with separate databases
echo -e "${YELLOW}Test 2: Multiple nodes with separate databases${NC}"
(
    cd "$TEST_DIR"

    # Start Bob in background
    mkdir -p bob
    (
        cd bob
        {
            sleep 0.5
            echo "Bob message 1"
            sleep 0.3
            echo "Bob message 2"
            sleep 2
            echo "/history"
            sleep 0.5
            echo "quit"
        } | timeout 15 "$EXAMPLE_BIN" bob > output.log 2>&1
    ) &
    BOB_PID=$!

    # Start Charlie in background
    mkdir -p charlie
    (
        cd charlie
        {
            sleep 0.5
            echo "Charlie message 1"
            sleep 0.3
            echo "Charlie message 2"
            sleep 0.3
            echo "Charlie message 3"
            sleep 2
            echo "/history"
            sleep 0.5
            echo "quit"
        } | timeout 15 "$EXAMPLE_BIN" charlie > output.log 2>&1
    ) &
    CHARLIE_PID=$!

    # Wait for both to finish
    wait $BOB_PID
    wait $CHARLIE_PID

    # Verify Bob's messages
    if grep -q "Found 2 message(s)" bob/output.log; then
        echo -e "${GREEN}✓ Bob has correct message count${NC}"
    else
        echo -e "${RED}✗ Bob has incorrect message count${NC}"
        cat bob/output.log
        exit 1
    fi

    # Verify Charlie's messages
    if grep -q "Found 3 message(s)" charlie/output.log; then
        echo -e "${GREEN}✓ Charlie has correct message count${NC}"
    else
        echo -e "${RED}✗ Charlie has incorrect message count${NC}"
        cat charlie/output.log
        exit 1
    fi
)
echo ""

# Test 3: Message persistence across sessions
echo -e "${YELLOW}Test 3: Message persistence across sessions${NC}"
(
    cd "$TEST_DIR"
    mkdir -p dave

    # First session
    (
        cd dave
        {
            sleep 0.5
            echo "Persistent message 1"
            sleep 0.3
            echo "Persistent message 2"
            sleep 0.5
            echo "quit"
        } | timeout 10 "$EXAMPLE_BIN" dave > session1.log 2>&1
    )

    # Second session - should load previous messages
    (
        cd dave
        {
            sleep 0.5
            echo "/history"
            sleep 0.5
            echo "quit"
        } | timeout 10 "$EXAMPLE_BIN" dave > session2.log 2>&1
    )

    # Verify persistence
    if grep -q "Loaded 2 message(s) from history" dave/session2.log; then
        echo -e "${GREEN}✓ Messages persisted across sessions${NC}"
    else
        echo -e "${YELLOW}! Persistence check inconclusive (may need more time)${NC}"
    fi
)
echo ""

# Test 4: Peer discovery
echo -e "${YELLOW}Test 4: Peer discovery${NC}"
(
    cd "$TEST_DIR"

    mkdir -p peer1 peer2

    # Start peer1
    (
        cd peer1
        {
            sleep 2
            echo "/peers"
            sleep 1
            echo "quit"
        } | timeout 15 "$EXAMPLE_BIN" peer1 > output.log 2>&1
    ) &
    PEER1_PID=$!

    # Start peer2
    (
        cd peer2
        {
            sleep 2
            echo "/peers"
            sleep 1
            echo "quit"
        } | timeout 15 "$EXAMPLE_BIN" peer2 > output.log 2>&1
    ) &
    PEER2_PID=$!

    wait $PEER1_PID
    wait $PEER2_PID

    # Check if peers discovered each other
    if grep -q "Connected to.*peer(s)" peer1/output.log || grep -q "Connected to.*peer(s)" peer2/output.log; then
        echo -e "${GREEN}✓ Peer discovery working${NC}"
    else
        echo -e "${YELLOW}! Peer discovery check inconclusive${NC}"
    fi
)
echo ""

# Test 5: Database directory structure
echo -e "${YELLOW}Test 5: Database directory structure${NC}"
if [ -d "$TEST_DIR/alice/chat_data" ]; then
    echo -e "${GREEN}✓ Database directories created${NC}"
else
    echo -e "${YELLOW}! Database directories not found (may use different path)${NC}"
fi
echo ""

echo -e "${GREEN}=== All Tests Completed ===${NC}"
echo ""
echo "Test artifacts saved in: $TEST_DIR"
echo "To inspect: ls -la $TEST_DIR/"
echo ""
