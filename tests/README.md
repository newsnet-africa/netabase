# Netabase Distributed Tests

This directory contains comprehensive tests for verifying Netabase's distributed functionality across processes and machines.

## Overview

The distributed tests verify that:
- Netabase instances can communicate across different processes
- Data can be stored and retrieved across the network
- Multiple schema types work correctly in distributed scenarios
- The libp2p networking layer functions properly

## Test Structure

### Schema Definitions
The tests use the following schemas defined with Netabase macros:

```rust
#[schema]
mod test_schemas {
    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq)]
    pub struct TestUser {
        #[key]
        pub id: String,
        pub name: String,
        pub age: u32,
        pub email: String,
    }

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq)]
    pub struct TestMessage {
        #[key]
        pub message_id: u64,
        pub sender: String,
        pub content: String,
        pub timestamp: u64,
    }

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq)]
    pub enum TestDocument {
        Text {
            #[key]
            id: String,
            content: String,
        },
        Binary {
            #[key]
            id: String,
            data: Vec<u8>,
        },
    }
}
```

### Available Tests

1. **`test_local_netabase_operations`** - Basic single-node functionality test
2. **`test_multiple_schema_types`** - Tests all schema types on a single node
3. **`distributed_two_nodes_local`** - Two processes on the same machine (manual coordination required)
4. **`distributed_two_nodes_remote`** - Two processes on different machines (manual coordination required)

## Running Tests

### Single Node Tests (Automated)

These tests run automatically without manual coordination:

```bash
# Test basic local operations
cargo test test_local_netabase_operations -- --nocapture

# Test multiple schema types
cargo test test_multiple_schema_types -- --nocapture
```

### Two Process Tests (Local Machine)

#### Step 1: Start Node A
```bash
NETABASE_NODE=A cargo test distributed_two_nodes_local -- --nocapture --ignored
```

This will output something like:
```
Node A started with PeerID: 12D3KooWXXXXXX at address: /ip4/127.0.0.1/tcp/54321/p2p/12D3KooWXXXXXX
Start Node B with: NETABASE_NODE=B NETABASE_BOOTSTRAP=/ip4/127.0.0.1/tcp/54321/p2p/12D3KooWXXXXXX cargo test distributed_two_nodes_local -- --nocapture --ignored
```

#### Step 2: Start Node B (in another terminal)
Copy the command from Node A's output:
```bash
NETABASE_NODE=B NETABASE_BOOTSTRAP=/ip4/127.0.0.1/tcp/54321/p2p/12D3KooWXXXXXX cargo test distributed_two_nodes_local -- --nocapture --ignored
```

### Two Machine Tests (Remote)

#### Step 1: Start Node A (Machine 1)
```bash
NETABASE_NODE=A NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 cargo test distributed_two_nodes_remote -- --nocapture --ignored
```

This will output:
```
Node A (Machine 1): Waiting for connections...
Start Node B on another machine with:
NETABASE_NODE=B NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 NETABASE_BOOTSTRAP=/ip4/<THIS_MACHINE_IP>/tcp/4001/p2p/12D3KooWXXXXXX cargo test distributed_two_nodes_remote -- --nocapture --ignored
```

#### Step 2: Start Node B (Machine 2)
Replace `<THIS_MACHINE_IP>` with Machine 1's actual IP address:
```bash
NETABASE_NODE=B NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 NETABASE_BOOTSTRAP=/ip4/192.168.1.100/tcp/4001/p2p/12D3KooWXXXXXX cargo test distributed_two_nodes_remote -- --nocapture --ignored
```

## Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `NETABASE_NODE` | Node identifier (A or B) | A | `NETABASE_NODE=B` |
| `NETABASE_IP` | IP address to bind to | 127.0.0.1 | `NETABASE_IP=0.0.0.0` |
| `NETABASE_PORT` | Port to bind to (0 = random) | 0 | `NETABASE_PORT=4001` |
| `NETABASE_BOOTSTRAP` | Bootstrap address for Node B | None | `NETABASE_BOOTSTRAP=/ip4/127.0.0.1/tcp/4001/p2p/12D3...` |

## Expected Test Flow

### Node A (First Node)
1. Creates and starts a Netabase instance
2. Displays connection information for Node B
3. Stores test data (TestUser, TestMessage, TestDocument)
4. Waits for replication
5. Verifies local retrieval
6. Keeps running for extended testing

### Node B (Second Node)
1. Connects to Node A using bootstrap address
2. Waits for network connectivity
3. Attempts to retrieve data stored by Node A
4. If successful, verifies the data matches expected values
5. If unsuccessful, stores its own test data to verify local functionality

## Success Indicators

Look for these messages indicating successful distributed operation:

```
Node A: Put result: PutRecordOk { ... }
Node A: Test completed successfully!

Node B: Successfully retrieved user: TestUser { ... }
Node B: Test completed successfully!
```

## Troubleshooting

### Connection Issues
- Ensure firewall allows the specified ports
- For remote tests, verify machines can reach each other
- Check that the bootstrap address is correct and reachable

### Port Conflicts
- Use `NETABASE_PORT=0` for automatic port assignment
- Or specify different ports for each test run

### Network Discovery
- Local tests use mDNS for discovery
- Remote tests require explicit bootstrap addresses
- Wait sufficient time for peer discovery (tests include built-in delays)

### Data Not Found
- This is expected if nodes haven't fully synchronized
- Tests include fallback scenarios where Node B stores its own data
- Increase wait times in the test if needed

## Test Architecture

The tests use:
- **libp2p Kademlia DHT** for distributed storage and retrieval
- **Bincode** for serialization with the NetabaseSchema macro system
- **Tokio** for async operation
- **Environment variables** for coordination between test instances

Each test creates temporary storage directories and generates unique peer IDs for isolation.

## Extending Tests

To add new test scenarios:

1. Define new schemas in the `test_schemas` module using `#[derive(NetabaseSchema)]`
2. Add key fields with `#[key]` attribute
3. Create test functions following the existing patterns
4. Use `#[ignore]` for tests requiring manual coordination

## Development Notes

- Tests use deterministic keys where possible for reproducibility
- Storage paths are temporary and automatically cleaned up
- Network timeouts are generous to account for slow connections
- All tests include detailed logging for debugging