//! Type-safe primitives for Netabase networking layer.
//!
//! This module provides strongly-typed primitives that ensure correctness
//! at compile-time rather than runtime. All networking-related types live here,
//! separate from storage primitives.

pub mod node_id;
pub mod path;
pub mod rank;
pub mod range;

pub use node_id::NodeId;
pub use path::{Path, PathBuilder, PathNode, PathSegment, PathValidationError};
pub use rank::{ConflictRank, ConflictStrategy, LamportClock, RankStrategy};
pub use range::{KeyRange, NDimensionalRange, NodeIdRange, PathRange, SecondaryKeyRange};

