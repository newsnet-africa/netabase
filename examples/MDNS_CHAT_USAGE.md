# Simple mDNS Chat - Usage Guide

A demonstration of Netabase's Byzantine fault-tolerant synchronization using a simple chat application.

## Features

This example showcases:
- **Peer-to-peer messaging** via mDNS discovery
- **Byzantine fault tolerance** with configurable sync algorithms
- **Sybil attack resistance** using Proof-of-Work
- **Reputation-based peer filtering**
- **Gossip protocol** for efficient state propagation
- **Paxos consensus** for critical operations (optional)

## Quick Start

### Default Configuration

```bash
# Start a chat instance with default settings
cargo run --example simple_mdns_chat

# Or with a custom username
cargo run --example simple_mdns_chat -- --username alice
```

### Using Presets

The example includes four preconfigured presets:

#### 1. Development (Fast, minimal security)
```bash
cargo run --example simple_mdns_chat -- --preset development
```
- Gossip interval: 5s
- PoW difficulty: 12 (~0.004s to solve)
- BRB disabled
- Paxos disabled

#### 2. Production (Balanced)
```bash
cargo run --example simple_mdns_chat -- --preset production
```
- Gossip interval: 10s
- PoW difficulty: 20 (~1s to solve)
- BRB enabled (tolerates 2 Byzantine peers)
- Paxos disabled

#### 3. High Security (Maximum protection)
```bash
cargo run --example simple_mdns_chat -- --preset high-security
```
- Gossip interval: 15s
- PoW difficulty: 24 (~16s to solve)
- BRB enabled (tolerates 3 Byzantine peers)
- Paxos enabled

#### 4. Low Latency (Optimized for speed)
```bash
cargo run --example simple_mdns_chat -- --preset low-latency
```
- Gossip interval: 1s
- PoW difficulty: 16 (~0.06s to solve)
- BRB enabled
- High gossip fanout (5 peers)

## Command-Line Options

### Basic Options

```
--username <NAME>        Username for the chat
--db-path <PATH>         Database storage path
```

### Sync Configuration

```
--sync-enabled <BOOL>         Enable/disable synchronization (default: true)
--auto-sync <BOOL>            Auto-sync on record updates (default: true)
--sync-interval <SECONDS>     Sync interval in seconds (default: 30)
```

### Gossip Protocol

```
--gossip-enabled <BOOL>       Enable gossip protocol (default: true)
--gossip-interval <SECONDS>   Gossip interval (default: 10)
--gossip-fanout <N>           Number of peers per gossip round (default: 3)
```

**Gossip Fanout Recommendations:**
- **2-3**: Good for small networks (< 10 peers)
- **3-5**: Balanced for medium networks (10-100 peers)
- **5-7**: Better redundancy for large networks (100+ peers)

### Byzantine Reliable Broadcast (BRB)

```
--brb-enabled <BOOL>          Enable BRB (default: true)
--brb-total-peers <N>         Total peers in quorum (default: 7)
--brb-max-faulty <F>          Max Byzantine faults to tolerate (default: 2)
```

**BRB Configuration:**
- Must satisfy: `total_peers >= 3 * max_faulty + 1`
- Example: With `max_faulty=2`, need at least 7 peers
- Higher `max_faulty` = more Byzantine tolerance but more overhead

### Proof-of-Work (PoW) Sybil Resistance

```
--pow-enabled <BOOL>                Enable PoW (default: true)
--pow-difficulty <BITS>             Leading zero bits (default: 16)
--pow-challenge-duration <SECONDS>  Time to solve challenge (default: 60)
--pow-verification-duration <SECONDS> Verification validity (default: 3600)
```

**PoW Difficulty Guide:**
```
Difficulty  |  Avg Time  |  Use Case
------------|------------|---------------------------
12          |  ~0.004s   |  Development/Testing
16          |  ~0.06s    |  Light protection
20          |  ~1s       |  Production (recommended)
24          |  ~16s      |  High security
28          |  ~4 min    |  Maximum security
```

### Reputation System

```
--reputation-enabled <BOOL>   Enable reputation tracking (default: true)
```

The reputation system automatically:
- Tracks successful/failed interactions with peers
- Applies time-based decay (reputation approaches 0.5 over time)
- Uses diminishing returns for repeated successes
- Filters low-reputation peers

### Paxos Consensus

```
--paxos-enabled <BOOL>        Enable Paxos (default: false)
--paxos-acceptors <N>         Number of acceptors (default: 5)
--paxos-max-failures <F>      Max failures to tolerate (default: 2)
```

**When to Enable Paxos:**
- Critical operations requiring strong consistency
- When ordering of operations matters
- Trade-off: Higher latency but guaranteed consensus

## Example Usage Scenarios

### Development Setup (2 Peers)

**Terminal 1:**
```bash
cargo run --example simple_mdns_chat -- \
  --username alice \
  --preset development
```

**Terminal 2:**
```bash
cargo run --example simple_mdns_chat -- \
  --username bob \
  --preset development
```

### Production Setup with Custom Settings

**High Security, Slow Network:**
```bash
cargo run --example simple_mdns_chat -- \
  --username alice \
  --gossip-interval 30 \
  --gossip-fanout 3 \
  --pow-difficulty 24 \
  --brb-enabled true \
  --brb-total-peers 10 \
  --brb-max-faulty 3
```

**Low Latency LAN:**
```bash
cargo run --example simple_mdns_chat -- \
  --username alice \
  --gossip-interval 1 \
  --gossip-fanout 5 \
  --pow-difficulty 16 \
  --sync-interval 5
```

### Disable All Sync (DHT Only)

```bash
cargo run --example simple_mdns_chat -- \
  --username alice \
  --sync-enabled false
```

## In-Chat Commands

Once running, you can use these commands:

```
/history    - View all messages in local store
/config     - Show current sync configuration
/help       - Show help message
quit        - Exit the chat
```

## Understanding the Output

### Configuration Display

On startup, you'll see your sync configuration:

```
=== Sync Configuration ===
Sync enabled: true
Auto-sync: true
Sync interval: 30s

Gossip Protocol:
  Enabled: true
  Interval: 10s
  Fanout: 3

Byzantine Reliable Broadcast:
  Enabled: true
  Total peers: 7
  Max faulty: 2 (tolerates up to 2 Byzantine peers)

Sybil Resistance:
  PoW enabled: true
  PoW difficulty: 16 (avg ~0.1s to solve)
  Challenge duration: 60s
  Verification duration: 3600s
  Reputation enabled: true

Paxos Consensus:
  Enabled: false
==========================
```

### Peer Discovery

```
✓ Discovered peer via mDNS: 12D3KooWAbC123...
✓ Identified peer: 12D3KooWAbC123...
✓ Connected to peers! Messages will be synced.
```

### Sync Events

When sync is active, you'll see:
```
🔄 Sync message with peer: 12D3KooWAbC123...
```

Error messages indicate sync issues:
```
⚠ Sync failed with 12D3KooWAbC123...: Timeout
```

## Performance Tuning

### For Small Networks (2-5 peers)

```bash
--gossip-fanout 2 \
--gossip-interval 5 \
--brb-total-peers 4 \
--brb-max-faulty 1
```

### For Medium Networks (5-20 peers)

```bash
--gossip-fanout 3 \
--gossip-interval 10 \
--brb-total-peers 7 \
--brb-max-faulty 2
```

### For Large Networks (20+ peers)

```bash
--gossip-fanout 5 \
--gossip-interval 15 \
--brb-total-peers 10 \
--brb-max-faulty 3
```

### For High-Bandwidth Environments

```bash
--gossip-interval 1 \
--sync-interval 5 \
--gossip-fanout 7
```

### For Low-Bandwidth Environments

```bash
--gossip-interval 30 \
--sync-interval 60 \
--gossip-fanout 2
```

## Security Considerations

### Against Sybil Attacks

1. **Enable PoW** with appropriate difficulty:
   - Development: 12-16
   - Production: 20
   - High security: 24+

2. **Enable Reputation** to track peer behavior

3. **Shorter verification duration** requires more frequent PoW (more expensive for attackers)

### Against Byzantine Peers

1. **Enable BRB** for critical message delivery
2. **Increase max_faulty** for higher tolerance
3. **Enable Paxos** for operations requiring consensus

### Against Eclipse Attacks

1. **Higher gossip fanout** increases peer diversity
2. **Reputation system** helps identify malicious peers
3. **Random peer selection** in gossip prevents targeted isolation

## Troubleshooting

### No Peers Discovered

- **Check firewall**: Ensure mDNS traffic (UDP 5353) is allowed
- **Check network**: Peers must be on same local network
- **Wait longer**: Give mDNS 30 seconds to discover peers

### Sync Not Working

1. Check sync is enabled: `--sync-enabled true`
2. Verify configuration: Use `/config` command
3. Check logs for sync errors

### High CPU Usage

- **Lower PoW difficulty**: Reduce `--pow-difficulty`
- **Increase gossip interval**: Raise `--gossip-interval`
- **Reduce fanout**: Lower `--gossip-fanout`

### High Network Usage

- **Increase gossip interval**: Space out sync rounds
- **Reduce fanout**: Sync with fewer peers per round
- **Disable BRB**: For non-critical applications

## Database Cleanup

Chat data is stored in `./chat_data/<username>/` by default.

To clean up:
```bash
rm -rf ./chat_data/
```

## Advanced: Testing Byzantine Behavior

To simulate Byzantine behavior for testing:

**Malicious peer (sends invalid signatures):**
```bash
# Note: This would require modifying the example code
# to intentionally send bad data
```

**Reputation testing:**
1. Start 3+ peers
2. Observe reputation scores with `/config`
3. Peers with failed syncs will have lower reputation

## See Also

- [SYNC_ARCHITECTURE.md](../SYNC_ARCHITECTURE.md) - Detailed sync architecture
- [SYNC_REQUEST_RESPONSE_INTEGRATION.md](../SYNC_REQUEST_RESPONSE_INTEGRATION.md) - Protocol details
- [SYNC_INTEGRATION_GUIDE.md](../SYNC_INTEGRATION_GUIDE.md) - Integration guide
