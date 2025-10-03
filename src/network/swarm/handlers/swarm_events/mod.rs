use netabase_store::traits::NetabaseSchema;

use crate::network::behaviour::clone_impl::NetabaseSwarmEvent;

// Handler modules
pub mod behaviour;
pub mod connection_closed;
pub mod connection_established;
pub mod dialing;
pub mod expired_listen_addr;
pub mod external_addr_confirmed;
pub mod external_addr_expired;
pub mod fallback;
pub mod incoming_connection;
pub mod incoming_connection_error;
pub mod listener_closed;
pub mod listener_error;
pub mod new_external_addr_candidate;
pub mod new_external_addr_of_peer;
pub mod new_listen_addr;
pub mod outgoing_connection_error;

pub fn handle_swarm_events<S: NetabaseSchema>(event: NetabaseSwarmEvent<S>) {
    match event.0 {
        libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
            behaviour::handle_behaviour_event::<S>(behaviour_event);
        }
        libp2p::swarm::SwarmEvent::ConnectionEstablished {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            concurrent_dial_errors,
            established_in,
        } => {
            connection_established::handle_connection_established::<S>(
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            );
        }
        libp2p::swarm::SwarmEvent::ConnectionClosed {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            cause,
        } => {
            connection_closed::handle_connection_closed::<S>(
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            );
        }
        libp2p::swarm::SwarmEvent::IncomingConnection {
            connection_id,
            local_addr,
            send_back_addr,
        } => {
            incoming_connection::handle_incoming_connection::<S>(
                connection_id,
                local_addr,
                send_back_addr,
            );
        }
        libp2p::swarm::SwarmEvent::IncomingConnectionError {
            connection_id,
            local_addr,
            send_back_addr,
            error,
            peer_id,
        } => {
            incoming_connection_error::handle_incoming_connection_error::<S>(
                connection_id,
                local_addr,
                send_back_addr,
                error,
                peer_id,
            );
        }
        libp2p::swarm::SwarmEvent::OutgoingConnectionError {
            connection_id,
            peer_id,
            error,
        } => {
            outgoing_connection_error::handle_outgoing_connection_error::<S>(
                connection_id,
                peer_id,
                error,
            );
        }
        libp2p::swarm::SwarmEvent::NewListenAddr {
            listener_id,
            address,
        } => {
            new_listen_addr::handle_new_listen_addr::<S>(listener_id, address);
        }
        libp2p::swarm::SwarmEvent::ExpiredListenAddr {
            listener_id,
            address,
        } => {
            expired_listen_addr::handle_expired_listen_addr::<S>(listener_id, address);
        }
        libp2p::swarm::SwarmEvent::ListenerClosed {
            listener_id,
            addresses,
            reason,
        } => {
            listener_closed::handle_listener_closed::<S>(listener_id, addresses, reason);
        }
        libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => {
            listener_error::handle_listener_error::<S>(listener_id, error);
        }
        libp2p::swarm::SwarmEvent::Dialing {
            peer_id,
            connection_id,
        } => {
            dialing::handle_dialing::<S>(peer_id, connection_id);
        }
        libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => {
            new_external_addr_candidate::handle_new_external_addr_candidate::<S>(address);
        }
        libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => {
            external_addr_confirmed::handle_external_addr_confirmed::<S>(address);
        }
        libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => {
            external_addr_expired::handle_external_addr_expired::<S>(address);
        }
        libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
            new_external_addr_of_peer::handle_new_external_addr_of_peer::<S>(peer_id, address);
        }
        _ => {
            fallback::handle_fallback_event::<S>(event);
        }
    }
}
