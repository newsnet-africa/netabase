use anyhow::anyhow;
use std::path::Path;

use libp2p::{
    identity::Keypair,
    swarm::{ConnectionDenied, ConnectionError, NetworkBehaviour, SwarmEvent},
    *,
};

use crate::database::SledStore;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub kad: kad::Behaviour<SledStore>,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

impl Clone for NetabaseBehaviourEvent {
    fn clone(&self) -> Self {
        match self {
            NetabaseBehaviourEvent::Kad(kad_event) => {
                // kad events implement Clone
                NetabaseBehaviourEvent::Kad(kad_event.clone())
            }
            NetabaseBehaviourEvent::Identify(identify_event) => {
                // Handle each identify event variant manually
                match identify_event {
                    libp2p::identify::Event::Received {
                        connection_id,
                        peer_id,
                        info,
                    } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Received {
                        connection_id: connection_id.clone(),
                        peer_id: peer_id.clone(),
                        info: info.clone(),
                    }),
                    libp2p::identify::Event::Sent {
                        connection_id,
                        peer_id,
                    } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent {
                        connection_id: connection_id.clone(),
                        peer_id: peer_id.clone(),
                    }),
                    libp2p::identify::Event::Pushed {
                        connection_id,
                        peer_id,
                        info,
                    } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Pushed {
                        connection_id: connection_id.clone(),
                        peer_id: peer_id.clone(),
                        info: info.clone(),
                    }),
                    libp2p::identify::Event::Error {
                        connection_id,
                        peer_id,
                        error: _,
                    } => {
                        // For the Error case, we create a placeholder error since the original is not cloneable
                        NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Error {
                            connection_id: connection_id.clone(),
                            peer_id: peer_id.clone(),
                            error: libp2p::swarm::StreamUpgradeError::Apply(
                                libp2p::identify::UpgradeError::StreamClosed,
                            ),
                        })
                    }
                }
            }
            NetabaseBehaviourEvent::Mdns(mdns) => NetabaseBehaviourEvent::Mdns(mdns.clone()),
        }
    }
}

#[repr(transparent)]
pub struct NetabaseEvent(pub SwarmEvent<NetabaseBehaviourEvent>);

fn multiaddr_cloner(multi: &Multiaddr) -> Multiaddr {
    Multiaddr::try_from(multi.to_vec()).expect("MultiAddre clone error")
}
fn cause_cloner(cause: &ConnectionError) -> ConnectionError {
    match cause {
        swarm::ConnectionError::IO(error) => {
            swarm::ConnectionError::IO(std::io::Error::from(error.kind()))
        }
        swarm::ConnectionError::KeepAliveTimeout => swarm::ConnectionError::KeepAliveTimeout,
    }
}
fn multi_trans_error_cloner(
    vect: &Vec<(Multiaddr, TransportError<std::io::Error>)>,
) -> Vec<(Multiaddr, TransportError<std::io::Error>)> {
    let mut new_vec = vec![];
    vect.iter().for_each(|(multi, tre)| {
        new_vec.push((
            Multiaddr::try_from(multi.to_vec()).expect("MultiAddre clone error"),
            {
                match tre {
                    TransportError::MultiaddrNotSupported(multiaddr) => {
                        TransportError::MultiaddrNotSupported(multiaddr_cloner(multiaddr))
                    }
                    TransportError::Other(o) => {
                        TransportError::Other(std::io::Error::from(o.kind()))
                    }
                }
            },
        ))
    });
    new_vec
}
impl Clone for NetabaseEvent {
    fn clone(&self) -> Self {
        match &self.0 {
            SwarmEvent::Behaviour(nbe) => NetabaseEvent(SwarmEvent::Behaviour(nbe.clone())),
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => NetabaseEvent(SwarmEvent::ConnectionEstablished {
                peer_id: peer_id.clone(),
                connection_id: connection_id.clone(),
                endpoint: endpoint.clone(),
                num_established: num_established.clone(),
                concurrent_dial_errors: {
                    if let Some(vect) = concurrent_dial_errors {
                        Some(multi_trans_error_cloner(vect))
                    } else {
                        None
                    }
                },
                established_in: established_in.clone(),
            }),
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => NetabaseEvent(SwarmEvent::ConnectionClosed {
                peer_id: peer_id.clone(),
                connection_id: connection_id.clone(),
                endpoint: endpoint.clone(),
                num_established: num_established.clone(),
                cause: {
                    if let Some(c) = cause {
                        Some(cause_cloner(c))
                    } else {
                        None
                    }
                },
            }),
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => NetabaseEvent(SwarmEvent::IncomingConnection {
                connection_id: connection_id.clone(),
                local_addr: multiaddr_cloner(local_addr),
                send_back_addr: multiaddr_cloner(send_back_addr),
            }),
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
                peer_id,
            } => NetabaseEvent(SwarmEvent::IncomingConnectionError {
                connection_id: connection_id.clone(),
                local_addr: multiaddr_cloner(local_addr),
                send_back_addr: multiaddr_cloner(send_back_addr),
                error: {
                    match error {
                        swarm::ListenError::Aborted => swarm::ListenError::Aborted,
                        swarm::ListenError::WrongPeerId { obtained, endpoint } => {
                            swarm::ListenError::WrongPeerId {
                                obtained: obtained.clone(),
                                endpoint: endpoint.clone(),
                            }
                        }
                        swarm::ListenError::LocalPeerId { address } => {
                            swarm::ListenError::LocalPeerId {
                                address: multiaddr_cloner(address),
                            }
                        }
                        swarm::ListenError::Denied { cause } => swarm::ListenError::Denied {
                            cause: ConnectionDenied::new(cause.clone()),
                        },
                        swarm::ListenError::Transport(transport_error) => {
                            swarm::ListenError::Transport({
                                match transport_error {
                                    TransportError::MultiaddrNotSupported(multiaddr) => {
                                        TransportError::MultiaddrNotSupported(multiaddr_cloner(
                                            multiaddr,
                                        ))
                                    }
                                    TransportError::Other(e) => {
                                        TransportError::Other(std::io::Error::from(e.kind()))
                                    }
                                }
                            })
                        }
                    }
                },
                peer_id: peer_id.clone(),
            }),
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => NetabaseEvent(SwarmEvent::OutgoingConnectionError {
                connection_id: connection_id.clone(),
                peer_id: peer_id.clone(),
                error: {
                    match error {
                        swarm::DialError::LocalPeerId { address } => {
                            swarm::DialError::LocalPeerId {
                                address: multiaddr_cloner(address),
                            }
                        }
                        swarm::DialError::NoAddresses => swarm::DialError::NoAddresses,
                        swarm::DialError::DialPeerConditionFalse(peer_condition) => {
                            swarm::DialError::DialPeerConditionFalse(peer_condition.clone())
                        }
                        swarm::DialError::Aborted => swarm::DialError::NoAddresses,
                        swarm::DialError::WrongPeerId { obtained, address } => {
                            swarm::DialError::WrongPeerId {
                                obtained: obtained.clone(),
                                address: multiaddr_cloner(address),
                            }
                        }
                        swarm::DialError::Denied { cause } => swarm::DialError::Denied {
                            cause: swarm::ConnectionDenied::new(cause.clone()),
                        },
                        swarm::DialError::Transport(items) => {
                            swarm::DialError::Transport(multi_trans_error_cloner(items))
                        }
                    }
                },
            }),
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => NetabaseEvent(SwarmEvent::NewListenAddr {
                listener_id: listener_id.clone(),
                address: multiaddr_cloner(address),
            }),
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => NetabaseEvent(SwarmEvent::ExpiredListenAddr {
                listener_id: listener_id.clone(),
                address: multiaddr_cloner(address),
            }),
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => NetabaseEvent(SwarmEvent::ListenerClosed {
                listener_id: listener_id.clone(),
                addresses: {
                    let mut new_vec = vec![];
                    addresses
                        .iter()
                        .for_each(|i| new_vec.push(multiaddr_cloner(i)));
                    new_vec
                },
                reason: {
                    match reason {
                        Ok(r) => Ok(()),
                        Err(r) => Err(std::io::Error::from(r.kind())),
                    }
                },
            }),
            SwarmEvent::ListenerError { listener_id, error } => {
                NetabaseEvent(SwarmEvent::ListenerError {
                    listener_id: listener_id.clone(),
                    error: std::io::Error::from(error.kind()),
                })
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => NetabaseEvent(SwarmEvent::Dialing {
                peer_id: peer_id.clone(),
                connection_id: connection_id.clone(),
            }),
            SwarmEvent::NewExternalAddrCandidate { address } => {
                NetabaseEvent(SwarmEvent::NewExternalAddrCandidate {
                    address: multiaddr_cloner(address),
                })
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                NetabaseEvent(SwarmEvent::NewExternalAddrCandidate {
                    address: multiaddr_cloner(address),
                })
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                NetabaseEvent(SwarmEvent::ExternalAddrExpired {
                    address: multiaddr_cloner(address),
                })
            }
            SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                NetabaseEvent(SwarmEvent::NewExternalAddrOfPeer {
                    peer_id: peer_id.clone(),
                    address: multiaddr_cloner(address),
                })
            }
            _ => panic!("Not handled yet"),
        }
    }
}

impl NetabaseBehaviour {
    pub fn new<P: AsRef<Path>>(storage_path: P, keypair: &Keypair) -> anyhow::Result<Self> {
        let local_peer_id = keypair.public().to_peer_id();
        let protocol = "/NetabaseDummy".to_string(); //TODO: Obviously check to see if the protocol works the same everywhere and if I can use the same protocol for everything.
        let kad = kad::Behaviour::new(local_peer_id, SledStore::new(local_peer_id, storage_path)?);
        let identify = identify::Behaviour::new(identify::Config::new(protocol, keypair.public()));
        let mdns = mdns::Behaviour::new(mdns::Config::default(), local_peer_id)?;
        Ok(Self {
            kad,
            identify,
            mdns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logging;
    use libp2p::identity::Keypair;

    #[test]
    fn test_netabase_behaviour_event_clone() {
        init_logging();
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();
        let connection_id = libp2p::swarm::ConnectionId::new_unchecked(1);

        // Test that we can clone the NetabaseBehaviourEvent with Identify variant
        let identify_event = NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent {
            connection_id,
            peer_id,
        });

        let cloned = identify_event.clone();
        assert!(matches!(
            cloned,
            NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent { .. })
        ));
    }
}
