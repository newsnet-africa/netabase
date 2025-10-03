use libp2p::{
    Multiaddr, PeerId,
    kad::{
        Addresses, Event as KadEvent, InboundRequest, Mode, ProgressStep, QueryId, QueryResult,
        QueryStats,
    },
};
use netabase_store::traits::NetabaseSchema;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot::Sender;

// Use the actual KBucketDistance type from libp2p
use libp2p::kad::KBucketDistance;

// Thread-safe storage for pending queries and their response channels
static PENDING_QUERIES: Lazy<Mutex<HashMap<QueryId, Box<dyn std::any::Any + Send>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Store a response channel for a query
pub fn store_query_response_channel<T: 'static + Send>(query_id: QueryId, sender: Sender<T>) {
    if let Ok(mut pending_queries) = PENDING_QUERIES.lock() {
        pending_queries.insert(query_id, Box::new(sender));
    }
}

/// Handle Kademlia behaviour events
pub fn handle_kad_event<S: NetabaseSchema>(kad_event: KadEvent) {
    match kad_event {
        KadEvent::InboundRequest { request } => {
            handle_inbound_request::<S>(request);
        }
        KadEvent::OutboundQueryProgressed {
            id,
            result,
            stats,
            step,
        } => {
            handle_outbound_query_progressed::<S>(id, result, stats, step);
        }
        KadEvent::RoutingUpdated {
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer,
        } => {
            handle_routing_updated::<S>(peer, is_new_peer, addresses, bucket_range, old_peer);
        }
        KadEvent::UnroutablePeer { peer } => {
            handle_unroutable_peer::<S>(peer);
        }
        KadEvent::RoutablePeer { peer, address } => {
            handle_routable_peer::<S>(peer, address);
        }
        KadEvent::PendingRoutablePeer { peer, address } => {
            handle_pending_routable_peer::<S>(peer, address);
        }
        KadEvent::ModeChanged { new_mode } => {
            handle_mode_changed::<S>(new_mode);
        }
    }
}

/// Handle inbound Kademlia requests
fn handle_inbound_request<S: NetabaseSchema>(request: InboundRequest) {
    println!("Handling Kademlia inbound request: {:?}", request);
}

/// Handle outbound query progress - wait for ProgressStep.last and return result
fn handle_outbound_query_progressed<S: NetabaseSchema>(
    id: QueryId,
    result: QueryResult,
    stats: QueryStats,
    step: ProgressStep,
) {
    println!(
        "Handling Kademlia outbound query progress - ID: {:?}, Result: {:?}, Stats: {:?}, Step: {:?}",
        id, result, stats, step
    );

    // Check if this is the last step of the query
    if step.last {
        println!("Query {:?} completed - this is the last step", id);

        // Try to find and send the result through the stored response channel
        if let Ok(mut pending_queries) = PENDING_QUERIES.lock() {
            if let Some(sender_box) = pending_queries.remove(&id) {
                // Handle different query result types
                let sent = match &result {
                    QueryResult::Bootstrap(_) => {
                        if let Ok(sender) = sender_box
                            .downcast::<Sender<Result<QueryResult, libp2p::kad::NoKnownPeers>>>()
                        {
                            sender.send(Ok(result)).is_ok()
                        } else {
                            false
                        }
                    }
                    QueryResult::GetProviders(_) => {
                        if let Ok(sender) = sender_box.downcast::<Sender<QueryResult>>() {
                            sender.send(result).is_ok()
                        } else {
                            false
                        }
                    }
                    QueryResult::GetRecord(_) => {
                        if let Ok(sender) = sender_box.downcast::<Sender<QueryResult>>() {
                            sender.send(result).is_ok()
                        } else {
                            false
                        }
                    }
                    QueryResult::PutRecord(_) => {
                        // Try for PutRecord-specific response channel first
                        if let Ok(sender) = sender_box
                            .downcast::<Sender<Result<QueryResult, libp2p::kad::store::Error>>>()
                        {
                            sender.send(Ok(result)).is_ok()
                        } else {
                            false
                        }
                    }
                    QueryResult::StartProviding(_) => {
                        if let Ok(sender) = sender_box
                            .downcast::<Sender<Result<QueryResult, libp2p::kad::store::Error>>>()
                        {
                            sender.send(Ok(result)).is_ok()
                        } else {
                            false
                        }
                    }
                    QueryResult::RepublishProvider(republish_result) => {
                        // Handle republish provider results if needed
                        println!("RepublishProvider result: {:?}", republish_result);
                        true
                    }
                    QueryResult::GetClosestPeers(_) => {
                        if let Ok(sender) = sender_box.downcast::<Sender<QueryResult>>() {
                            sender.send(result).is_ok()
                        } else {
                            false
                        }
                    }
                    QueryResult::RepublishRecord(republish_result) => {
                        // Handle republish record results if needed
                        println!("RepublishRecord result: {:?}", republish_result);
                        true
                    }
                };

                if !sent {
                    println!(
                        "Failed to send result for query {:?} - channel type mismatch",
                        id
                    );
                }
            } else {
                println!("No response channel found for completed query {:?}", id);
            }
        } else {
            println!("Failed to acquire lock on pending queries");
        }
    } else {
        println!("Query {:?} in progress - step is not last", id);
    }
}

/// Handle routing table updates
fn handle_routing_updated<S: NetabaseSchema>(
    peer: PeerId,
    is_new_peer: bool,
    addresses: Addresses,
    bucket_range: (KBucketDistance, KBucketDistance),
    old_peer: Option<PeerId>,
) {
    println!(
        "Handling Kademlia routing update - Peer: {:?}, New: {}, Addresses: {:?}, Bucket range: {:?}, Old peer: {:?}",
        peer, is_new_peer, addresses, bucket_range, old_peer
    );
}

/// Handle unroutable peer events
fn handle_unroutable_peer<S: NetabaseSchema>(peer: PeerId) {
    println!("Handling Kademlia unroutable peer: {:?}", peer);
}

/// Handle routable peer events
fn handle_routable_peer<S: NetabaseSchema>(peer: PeerId, address: Multiaddr) {
    println!(
        "Handling Kademlia routable peer: {:?} at {:?}",
        peer, address
    );
}

/// Handle pending routable peer events
fn handle_pending_routable_peer<S: NetabaseSchema>(peer: PeerId, address: Multiaddr) {
    println!(
        "Handling Kademlia pending routable peer: {:?} at {:?}",
        peer, address
    );
}

/// Handle mode change events
fn handle_mode_changed<S: NetabaseSchema>(new_mode: Mode) {
    println!("Handling Kademlia mode change to: {:?}", new_mode);
}
