use libp2p::Swarm;
use libp2p::kad::QueryId;
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
use crate::network::event_loop::handle_commands::database_commands::DatabaseOperationContext;
use crate::network::event_messages::command_messages::CommandResponse;
use crate::network::{
    behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent},
    event_loop::handle_behaviour_events::{
        handle_identify_events::handle_identify_event, handle_kad_events::handle_kad_event,
        handle_mdns_events::handle_mdns_event,
    },
};

pub mod handle_identify_events;
pub mod handle_kad_events;
pub mod handle_mdns_events;

pub fn handle_behaviour_event<
    K: NetabaseSchemaKey + std::fmt::Debug,
    V: NetabaseSchema + std::fmt::Debug,
>(
    event: NetabaseBehaviourEvent,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    match event {
        NetabaseBehaviourEvent::Kad(event) => {
            handle_kad_event(event, query_queue, database_context)
        }
        NetabaseBehaviourEvent::Mdns(event) => handle_mdns_event(event),
        NetabaseBehaviourEvent::Identify(event) => handle_identify_event(event),
    }
}
