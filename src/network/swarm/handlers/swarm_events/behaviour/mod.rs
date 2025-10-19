use crate::network::behaviour::NetabaseBehaviourEvent;
use netabase_store::traits::definition::NetabaseDefinitionTrait;

pub mod connection_limit;
pub mod identify;
pub mod kad;
#[cfg(feature = "native")]
pub mod mdns;

use connection_limit::handle_connection_limit_event;
use identify::handle_identify_event;
use kad::handle_kad_event;
#[cfg(feature = "native")]
use mdns::handle_mdns_event;

/// Handle all NetabaseBehaviour events by delegating to specific handlers
pub(crate) fn handle_behaviour_event<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    behaviour_event: NetabaseBehaviourEvent<D>,
) {
    match behaviour_event {
        NetabaseBehaviourEvent::Kad(kad_event) => {
            handle_kad_event::<D>(kad_event);
        }
        NetabaseBehaviourEvent::Identify(identify_event) => {
            handle_identify_event::<D>(identify_event);
        }
        #[cfg(feature = "native")]
        NetabaseBehaviourEvent::Mdns(mdns_event) => {
            handle_mdns_event::<D>(mdns_event);
        }
        NetabaseBehaviourEvent::ConnectionLimit(connection_limit_event) => {
            handle_connection_limit_event::<D>(connection_limit_event);
        }
    }
}
