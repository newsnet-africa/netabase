# Netabase Communication Architecture

## Overview

The Netabase system follows a clean thread-based architecture where the main `Netabase` struct communicates with a background swarm thread through channels.

The old (generated) documentation is outdated, but at the very least, this should be clear from the code:

## Architecture Components

```
┌─────────────────┐    Commands     ┌─────────────────────┐
│                 │  ───────────→   │                     │
│  Netabase API   │                 │   Swarm Thread      │
│                 │  ←───────────   │                     │
└─────────────────┘    Events       └─────────────────────┘
        │                                       │
        │                                       │
        ▼                                       ▼
┌─────────────────┐                 ┌─────────────────────┐
│   User Code     │                 │   libp2p Swarm      │
│                 │                 │   + Kademlia DHT    │
└─────────────────┘                 └─────────────────────┘
```
