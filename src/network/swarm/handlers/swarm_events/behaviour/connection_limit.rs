use netabase_store::traits::definition::NetabaseDefinitionTrait;
use std::convert::Infallible;

/// Handle connection limit behaviour events
/// Note: libp2p connection limits use Infallible as the event type,
/// meaning this behaviour never emits events
pub fn handle_connection_limit_event<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    event: Infallible,
) where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // This match is unreachable since Infallible can never be constructed
    // But we include it for completeness and future-proofing
    match event {}

    // Note: The connection limits behaviour in libp2p works by enforcing limits
    // at the swarm level but doesn't emit events. Connection limiting is handled
    // automatically by rejecting new connections when limits are reached.
    //
    // If you need to handle connection limit events, you would typically:
    // - Monitor connection establishment/closure events in other handlers
    // - Track connection counts manually
    // - Implement custom logic based on connection state changes
}
