# Simple mDNS Chat Usage Guide

## Overview
A clean, minimal P2P chat application that uses mDNS for peer discovery and Kademlia DHT for message distribution.

## Running the Chat

Start multiple instances with different usernames:

```bash
# Terminal 1
cargo run --example simple_mdns_chat alice

# Terminal 2
cargo run --example simple_mdns_chat bob
```

## Available Commands

- `/history` - View all messages stored in your local database
- `/help` - Display the help message with available commands
- `quit` or `exit` - Leave the chat

## Example Output

```
=== Simple mDNS Chat Application ===
Initializing netabase...

Welcome, alice!
Database path: ./chat_data/alice

=== Chat History ===
(No messages yet)
====================

Starting P2P network...
Network started! Listening for peers...

🎧 Listening on /ip4/0.0.0.0/tcp/45821
🎧 Listening on /ip4/192.168.1.100/tcp/45821

Waiting for peer discovery via mDNS...
🔍 Discovered peer 12D3KooW via mDNS

🤝 Connected to peer 12D3KooW

✓ Connected to peers! You can now send messages.

Commands:
  /history - View all messages in local store
  /help    - Show this help message
  quit     - Exit the chat

alice: Hello everyone!
📤 alice: Hello everyone!

📥 Receiving message from peer 12D3KooW...
   Record received and stored

alice: /history

=== Message History ===
[14:32:15] alice: Hello everyone!
[14:32:18] bob: Hi Alice!
=======================

alice: quit

Goodbye!

Chat session ended.
```

## Features

### Clean Message Display
- Each message is followed by a blank line for better readability
- Sent messages show: `📤 username: message`
- Received messages show: `📥 Receiving message from peer [ID]...`

### Local Message Storage
- All messages are automatically persisted to your local database
- Use `/history` to view all messages in chronological order
- Messages persist across chat sessions

### Automatic Peer Discovery
- Uses mDNS for local network discovery
- No configuration needed - peers find each other automatically
- Visual feedback when peers connect

### Network Events
- 🎧 Shows listening addresses
- 🔍 mDNS peer discovery notifications
- 🤝 Peer connection confirmations
- 📤 Message sent confirmations
- 📥 Message received notifications

## Data Storage

Messages are stored in `./chat_data/<username>/` using the sled embedded database.
Each user has their own isolated database directory.

To clear chat history:
```bash
rm -rf ./chat_data
```

## Network Architecture

- **mDNS**: Automatic peer discovery on the local network
- **libp2p**: P2P networking stack
- **Kademlia DHT**: Distributed message storage and routing
- **Sled**: Local embedded database for message persistence
