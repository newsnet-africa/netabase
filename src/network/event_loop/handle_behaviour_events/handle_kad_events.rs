use libp2p::kad::{self, QueryId};
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::netabase_trait::{
    NetabaseRegistery, NetabaseRegistryKey, NetabaseSchema, NetabaseSchemaKey,
};
use crate::network::event_loop::handle_commands::database_commands::{
    DatabaseOperationContext, process_database_dht_response,
};
use crate::network::event_messages::command_messages::{CommandResponse, NetworkResponse};

pub fn handle_kad_event<R: NetabaseRegistery>(
    event: kad::Event,
    pending_queries: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<R>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext<R::KeyRegistry>>,
) {
    println!("Kademlia Event: {event:?}");
    match event {
        kad::Event::InboundRequest { request } => {}
        kad::Event::OutboundQueryProgressed {
            id,
            result,
            stats,
            step,
        } => {
            // Handle query completion based on result type
            if let Some(sender) = pending_queries.remove(&id) {
                // First check if this is a database operation
                if let Some(db_response) =
                    process_database_dht_response::<R>(id, &result, database_context)
                {
                    let _ = sender.send(db_response);
                    return;
                }

                // If not a database operation, handle as network operation
                let response = match result {
                    kad::QueryResult::GetRecord(get_record_result) => {
                        CommandResponse::Network(NetworkResponse::DhtGetRecord(get_record_result))
                    }
                    kad::QueryResult::PutRecord(put_record_result) => {
                        CommandResponse::Network(NetworkResponse::DhtPutRecord(put_record_result))
                    }
                    kad::QueryResult::GetClosestPeers(get_closest_peers_result) => {
                        CommandResponse::Network(NetworkResponse::DhtGetClosestPeers(
                            get_closest_peers_result,
                        ))
                    }
                    kad::QueryResult::GetProviders(get_providers_result) => {
                        CommandResponse::Network(NetworkResponse::DhtGetProviders(
                            get_providers_result,
                        ))
                    }
                    kad::QueryResult::StartProviding(start_providing_result) => {
                        CommandResponse::Network(NetworkResponse::DhtStartProviding(
                            start_providing_result,
                        ))
                    }
                    kad::QueryResult::RepublishProvider(republish_provider_result) => {
                        CommandResponse::Network(NetworkResponse::DhtRepublishProvider(
                            republish_provider_result,
                        ))
                    }
                    kad::QueryResult::Bootstrap(bootstrap_result) => {
                        CommandResponse::Network(NetworkResponse::DhtBootstrap(bootstrap_result))
                    }
                    kad::QueryResult::RepublishRecord(republish_record_result) => {
                        CommandResponse::Network(NetworkResponse::DhtRepublishRecord(
                            republish_record_result,
                        ))
                    }
                };

                let _ = sender.send(response);
            }
        }
        kad::Event::RoutingUpdated {
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer,
        } => {}
        kad::Event::UnroutablePeer { peer } => {}
        kad::Event::RoutablePeer { peer, address } => {}
        kad::Event::PendingRoutablePeer { peer, address } => {}
        kad::Event::ModeChanged { new_mode } => {}
    }
}
