//! # Handshake and Encryption Module
//!
//! This module implements the Willow Handshake Protocol adapted for netabase.
//! The handshake establishes a secure, authenticated channel between peers
//! and enables private discovery of mutual interests.
//!
//! ## Two-Phase Connection Protocol
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        Connection Establishment                          │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  Phase 1: Encrypted Tunnel (Noise XX)                                  │
//! │  ┌──────────────────────────────────────────────────────────────────┐  │
//! │  │  Initiator                              Responder                │  │
//! │  │      │                                      │                    │  │
//! │  │      │──── e (ephemeral pubkey) ──────────►│                    │  │
//! │  │      │                                      │                    │  │
//! │  │      │◄─── e, ee, s, es ──────────────────│                    │  │
//! │  │      │    (eph + encrypted static)         │                    │  │
//! │  │      │                                      │                    │  │
//! │  │      │──── s, se ────────────────────────►│                    │  │
//! │  │      │    (encrypted static)               │                    │  │
//! │  │      │                                      │                    │  │
//! │  │      ├─────── Session Keys Derived ────────┤                    │  │
//! │  └──────────────────────────────────────────────────────────────────┘  │
//! │                                                                         │
//! │  Phase 2: Private Area Intersection + Capability Exchange              │
//! │  ┌──────────────────────────────────────────────────────────────────┐  │
//! │  │  Initiator                              Responder                │  │
//! │  │      │                                      │                    │  │
//! │  │      │◄──── PAI Fragments ───────────────►│                    │  │
//! │  │      │     (salted interest hashes)        │                    │  │
//! │  │      │                                      │                    │  │
//! │  │      │◄──── Overlap Announcements ───────►│                    │  │
//! │  │      │     (with authentication)           │                    │  │
//! │  │      │                                      │                    │  │
//! │  │      │◄──── Capabilities ────────────────►│                    │  │
//! │  │      │     (encoded, for overlaps only)    │                    │  │
//! │  │      │                                      │                    │  │
//! │  │      │◄──── Capability Ack ──────────────►│                    │  │
//! │  └──────────────────────────────────────────────────────────────────┘  │
//! │                                                                         │
//! │  Data Synchronization can now begin...                                 │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Properties
//!
//! ### Forward Secrecy
//!
//! Each session uses fresh ephemeral X25519 keys. Compromising a static key
//! doesn't compromise past session data:
//!
//! ```text
//! Session 1: ephemeral_1 + static → session_key_1
//! Session 2: ephemeral_2 + static → session_key_2
//!
//! Compromising `static` doesn't reveal ephemeral_1 or ephemeral_2,
//! so session_key_1 and session_key_2 remain secure.
//! ```
//!
//! ### Identity Authentication
//!
//! Both peers prove knowledge of their static key's secret:
//!
//! ```text
//! Responder proves identity: encrypts static with key derived from ee
//! Initiator proves identity: encrypts static with key derived from ee+es
//!
//! Without the static secret key, cannot produce valid encryption
//! ```
//!
//! ### Active Eavesdropper Protection
//!
//! An active attacker (Epson) faces a dilemma:
//!
//! ```text
//! Option 1: Don't manipulate handshake
//!   → Cannot derive session key
//!   → Cannot decrypt subsequent messages
//!
//! Option 2: Replace public keys with own
//!   → Must produce valid capabilities for those keys
//!   → Cannot forge capabilities without original signer
//!   → Peers won't send sensitive data
//! ```
//!
//! ### Private Interest Protection
//!
//! The PAI phase protects interest confidentiality:
//!
//! ```text
//! - Interests are only revealed through salted hashes
//! - Salt is derived from the handshake (cannot be predicted)
//! - Initiator uses `rnd`, responder uses `~rnd` (prevents mirroring)
//! - Capability exchange only happens after overlap is mutually detected
//! ```
//!
//! ## Module Structure
//!
//! - [`keys`]: Cryptographic key types (ephemeral, static, signatures)
//! - [`state`]: Handshake state machine
//! - [`protocol`]: Message processing and transitions
//! - [`challenge`]: Challenge-response authentication
//! - [`session`]: Session key derivation and encrypted channel
//!
//! ## Example: Complete Handshake
//!
//! ```rust
//! use netabase::data::network::handshake::{
//!     HandshakeState, HandshakePhase, StaticKeyPair,
//!     protocol::{generate_hello, process_initiator_message, process_responder_message,
//!                derive_session_keys, derive_pai_rnd, HandshakeMessage},
//! };
//!
//! // Setup: both peers generate their static keys
//! let initiator_static = StaticKeyPair::generate();
//! let responder_static = StaticKeyPair::generate();
//!
//! // Our interests (subscription hashes we want to sync)
//! let initiator_interests: Vec<[u8; 32]> = vec![[1u8; 32], [2u8; 32]];
//! let responder_interests: Vec<[u8; 32]> = vec![[1u8; 32], [3u8; 32]];
//!
//! // Create handshake states
//! let mut initiator_state = HandshakeState::new_initiator(initiator_static, initiator_interests);
//! let mut responder_state = HandshakeState::new_responder(responder_static, responder_interests);
//!
//! // Step 1: Initiator generates Hello message
//! let hello = match &mut initiator_state {
//!     HandshakeState::Initiator(s) => generate_hello(s),
//!     _ => unreachable!(),
//! };
//!
//! // Step 2: Responder processes Hello, generates HelloResponse
//! let hello_response = match &mut responder_state {
//!     HandshakeState::Responder(s) => process_responder_message(s, hello)
//!         .expect("Should process hello")
//!         .expect("Should produce response"),
//!     _ => unreachable!(),
//! };
//!
//! // Step 3: Initiator processes HelloResponse, generates KeyExchangeComplete
//! let key_exchange_complete = match &mut initiator_state {
//!     HandshakeState::Initiator(s) => process_initiator_message(s, hello_response)
//!         .expect("Should process response")
//!         .expect("Should produce key exchange message"),
//!     _ => unreachable!(),
//! };
//!
//! // Step 4: Responder processes KeyExchangeComplete
//! match &mut responder_state {
//!     HandshakeState::Responder(s) => {
//!         let _ = process_responder_message(s, key_exchange_complete)
//!             .expect("Should process key exchange");
//!     }
//!     _ => unreachable!(),
//! };
//!
//! // Both sides are now in PaiExchange phase and can derive session keys
//! assert_eq!(initiator_state.phase(), HandshakePhase::PaiExchange);
//! assert_eq!(responder_state.phase(), HandshakePhase::PaiExchange);
//!
//! // Derive session keys and PAI random bytes
//! let _initiator_session = derive_session_keys(&initiator_state)
//!     .expect("Should derive session keys");
//! let initiator_rnd = derive_pai_rnd(&initiator_state)
//!     .expect("Should derive PAI rnd");
//!
//! // The rnd can now be used for Private Area Intersection
//! assert_eq!(initiator_rnd.len(), 32);
//! ```
//!
//! ## Noise XX Pattern
//!
//! This implementation is based on the Noise protocol framework's XX pattern,
//! modified for Willow:
//!
//! - Uses "Nose" prefix instead of "Noise" in protocol names
//! - Always hashes protocol name (no zero-padding short names)
//! - No message payloads during handshake (saves 16 bytes per message)
//! - Algorithm names may be non-standard
//!
//! The XX pattern provides mutual authentication with no prior knowledge:
//! - `X`: Initiator static key transmitted, encrypted
//! - `X`: Responder static key transmitted, encrypted

pub mod challenge;
pub mod keys;
pub mod protocol;
pub mod session;
pub mod state;

pub use challenge::{Challenge, ChallengeResponse, InterestReveal};
pub use keys::{EphemeralKeyPair, EphemeralPublicKey, EphemeralSecretKey, StaticKeyPair};
pub use protocol::{HandshakeError, HandshakeMessage};
pub use session::{EncryptedChannel, SessionKey};
pub use state::{ConnectionState, HandshakePhase, HandshakeState};
