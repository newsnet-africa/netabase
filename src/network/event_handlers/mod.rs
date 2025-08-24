use std::sync::Arc;

use libp2p::{
    Swarm,
    futures::{SinkExt, StreamExt},
    swarm::SwarmEvent,
};
use tokio::sync::Mutex;

use crate::{
    NetabaseCommand, init_logging,
    network::{
        behaviour::{NetabaseBehaviour, NetabaseEvent},
        event_handlers::behaviour_events::handle_behaviour_events,
    },
};

mod behaviour_events;
mod command_events;
mod connection_events;

pub async fn handle_events(
    swarm: Arc<Mutex<Swarm<NetabaseBehaviour>>>,
    event_sender: tokio::sync::broadcast::Sender<NetabaseEvent>,
    mut netabase_command_receiver: tokio::sync::mpsc::Receiver<NetabaseCommand>,
) {
    init_logging();
    loop {
        {
            let mut locked_swarm = swarm.lock().await;

            tokio::select! {
                event = locked_swarm.select_next_some() => {
            eprintln!("Sending Event");
                    let wrapped_event = NetabaseEvent(event);
                    let _send_event_result = event_sender.send(wrapped_event.clone());
                    match wrapped_event.0 {
                        SwarmEvent::Behaviour(behaviour_event) => {
                            // handle_behaviour_events(behaviour_event, &mut locked_swarm);
                        },
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
                },
                wrapped_command = netabase_command_receiver.recv() => {
                    match wrapped_command {
                        Some(command) => match command {
                            NetabaseCommand::Close => {
                                eprintln!("Closing swarm");
                                break
                            },
                            NetabaseCommand::Database => todo!(),
                        },
                        None => todo!(),
                    }
                }
            }
        }
    }
}
