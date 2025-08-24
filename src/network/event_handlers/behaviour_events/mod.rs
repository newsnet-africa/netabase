use libp2p::Swarm;
use tokio::sync::MutexGuard;

use crate::network::behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent, NetabaseEvent};

mod identify_events;
mod kad_events;
mod mdns_events;

pub(super) fn handle_behaviour_events(
    behavior_event: NetabaseBehaviourEvent,
    swarm: &mut MutexGuard<'_, Swarm<NetabaseBehaviour>>,
) {
    match behavior_event {
        NetabaseBehaviourEvent::Kad(event) => kad_events::handle_kad_events(event),
        NetabaseBehaviourEvent::Identify(event) => identify_events::handle_identify_events(event),
        NetabaseBehaviourEvent::Mdns(event) => mdns_events::handle_mdns_events(event, swarm),
    }
}
