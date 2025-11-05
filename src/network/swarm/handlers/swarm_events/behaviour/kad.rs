use libp2p::{
    Multiaddr, PeerId,
    kad::{
        Addresses, Event as KadEvent, InboundRequest, Mode, ProgressStep, QueryId, QueryResult,
        QueryStats,
    },
};
use log::{debug, info, warn};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
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
pub fn handle_kad_event<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(kad_event: KadEvent)
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    match kad_event {
        KadEvent::InboundRequest { request } => {
            handle_inbound_request::<D>(request);
        }
        KadEvent::OutboundQueryProgressed {
            id,
            result,
            stats,
            step,
        } => {
            handle_outbound_query_progressed::<D>(id, result, stats, step);
        }
        KadEvent::RoutingUpdated {
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer,
        } => {
            handle_routing_updated::<D>(peer, is_new_peer, addresses, bucket_range, old_peer);
        }
        KadEvent::UnroutablePeer { peer } => {
            handle_unroutable_peer::<D>(peer);
        }
        KadEvent::RoutablePeer { peer, address } => {
            handle_routable_peer::<D>(peer, address);
        }
        KadEvent::PendingRoutablePeer { peer, address } => {
            handle_pending_routable_peer::<D>(peer, address);
        }
        KadEvent::ModeChanged { new_mode } => {
            handle_mode_changed::<D>(new_mode);
        }
    }
}

/// Handle inbound Kademlia requests
fn handle_inbound_request<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    request: InboundRequest,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Only log PutRecord requests in a clean format
    if let InboundRequest::PutRecord { source, record, .. } = &request {
        let peer_short = format!("{}", source).chars().take(8).collect::<String>();
        info!("📥 Receiving message from peer {}...", peer_short);
        if record.is_some() {
            debug!("   Record received and stored\n");
        }
    }
}

/// Handle outbound query progress - wait for ProgressStep.last and return result
fn handle_outbound_query_progressed<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    id: QueryId,
    result: QueryResult,
    _stats: QueryStats,
    step: ProgressStep,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Check if this is the last step of the query
    if step.last {
        // Try to find and send the result through the stored response channel
        if let Ok(mut pending_queries) = PENDING_QUERIES.lock()
            && let Some(sender_box) = pending_queries.remove(&id)
        {
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
                    debug!("RepublishProvider result: {:?}", republish_result);
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
                    debug!("RepublishRecord result: {:?}", republish_result);
                    true
                }
            };

            if !sent {
                warn!(
                    "Failed to send result for query {:?} - channel type mismatch",
                    id
                );
            }
        }
    }
}

/// Handle routing table updates
fn handle_routing_updated<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    _peer: PeerId,
    _is_new_peer: bool,
    _addresses: Addresses,
    _bucket_range: (KBucketDistance, KBucketDistance),
    _old_peer: Option<PeerId>,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Silent - routing updates are internal DHT operations
}

/// Handle unroutable peer events
fn handle_unroutable_peer<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(_peer: PeerId)
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Silent - unroutable peers are normal in P2P networks
}

/// Handle routable peer events
fn handle_routable_peer<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    _peer: PeerId,
    _address: Multiaddr,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Silent - routable peer detection is internal
}

/// Handle pending routable peer events
fn handle_pending_routable_peer<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    _peer: PeerId,
    _address: Multiaddr,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Silent - pending peer routing is internal
}

/// Handle mode change events
fn handle_mode_changed<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(_new_mode: Mode)
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // Silent - mode changes are internal DHT operations
}
