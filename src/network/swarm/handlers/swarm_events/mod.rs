#[cfg(feature = "native")]
use netabase_store::traits::definition::NetabaseDefinition;

#[cfg(feature = "native")]
use crate::network::behaviour::clone_impl::NetabaseSwarmEvent;

// Handler modules - only available for native builds
#[cfg(feature = "native")]
pub mod behaviour;
#[cfg(feature = "native")]
pub mod connection_closed;
#[cfg(feature = "native")]
pub mod connection_established;
#[cfg(feature = "native")]
pub mod dialing;
#[cfg(feature = "native")]
pub mod expired_listen_addr;
#[cfg(feature = "native")]
pub mod external_addr_confirmed;
#[cfg(feature = "native")]
pub mod external_addr_expired;
#[cfg(feature = "native")]
pub mod fallback;
#[cfg(feature = "native")]
pub mod incoming_connection;
#[cfg(feature = "native")]
pub mod incoming_connection_error;
#[cfg(feature = "native")]
pub mod listener_closed;
#[cfg(feature = "native")]
pub mod listener_error;
#[cfg(feature = "native")]
pub mod new_external_addr_candidate;
#[cfg(feature = "native")]
pub mod new_external_addr_of_peer;
#[cfg(feature = "native")]
pub mod new_listen_addr;
#[cfg(feature = "native")]
pub mod outgoing_connection_error;

#[cfg(feature = "native")]
pub fn handle_swarm_events<D: NetabaseDefinition + Send + Sync + 'static>(event: NetabaseSwarmEvent<D>) {
    match event.0 {
        libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
            behaviour::handle_behaviour_event::<D>(behaviour_event);
        }
        libp2p::swarm::SwarmEvent::ConnectionEstablished {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            concurrent_dial_errors,
            established_in,
        } => {
            connection_established::handle_connection_established::<D>(
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
            connection_closed::handle_connection_closed::<D>(
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
            incoming_connection::handle_incoming_connection::<D>(
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
            incoming_connection_error::handle_incoming_connection_error::<D>(
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
            outgoing_connection_error::handle_outgoing_connection_error::<D>(
                connection_id,
                peer_id,
                error,
            );
        }
        libp2p::swarm::SwarmEvent::NewListenAddr {
            listener_id,
            address,
        } => {
            new_listen_addr::handle_new_listen_addr::<D>(listener_id, address);
        }
        libp2p::swarm::SwarmEvent::ExpiredListenAddr {
            listener_id,
            address,
        } => {
            expired_listen_addr::handle_expired_listen_addr::<D>(listener_id, address);
        }
        libp2p::swarm::SwarmEvent::ListenerClosed {
            listener_id,
            addresses,
            reason,
        } => {
            listener_closed::handle_listener_closed::<D>(listener_id, addresses, reason);
        }
        libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => {
            listener_error::handle_listener_error::<D>(listener_id, error);
        }
        libp2p::swarm::SwarmEvent::Dialing {
            peer_id,
            connection_id,
        } => {
            dialing::handle_dialing::<D>(peer_id, connection_id);
        }
        libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => {
            new_external_addr_candidate::handle_new_external_addr_candidate::<D>(address);
        }
        libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => {
            external_addr_confirmed::handle_external_addr_confirmed::<D>(address);
        }
        libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => {
            external_addr_expired::handle_external_addr_expired::<D>(address);
        }
        libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
            new_external_addr_of_peer::handle_new_external_addr_of_peer::<D>(peer_id, address);
        }
        _ => {
            fallback::handle_fallback_event::<D>(event);
        }
    }
}
