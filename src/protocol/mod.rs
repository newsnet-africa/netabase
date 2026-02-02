//! Protocol state machines for Netabase network communication.
//!
//! This module contains the actual implementation of the Netabase protocol,
//! including handshake, query execution, synchronization, and capability
//! management. These are transport-agnostic - they work with any transport
//! that can send/receive messages.

pub mod handshake;
pub mod query;
pub mod sync;
pub mod session;

pub use handshake::{HandshakeState, HandshakeStateMachine};
pub use query::{QueryHandler, QueryResult};
pub use sync::{SyncHandler, SyncState};
pub use session::{PeerSession, SessionManager};
