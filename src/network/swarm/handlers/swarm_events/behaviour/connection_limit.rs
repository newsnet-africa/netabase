use netabase_store::traits::NetabaseSchema;
use std::convert::Infallible;

/// Handle connection limit behaviour events
/// Note: libp2p connection limits use Infallible as the event type,
/// meaning this behaviour never emits events
pub fn handle_connection_limit_event<S: NetabaseSchema>(event: Infallible) {
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
