# Meadowcap Capability System Specification

## Overview

This document specifies the Meadowcap capability system as implemented in netabase, adapted from the [Willow Protocol](https://willowprotocol.org/specs/meadowcap/index.html). Meadowcap provides:

- **Unforgeable access tokens**: Capabilities that cannot be created without proper authorization
- **Delegable access**: Capability holders can delegate (restricted) access to others  
- **Confidential discovery**: Private Area Intersection (PAI) reveals only mutual interests

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Capability Types](#capability-types)
3. [Area System](#area-system)
4. [Delegation](#delegation)
5. [Private Area Intersection (PAI)](#private-area-intersection-pai)
6. [Handshake Protocol](#handshake-protocol)
7. [Security Model](#security-model)
8. [Implementation Details](#implementation-details)
9. [Protocol Flow](#protocol-flow)

---

## Core Concepts

### What is a Capability?

A capability is an unforgeable token that grants access rights to data. Each capability answers four questions:

1. **To whom does it grant access?** → The *receiver* (a `UserPublicKey`/`PeerId`)
2. **Does it grant read or write access?** → The *access mode*
3. **For which data does it grant access?** → The *granted area*
4. **Is it valid or a forgery?** → Verified through *signature chain*

### Namespaces

Namespaces are identified by `NamespacePublicKey` and can be either:

- **Communal**: Each subspace is owned by its author. Anyone can create entries in their own subspace without prior authorization.
- **Owned**: A single owner (the namespace creator) controls all data and must explicitly delegate access.

The `is_communal()` function determines which type a namespace is.

### Subspaces

Within a namespace, data is organized into subspaces. In netabase:
- `SubspaceId` = `UserPublicKey` = `PeerId`
- Each user's subspace is identified by their peer ID

---

## Capability Types

### CommunalCapability

For communal namespaces where each subspace is independently owned.

```rust
pub struct CommunalCapability {
    pub access_mode: AccessMode,        // Read or Write
    pub namespace_key: NamespacePublicKey,
    pub user_key: UserPublicKey,        // Subspace owner
    pub delegations: Vec<CapabilityDelegation>,
}
```

**Properties:**
- Root capabilities (no delegations) are always valid
- The `user_key` is both the subspace owner AND the initial receiver
- Granted area starts as the subspace of `user_key`

**Example:**
```rust
// Alice creates a root capability for her subspace
let alice_cap = CommunalCapability::new_root(
    AccessMode::Write,
    namespace_key,
    alice_peer_id,
);
// alice_cap.receiver() == alice_peer_id
// alice_cap.granted_area() == Area::subspace(alice_peer_id)
```

### OwnedCapability

For owned namespaces where the namespace creator controls everything.

```rust
pub struct OwnedCapability {
    pub access_mode: AccessMode,
    pub namespace_key: NamespacePublicKey,
    pub user_key: UserPublicKey,
    pub initial_authorisation: NamespaceSignature,  // From namespace owner
    pub delegations: Vec<CapabilityDelegation>,
}
```

**Properties:**
- Requires `initial_authorisation` signed by namespace secret key
- Initial granted area is the full namespace
- Can delegate to specific subspaces

**Initial Authorization Message:**
```
0x02 || user_key  (for Read)
0x03 || user_key  (for Write)
```

### McCapability

Unified wrapper for both capability types:

```rust
pub enum McCapability {
    Communal(CommunalCapability),
    Owned(OwnedCapability),
}
```

**Validity requires:**
1. The inner capability is valid
2. The capability type matches the namespace type (communal for communal, owned for owned)

### McEnumerationCapability

Special capability for resolving "awkward" PAI cases:

```rust
pub struct McEnumerationCapability {
    pub namespace_key: NamespacePublicKey,
    pub user_key: UserPublicKey,
    pub initial_authorisation: NamespaceSignature,
    pub delegations: Vec<(UserPublicKey, UserSignature)>,
}
```

**Initial Authorization Message:**
```
0x04 || user_key
```

---

## Area System

An `Area` defines a region of entries within a namespace, constraining:

### SubspaceConstraint

```rust
pub enum SubspaceConstraint {
    Any,                    // All subspaces
    Specific(SubspaceId),   // Single subspace
}
```

### PathConstraint

Path prefix that entries must match:

```rust
pub struct PathConstraint {
    pub components: Vec<Vec<u8>>,  // e.g., ["data", "public"]
}
```

- Empty path matches everything
- `["a"]` is a prefix of `["a", "b"]`
- `["a", "b"]` is NOT a prefix of `["a"]`

### TimeRange

Optional temporal bounds:

```rust
pub struct TimeRange {
    pub start: Option<u64>,  // Inclusive
    pub end: Option<u64>,    // Exclusive
}
```

### Area Inclusion

Area A includes Area B if:
- A's subspace constraint includes B's subspace constraint
- A's path is a prefix of B's path
- A's time range includes B's time range

```rust
impl Area {
    pub fn includes(&self, other: &Area) -> bool {
        self.subspace.includes_constraint(&other.subspace)
            && self.path.includes(&other.path)
            && self.times.includes(&other.times)
    }
}
```

---

## Delegation

### CapabilityDelegation

```rust
pub struct CapabilityDelegation {
    pub area: Area,              // Restricted area
    pub user: UserPublicKey,     // New receiver
    pub signature: UserSignature, // From previous receiver
}
```

### Delegation Rules

1. **Restriction Only**: New area must be included in previous area
2. **Signature Required**: Previous receiver must sign the handover
3. **Chain Preserved**: Each delegation includes the previous signature

### Handover Message (Communal)

**First delegation:**
```
access_mode_byte || namespace_key || relative_area || new_user
```

**Subsequent delegations:**
```
prev_signature || relative_area || new_user
```

### Handover Message (Owned)

**First delegation:**
```
relative_area || initial_authorisation || new_user
```

**Subsequent delegations:**
```
relative_area || prev_signature || new_user
```

### Example: Delegation Chain

```
Root: Alice (full subspace)
  │
  ▼ delegate to Bob with path=/data/
Capability: Bob for /data/
  │
  ▼ delegate to Charlie with path=/data/public/
Capability: Charlie for /data/public/
  │
  ▼ delegate to Dana with time restriction
Capability: Dana for /data/public/ in 2024
```

Each step:
- Verifies previous capability is valid
- Signs handover message
- Restricts (never expands) the area

---

## Private Area Intersection (PAI)

PAI allows peers to discover shared interests without revealing information about interests they don't share.

### PrivateInterest

```rust
pub struct PrivateInterest {
    pub namespace_id: NamespacePublicKey,
    pub subspace_id: SubspaceConstraint,
    pub path: PathConstraint,
}
```

### Specificity Relationships

Interest A is **more specific** than B if:
- Same namespace
- B's subspace is `Any` OR A's subspace equals B's
- A's path extends B's path

Interests are **awkward** if neither comparable nor disjoint (one has `Any` subspace with long path, other has specific subspace with short path).

### Protocol Flow

#### 1. Salt Derivation

From handshake random bytes `rnd`:
- Initiator salt = `rnd`
- Responder salt = `~rnd` (bitwise NOT)

Different salts prevent mirroring attacks.

#### 2. Fragment Generation

Each peer generates `PaiFragment`s:

```rust
pub struct PaiFragment {
    pub hash: [u8; 32],   // Salted hash
    pub is_primary: bool, // True for actual interest, false for relaxation
}
```

**For `Any` subspace interests:**
- Send: `(hash(my_salt, interest), true)`

**For specific subspace interests:**
- Send: `(hash(my_salt, interest), true)`
- Send: `(hash(my_salt, relaxed_interest), false)`

Relaxation replaces specific subspace with `Any`.

#### 3. Local Comparison

Compute hashes using **peer's salt** for:
- Each interest
- All path prefixes of each interest

Compare with received fragments. Match if hashes equal AND at least one `is_primary` is true.

#### 4. Overlap Detection

```rust
pub enum OverlapType {
    Equal,           // Same interest
    WeMoreSpecific,  // We have more specific interest
    PeerMoreSpecific,// Peer has more specific interest
    Awkward,         // Neither comparable nor disjoint
}
```

#### 5. Overlap Announcement

```rust
pub struct OverlapAnnouncement {
    pub announcement_auth: [u8; 32],  // hash(my_salt, interest)
    pub enumeration_capability: Option<EncodedEnumerationCapability>,
}
```

The auth proves we know the interest (can only be computed by someone who knows it).

---

## Handshake Protocol

Based on Noise XX pattern, modified for Willow.

### Message Flow

```
Initiator                           Responder
    │                                   │
    │──── Hello {ephemeral_key} ────────►│
    │                                   │
    │◄─── HelloResponse {               │
    │      ephemeral_key,               │
    │      encrypted_static_key         │
    │     } ────────────────────────────│
    │                                   │
    │──── KeyExchangeComplete {         │
    │      encrypted_static_key         │
    │     } ────────────────────────────►│
    │                                   │
    │◄───── PaiFragments ──────────────►│
    │                                   │
    │◄───── OverlapAnnounce ───────────►│
    │                                   │
    │◄───── Capabilities ──────────────►│
    │                                   │
    │◄───── CapabilityAck ─────────────►│
    │                                   │
    │         Complete                   │
```

### Key Types

- **EphemeralKeyPair**: X25519 for key exchange (generated per session)
- **StaticKeyPair**: Ed25519 for identity (long-term)
- **SessionKey**: Derived symmetric keys for encryption

### Session Key Derivation

```rust
pub fn derive(
    ephemeral_shared: &SharedSecret,
    static_shared: Option<&SharedSecret>,
    is_initiator: bool,
) -> SessionKey
```

Produces `send_key` and `recv_key` (swapped for initiator/responder).

### PAI Random Derivation

```rust
pub fn derive_pai_rnd(state: &HandshakeState) -> [u8; 32] {
    SHA256("pai_rnd" || shared_secret)
}
```

---

## Security Model

### Threat Model

**Scenario 1: Alfie syncs with malicious Muriarty**
- Muriarty tries to learn about Alfie's interests

**Scenario 2: Alfie and Betty sync, active eavesdropper Epson attacks**
- Epson can read/modify all bytes on the wire

### Information Levels

| Level | Data | Protection |
|-------|------|------------|
| L0 | Entry payloads, full entry metadata | Requires valid read capability |
| L1 | NamespaceIds, SubspaceIds, Paths in capabilities | Protected by PAI, capability exchange |
| L2 | Other capability data (timestamps, non-ID keys) | Protected by encryption |
| L3 | Session behavior (timing, resource control) | Out of scope |

### What's Protected

1. **NamespaceIds**: Cannot learn namespaces you don't know about
2. **SubspaceIds**: Cannot learn who participates in a namespace
3. **Paths**: Cannot learn path structure

### What's NOT Protected

1. **Timestamps**: Easily guessable, not hidden
2. **Guess Confirmation**: Can confirm guesses about data you suspect

### Active Eavesdropper Defense

Epson faces a dilemma:

**Option 1: Don't manipulate handshake**
- Cannot derive session key
- Cannot decrypt messages

**Option 2: Replace public keys**
- Must produce valid capabilities for those keys
- Cannot forge capabilities without proper signatures
- Peers won't exchange sensitive data

---

## Implementation Details

### Capability Validation

```rust
impl CommunalCapability {
    pub fn is_valid(&self) -> bool {
        // Root (no delegations) always valid
        if self.delegations.is_empty() {
            return true;
        }

        let mut prev_receiver = self.user_key.clone();
        let mut prev_area = Area::subspace(self.user_key.clone());

        for (i, delegation) in self.delegations.iter().enumerate() {
            // Check area restriction
            if !prev_area.includes(&delegation.area) {
                return false;
            }

            // Verify signature over handover message
            let handover = self.compute_handover(i, &prev_area, delegation);
            if !delegation.signature.verify(&handover, &prev_receiver) {
                return false;
            }

            prev_receiver = delegation.user.clone();
            prev_area = delegation.area.clone();
        }

        true
    }
}
```

### Encoded Capability

For transmission, capabilities are encoded to omit context-known data:

```rust
pub struct EncodedCapability {
    pub is_communal: bool,
    pub access_mode: AccessMode,
    pub authority: EncodedAuthority,
    pub delegations: Vec<EncodedDelegation>,
}

pub struct EncodedDelegation {
    pub relative_area: RelativeArea,  // Only diff from context
    pub user: UserPublicKey,
    pub signature: Vec<u8>,
}
```

### MeadowcapAuthorisationToken

For authorizing entry writes:

```rust
pub struct MeadowcapAuthorisationToken {
    pub capability: EncodedCapability,
    pub signature: Vec<u8>,  // Over entry by capability receiver
}
```

**Validation:**
1. Capability is valid
2. Access mode is `Write`
3. Granted area includes the entry
4. Signature by receiver over encoded entry is valid

---

## Protocol Flow

### Complete Session

```
1. HANDSHAKE (Noise XX)
   ├── Exchange ephemeral keys
   ├── Exchange encrypted static keys  
   └── Derive: session_keys, rnd

2. PRIVATE AREA INTERSECTION
   ├── Generate fragments from interests
   ├── Exchange fragments (concurrent)
   ├── Detect overlaps locally
   └── Send overlap announcements with auth

3. CAPABILITY EXCHANGE
   ├── For each overlap, send encoded capability
   ├── Verify received capabilities
   ├── Acknowledge accepted/rejected
   └── Establish sync areas

4. DATA SYNCHRONIZATION
   └── Sync entries within granted areas
```

### State Machine (Handshake)

```
Start
  │
  ▼
KeyExchange ──── Hello ────►
  │
  ◄──── HelloResponse ────
  │
  │ ──── KeyExchangeComplete ────►
  ▼
PaiExchange
  │
  ◄────► PaiFragments
  │
  ◄────► OverlapAnnounce
  ▼
CapabilityExchange
  │
  ◄────► Capabilities
  │
  ◄────► CapabilityAck
  ▼
Complete
```

---

## References

- [Willow Protocol - Meadowcap](https://willowprotocol.org/specs/meadowcap/index.html)
- [Willow Protocol - Private Interest Overlap](https://willowprotocol.org/specs/pio/index.html)
- [Willow Protocol - Handshake and Encryption](https://willowprotocol.org/specs/handshake_and_encryption/index.html)
- [Noise Protocol Framework](https://noiseprotocol.org/)

---

## Netabase Extensions

This implementation extends Meadowcap for netabase's use case:

1. **Table-level granularity**: Subscriptions map to tables for fine-grained access
2. **Typed capabilities**: Generic `Capability<D, M>` for compile-time model safety
3. **libp2p integration**: Uses `PeerId` as `SubspaceId`/`UserPublicKey`
4. **Definition-aware**: Capabilities integrate with netabase's definition system
