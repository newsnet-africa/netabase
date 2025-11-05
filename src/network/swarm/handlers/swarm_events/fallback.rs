use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use log::{debug, warn};

use crate::network::behaviour::clone_impl::NetabaseSwarmEvent;

pub fn handle_fallback_event<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    event: NetabaseSwarmEvent<D>,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,{
    // TODO: Implement fallback event handling
    debug!("Unhandled swarm event (fallback): {:?}", event);

    // This handler catches any SwarmEvent variants that are not explicitly
    // handled by other specific handlers. This is useful for:
    // 1. Future libp2p versions that might add new SwarmEvent variants
    // 2. Debugging and logging unhandled events
    // 3. Providing a safe fallback when the match arms are incomplete

    warn!(
        "Warning: SwarmEvent fell through to fallback handler. Consider implementing a specific handler for this event type."
    );
}
