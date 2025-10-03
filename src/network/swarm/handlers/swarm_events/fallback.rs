use netabase_store::traits::NetabaseSchema;

use crate::network::behaviour::clone_impl::NetabaseSwarmEvent;

pub fn handle_fallback_event<S: NetabaseSchema>(event: NetabaseSwarmEvent<S>) {
    // TODO: Implement fallback event handling
    println!("Unhandled swarm event (fallback): {:?}", event);

    // This handler catches any SwarmEvent variants that are not explicitly
    // handled by other specific handlers. This is useful for:
    // 1. Future libp2p versions that might add new SwarmEvent variants
    // 2. Debugging and logging unhandled events
    // 3. Providing a safe fallback when the match arms are incomplete

    eprintln!(
        "Warning: SwarmEvent fell through to fallback handler. Consider implementing a specific handler for this event type."
    );
}
