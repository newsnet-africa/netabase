use libp2p::Swarm;
use libp2p::futures::StreamExt;
use libp2p::kad::QueryId;
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::config::DefaultNetabaseConfig;
use crate::netabase_trait::{
    self, NetabaseRegistery, NetabaseRegistryKey, NetabaseSchema, NetabaseSchemaKey,
};
use crate::network::behaviour::NetabaseBehaviour;
use crate::network::event_loop::handle_behaviour_events::handle_behaviour_event;
use crate::network::event_loop::handle_commands::{
    database_commands::DatabaseOperationContext, handle_command,
};
use crate::network::event_messages::command_messages::{CommandResponse, CommandWithResponse};
use crate::network::event_messages::swarm_messages::NetabaseEvent;
pub mod handle_behaviour_events;
pub mod handle_commands;

pub async fn event_loop<
    K: NetabaseRegistryKey + std::fmt::Debug,
    V: NetabaseRegistery + std::fmt::Debug,
>(
    swarm: &mut Swarm<NetabaseBehaviour>,
    mut event_sender: tokio::sync::broadcast::Sender<NetabaseEvent>,
    mut command_receiver: tokio::sync::mpsc::UnboundedReceiver<CommandWithResponse<K, V>>,
    config: &DefaultNetabaseConfig,
) {
    let mut query_queue: HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>> = HashMap::new();
    let mut database_context: HashMap<QueryId, DatabaseOperationContext> = HashMap::new();
    let auto_connect_enabled = config.swarm_config().mdns_auto_connect();
    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                let sent_event = NetabaseEvent(event);
                let _ = event_sender.send(sent_event.clone());
                match sent_event.0 {
                    libp2p::swarm::SwarmEvent::Behaviour(event) => handle_behaviour_event(event, &mut query_queue, &mut database_context, swarm, auto_connect_enabled),
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id: _, connection_id: _, endpoint: _, num_established: _, concurrent_dial_errors: _, established_in: _ } => {},
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id: _, connection_id: _, endpoint: _, num_established: _, cause: _ } => {},
                    libp2p::swarm::SwarmEvent::IncomingConnection { connection_id: _, local_addr: _, send_back_addr: _ } => {},
                    libp2p::swarm::SwarmEvent::IncomingConnectionError { connection_id: _, local_addr: _, send_back_addr: _, error: _, peer_id: _ } => {},
                    libp2p::swarm::SwarmEvent::OutgoingConnectionError { connection_id: _, peer_id: _, error: _ } => {},
                    libp2p::swarm::SwarmEvent::NewListenAddr { listener_id: _, address: _ } => {},
                    libp2p::swarm::SwarmEvent::ExpiredListenAddr { listener_id: _, address: _ } => {},
                    libp2p::swarm::SwarmEvent::ListenerClosed { listener_id: _, addresses: _, reason: _ } => {},
                    libp2p::swarm::SwarmEvent::ListenerError { listener_id: _, error: _ } => {},
                    libp2p::swarm::SwarmEvent::Dialing { peer_id: _, connection_id: _ } => {},
                    libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address: _ } => {},
                    libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address: _ } => {},
                    libp2p::swarm::SwarmEvent::ExternalAddrExpired { address: _ } => {},
                    libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id: _, address: _ } => {},
                    _ => {},
                }
            },
            command = command_receiver.recv() => {
                match command {
                    Some(cmd_with_response) => {
                        handle_command::<K, V>(
                            cmd_with_response.command,
                            Some(cmd_with_response.response_sender),
                            &mut query_queue,
                            &mut database_context,
                            swarm
                        );
                    }
                    None => {
                        log::info!("Command channel closed, shutting down event loop");
                        break;
                    }
                }
            }
        }
    }
}
