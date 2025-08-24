use std::collections::HashMap;

use libp2p::{
    Multiaddr, PeerId, Swarm,
    futures::{SinkExt, StreamExt},
    kad::{GetRecordOk, PutRecordOk, QueryId, Quorum},
    swarm::SwarmEvent,
};
use tracing::Instrument;

use crate::{
    DatabaseCommand, NetabaseCommand, init_logging,
    network::{
        behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent, NetabaseEvent},
        event_handlers::behaviour_events::handle_behaviour_events,
    },
};

mod behaviour_events;
mod command_events;
mod connection_events;

// Struct to track pending queries and their response channels
pub(super) struct PendingQueries {
    put_queries: HashMap<QueryId, tokio::sync::oneshot::Sender<anyhow::Result<PutRecordOk>>>,
    get_queries: HashMap<QueryId, tokio::sync::oneshot::Sender<anyhow::Result<GetRecordOk>>>,
}

impl PendingQueries {
    pub(super) fn new() -> Self {
        Self {
            put_queries: HashMap::new(),
            get_queries: HashMap::new(),
        }
    }

    pub(super) fn insert_put_query(
        &mut self,
        id: QueryId,
        sender: tokio::sync::oneshot::Sender<anyhow::Result<PutRecordOk>>,
    ) {
        self.put_queries.insert(id, sender);
    }

    pub(super) fn insert_get_query(
        &mut self,
        id: QueryId,
        sender: tokio::sync::oneshot::Sender<anyhow::Result<GetRecordOk>>,
    ) {
        self.get_queries.insert(id, sender);
    }

    pub(super) fn complete_put_query(&mut self, id: &QueryId, result: anyhow::Result<PutRecordOk>) {
        if let Some(sender) = self.put_queries.remove(id) {
            let _ = sender.send(result);
        }
    }

    pub(super) fn complete_get_query(&mut self, id: &QueryId, result: anyhow::Result<GetRecordOk>) {
        if let Some(sender) = self.get_queries.remove(id) {
            let _ = sender.send(result);
        }
    }
}

pub async fn handle_events(
    mut swarm: Swarm<NetabaseBehaviour>,
    event_sender: tokio::sync::broadcast::Sender<NetabaseEvent>,
    mut netabase_command_receiver: tokio::sync::mpsc::Receiver<NetabaseCommand>,
    listen_addresses: Vec<Multiaddr>,
) {
    init_logging();

    // Track pending queries
    let mut pending_queries = PendingQueries::new();

    // Start listening on configured addresses
    println!("Starting listen:");
    for multi in listen_addresses {
        let listen_res = swarm.listen_on(multi);
        println!("Listen Result: {listen_res:?}");
    }

    loop {
        eprintln!("Inner Loop");
        eprintln!("Swarm: {:?}", swarm.network_info());

        tokio::select! {
            event = swarm.select_next_some() => {
                let wrapped_event = NetabaseEvent(event);
                eprintln!("Sending Event: {:?}", wrapped_event);
                let send_event_result = event_sender.send(wrapped_event.clone());
                eprintln!("Sent: {send_event_result:?}");

                // Handle event types through appropriate handlers
                match wrapped_event.0 {
                    SwarmEvent::Behaviour(behaviour_event) => {
                        handle_behaviour_events(behaviour_event, &mut swarm, &mut pending_queries);
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
                    SwarmEvent::NewExternalAddrOfPeer { peer_id, address: _ } => {},
                    _ => {},
                };
            },
            wrapped_command = netabase_command_receiver.recv() => {
                match wrapped_command {
                    Some(command) => match command {
                        NetabaseCommand::Close => {
                            eprintln!("Closing");
                            break
                        },
                        NetabaseCommand::Database(db_command) => {
                            handle_database_command(db_command, &mut swarm, &mut pending_queries).await;
                        },
                        NetabaseCommand::GetListeners(response_tx) => {
                            let listeners: Vec<Multiaddr> = swarm.listeners().cloned().collect();
                            let _ = response_tx.send(listeners);
                        }
                    },
                    None => {
                        eprintln!("Command receiver closed");
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_database_command(
    command: DatabaseCommand,
    swarm: &mut Swarm<NetabaseBehaviour>,
    pending_queries: &mut PendingQueries,
) {
    match command {
        DatabaseCommand::Put {
            record,
            put_to,
            quorum,
            response_tx,
        } => {
            let query_id = match put_to {
                None => match swarm.behaviour_mut().kad.put_record(record, quorum) {
                    Ok(id) => id,
                    Err(e) => {
                        let _ = response_tx.send(Err(anyhow::anyhow!(e)));
                        return;
                    }
                },
                Some(peers) => {
                    swarm
                        .behaviour_mut()
                        .kad
                        .put_record_to(record, peers.into_iter(), quorum)
                }
            };

            pending_queries.insert_put_query(query_id, response_tx);
        }
        DatabaseCommand::Get { key, response_tx } => {
            let query_id = swarm.behaviour_mut().kad.get_record(key);
            pending_queries.insert_get_query(query_id, response_tx);
        }
    }
}
