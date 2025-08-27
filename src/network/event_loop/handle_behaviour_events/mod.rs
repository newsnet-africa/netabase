use crate::network::{
    behaviour::NetabaseBehaviourEvent,
    event_loop::handle_behaviour_events::{
        handle_kad_events::handle_kad_event, handle_mdns_events::handle_mdns_event,
    },
};

pub mod handle_identify_events;
pub mod handle_kad_events;
pub mod handle_mdns_events;

pub fn handle_behaviour_event(event: NetabaseBehaviourEvent) {
    match event {
        NetabaseBehaviourEvent::Kad(event) => handle_kad_event(event),
        NetabaseBehaviourEvent::Mdns(event) => handle_mdns_event(event),
        NetabaseBehaviourEvent::Identify(event) => todo!(),
    }
}
