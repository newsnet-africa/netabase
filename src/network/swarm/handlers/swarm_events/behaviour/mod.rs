use crate::network::behaviour::NetabaseBehaviourEvent;
use netabase_store::traits::NetabaseSchema;

pub mod connection_limit;
pub mod identify;
pub mod kad;
pub mod mdns;

use connection_limit::handle_connection_limit_event;
use identify::handle_identify_event;
use kad::handle_kad_event;
use mdns::handle_mdns_event;

/// Handle all NetabaseBehaviour events by delegating to specific handlers
pub fn handle_behaviour_event<S: NetabaseSchema>(behaviour_event: NetabaseBehaviourEvent<S>) {
    match behaviour_event {
        NetabaseBehaviourEvent::Kad(kad_event) => {
            handle_kad_event::<S>(kad_event);
        }
        NetabaseBehaviourEvent::Identify(identify_event) => {
            handle_identify_event::<S>(identify_event);
        }
        NetabaseBehaviourEvent::Mdns(mdns_event) => {
            handle_mdns_event::<S>(mdns_event);
        }
        NetabaseBehaviourEvent::ConnectionLimit(connection_limit_event) => {
            handle_connection_limit_event::<S>(connection_limit_event);
        }
    }
}
