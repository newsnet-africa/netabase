use std::sync::Arc;

use libp2p::{
    Swarm,
    futures::{SinkExt, StreamExt},
    swarm::SwarmEvent,
};
use tokio::sync::Mutex;

use crate::network::behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent, NetabaseEvent};

mod behaviour_events;
mod command_events;
mod connection_events;

async fn handle_events(
    swarm: Arc<Mutex<Swarm<NetabaseBehaviour>>>,
    mut event_sender: std::sync::mpsc::Sender<NetabaseEvent>,
) {
    loop {
        {
            let mut locked_swarm = swarm.lock().await;
            tokio::select! {
                event = locked_swarm.select_next_some() => {
                    let wrapped_event = NetabaseEvent(event);
                    event_sender.send(wrapped_event.clone());
                    let k = match wrapped_event.0 {
                        SwarmEvent::Behaviour(_) => {},
                        SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => {},
                        SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {},
                        SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {},
                        SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error, peer_id } => {},
                        SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => {},
                        SwarmEvent::NewListenAddr { listener_id, address } => {},
                        SwarmEvent::ExpiredListenAddr { listener_id, address } => {},
                        SwarmEvent::ListenerClosed { listener_id, addresses, reason } => {},
                        SwarmEvent::ListenerError { listener_id, error } => {},
                        SwarmEvent::Dialing { peer_id, connection_id } => {},
                        SwarmEvent::NewExternalAddrCandidate { address } => {},
                        SwarmEvent::ExternalAddrConfirmed { address } => {},
                        SwarmEvent::ExternalAddrExpired { address } => {},
                        SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {},
                        _ => {},
                    };
                }
            }
        }
    }
}
