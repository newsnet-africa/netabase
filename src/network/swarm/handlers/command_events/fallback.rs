use netabase_store::traits::NetabaseSchema;

use super::Command;

pub fn handle_fallback_command<S: NetabaseSchema>(command: Command<S>) {
    // TODO: Handle any unmatched command events or implement proper error handling
    println!("Fallback command handler: unhandled command={:?}", command);

    // This handler catches any commands that don't match the specific handlers
    // In the future, this could log errors, send default responses, or implement
    // graceful degradation for unsupported command types
}
