use libp2p::kad;

use crate::network::event_handlers::PendingQueries;

pub(super) fn handle_kad_events(kad_event: kad::Event, pending_queries: &mut PendingQueries) {
    match kad_event {
        kad::Event::InboundRequest { request } => {
            // Handle inbound requests if needed
        }
        kad::Event::OutboundQueryProgressed {
            id,
            result,
            stats: _,
            step,
        } => {
            // Handle query completion when step.last is true
            if step.last {
                match result {
                    libp2p::kad::QueryResult::PutRecord(res) => {
                        let result = res.clone().map_err(|e| anyhow::anyhow!(e));
                        pending_queries.complete_put_query(&id, result);
                    }
                    libp2p::kad::QueryResult::GetRecord(res) => {
                        let result = res.clone().map_err(|e| anyhow::anyhow!(e));
                        pending_queries.complete_get_query(&id, result);
                    }
                    _ => {
                        // Handle other query result types if needed
                    }
                }
            }
        }
        kad::Event::RoutingUpdated {
            peer: _,
            is_new_peer: _,
            addresses: _,
            bucket_range: _,
            old_peer: _,
        } => {
            // Handle routing table updates if needed
        }
        kad::Event::UnroutablePeer { peer: _ } => {
            // Handle unroutable peer events if needed
        }
        kad::Event::RoutablePeer {
            peer: _,
            address: _,
        } => {
            // Handle routable peer events if needed
        }
        kad::Event::PendingRoutablePeer {
            peer: _,
            address: _,
        } => {
            // Handle pending routable peer events if needed
        }
        kad::Event::ModeChanged { new_mode: _ } => {
            // Handle DHT mode changes if needed
        }
    }
}
