use std::{path::Path, time::Duration};

use libp2p::{Swarm, SwarmBuilder, futures::StreamExt, identity::Keypair, swarm::SwarmEvent};
use log::info;

use crate::network::behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent};

const BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

fn generate_swarm<P: AsRef<Path>>(storage_path: P) -> anyhow::Result<Swarm<NetabaseBehaviour>> {
    let local_key = Keypair::generate_ed25519();
    Ok(SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_dns()?
        .with_behaviour(|k| NetabaseBehaviour::new(storage_path, k).expect("Fix later"))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_mins(5)))
        .build())
}

async fn handle_event(
    swarm: &mut Swarm<NetabaseBehaviour>,
    event_sender: std::sync::mpmc::Sender<NetabaseBehaviourEvent>,
) {
    loop {
        let event = swarm.select_next_some().await;

        match event {
            libp2p::swarm::SwarmEvent::Behaviour(behaviour) => {
                event_sender.send(behaviour.clone());
                match behaviour {
                    super::behaviour::NetabaseBehaviourEvent::Kad(kad) => match kad {
                        libp2p::kad::Event::InboundRequest { request } => info!(
                            "Handling Event libp2p::kad::Event::InboundRequest: Request: {request:?}"
                        ),
                        libp2p::kad::Event::OutboundQueryProgressed {
                            id,
                            result,
                            stats,
                            step,
                        } => info!(
                            "Handling Event libp2p::kad::Event::OutboundQueryProgressed:\nID: {id:?},\nResult: {result:?}\nStats: {stats:?}\nStep: {step:?}"
                        ),
                        libp2p::kad::Event::RoutingUpdated {
                            peer,
                            is_new_peer,
                            addresses,
                            bucket_range,
                            old_peer,
                        } => info!(
                            "Handling Event libp2p::kad::Event::RoutingUpdated:\nPeer: {peer:?}\nIs New Peer: {is_new_peer}\nAddresses: {addresses:?}\nBucket Range: {bucket_range:?}\nOld Peer: {old_peer:?}"
                        ),
                        libp2p::kad::Event::UnroutablePeer { peer } => {
                            info!(
                                "Handling Event libp2p::kad::Event::UnroutablePeer: Peer: {peer:?}"
                            )
                        }
                        libp2p::kad::Event::RoutablePeer { peer, address } => {
                            info!(
                                "Handling Event libp2p::kad::Event::RoutablePeer: Peer: {peer:?}, Address: {address:?}"
                            )
                        }
                        libp2p::kad::Event::PendingRoutablePeer { peer, address } => {
                            info!(
                                "Handling Event libp2p::kad::Event::PendingRoutablePeer: Peer: {peer:?}, Address: {address:?}"
                            )
                        }
                        libp2p::kad::Event::ModeChanged { new_mode } => {
                            info!(
                                "Handling Event libp2p::kad::Event::ModeChanged: New Mode: {new_mode:?}"
                            )
                        }
                    },
                    super::behaviour::NetabaseBehaviourEvent::Identify(ident) => match ident {
                        libp2p::identify::Event::Received {
                            connection_id,
                            peer_id,
                            info,
                        } => info!(
                            "Handling Event libp2p::identify::Event::Received:\nConnection ID: {connection_id:?}\nPeer ID: {peer_id:?}\nInfo: {info:?}"
                        ),
                        libp2p::identify::Event::Sent {
                            connection_id,
                            peer_id,
                        } => info!(
                            "Handling Event libp2p::identify::Event::Sent:\nConnection ID: {connection_id:?}\nPeer ID: {peer_id:?}"
                        ),
                        libp2p::identify::Event::Pushed {
                            connection_id,
                            peer_id,
                            info,
                        } => info!(
                            "Handling Event libp2p::identify::Event::Pushed:\nConnection ID: {connection_id:?}\nPeer ID: {peer_id:?}\nInfo: {info:?}"
                        ),
                        libp2p::identify::Event::Error {
                            connection_id,
                            peer_id,
                            error,
                        } => info!(
                            "Handling Event libp2p::identify::Event::Error:\nConnection ID: {connection_id:?}\nPeer ID: {peer_id:?}\nError: {error:?}"
                        ),
                    },
                }
            }
            libp2p::swarm::SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::ConnectionEstablished:\nPeer ID: {peer_id:?}\nConnection ID: {connection_id:?}\nEndpoint: {endpoint:?}\nNum Established: {num_established}\nConcurrent Dial Errors: {concurrent_dial_errors:?}\nEstablished In: {established_in:?}"
            ),
            libp2p::swarm::SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::ConnectionClosed:\nPeer ID: {peer_id:?}\nConnection ID: {connection_id:?}\nEndpoint: {endpoint:?}\nNum Established: {num_established}\nCause: {cause:?}"
            ),
            libp2p::swarm::SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::IncomingConnection:\nConnection ID: {connection_id:?}\nLocal Address: {local_addr:?}\nSend Back Address: {send_back_addr:?}"
            ),
            libp2p::swarm::SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
                peer_id,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::IncomingConnectionError:\nConnection ID: {connection_id:?}\nLocal Address: {local_addr:?}\nSend Back Address: {send_back_addr:?}\nError: {error:?}\nPeer ID: {peer_id:?}"
            ),
            libp2p::swarm::SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::OutgoingConnectionError:\nConnection ID: {connection_id:?}\nPeer ID: {peer_id:?}\nError: {error:?}"
            ),
            libp2p::swarm::SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::NewListenAddr:\nListener ID: {listener_id:?}\nAddress: {address:?}"
            ),
            libp2p::swarm::SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::ExpiredListenAddr:\nListener ID: {listener_id:?}\nAddress: {address:?}"
            ),
            libp2p::swarm::SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::ListenerClosed:\nListener ID: {listener_id:?}\nAddresses: {addresses:?}\nReason: {reason:?}"
            ),
            libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => {
                info!(
                    "Handling Event libp2p::swarm::SwarmEvent::ListenerError:\nListener ID: {listener_id:?}\nError: {error:?}"
                )
            }
            libp2p::swarm::SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => info!(
                "Handling Event libp2p::swarm::SwarmEvent::Dialing:\nPeer ID: {peer_id:?}\nConnection ID: {connection_id:?}"
            ),
            libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => {
                info!(
                    "Handling Event libp2p::swarm::SwarmEvent::NewExternalAddrCandidate:\nAddress: {address:?}"
                )
            }
            libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => {
                info!(
                    "Handling Event libp2p::swarm::SwarmEvent::ExternalAddrConfirmed:\nAddress: {address:?}"
                )
            }
            libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => {
                info!(
                    "Handling Event libp2p::swarm::SwarmEvent::ExternalAddrExpired:\nAddress: {address:?}"
                )
            }
            libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                info!(
                    "Handling Event libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer:\nPeer ID: {peer_id:?}\nAddress: {address:?}"
                )
            }
            _ => info!("Handling Event: Unknown SwarmEvent: {event:?}"),
        }
    }
}
