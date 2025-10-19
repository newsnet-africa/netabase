use libp2p::Multiaddr;
use libp2p::TransportError;

use libp2p::swarm;
use libp2p::swarm::ConnectionDenied;
use libp2p::swarm::ConnectionError;
use libp2p::swarm::SwarmEvent;

use netabase_store::traits::definition::NetabaseDefinitionTrait;

use crate::network::behaviour::NetabaseBehaviourEvent;

impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> Clone for NetabaseBehaviourEvent<D>
where
    <D as netabase_store::traits::definition::NetabaseDefinitionTrait>::Discriminants:
        std::marker::Send,
{
    fn clone(&self) -> Self {
        match self {
            NetabaseBehaviourEvent::Kad(kad_event) => {
                NetabaseBehaviourEvent::Kad(kad_event.clone())
            }
            NetabaseBehaviourEvent::Identify(identify_event) => match identify_event {
                libp2p::identify::Event::Received {
                    connection_id,
                    peer_id,
                    info,
                } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Received {
                    connection_id: *connection_id,
                    peer_id: *peer_id,
                    info: info.clone(),
                }),
                libp2p::identify::Event::Sent {
                    connection_id,
                    peer_id,
                } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent {
                    connection_id: *connection_id,
                    peer_id: *peer_id,
                }),
                libp2p::identify::Event::Pushed {
                    connection_id,
                    peer_id,
                    info,
                } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Pushed {
                    connection_id: *connection_id,
                    peer_id: *peer_id,
                    info: info.clone(),
                }),
                libp2p::identify::Event::Error {
                    connection_id,
                    peer_id,
                    error: _,
                } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Error {
                    connection_id: *connection_id,
                    peer_id: *peer_id,
                    error: libp2p::swarm::StreamUpgradeError::Apply(
                        libp2p::identify::UpgradeError::StreamClosed,
                    ),
                }),
            },
            #[cfg(feature = "native")]
            NetabaseBehaviourEvent::Mdns(mdns) => NetabaseBehaviourEvent::Mdns(mdns.clone()),
            NetabaseBehaviourEvent::ConnectionLimit(i) => {
                NetabaseBehaviourEvent::ConnectionLimit(*i)
            }
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct NetabaseSwarmEvent<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    pub SwarmEvent<NetabaseBehaviourEvent<D>>,
);

fn multiaddr_cloner(multi: &Multiaddr) -> Multiaddr {
    Multiaddr::try_from(multi.to_vec()).expect("MultiAddr clone error")
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
    vect.iter()
        .map(|(multi, tre)| {
            (
                multiaddr_cloner(multi),
                match tre {
                    TransportError::MultiaddrNotSupported(multiaddr) => {
                        TransportError::MultiaddrNotSupported(multiaddr_cloner(multiaddr))
                    }
                    TransportError::Other(o) => {
                        TransportError::Other(std::io::Error::from(o.kind()))
                    }
                },
            )
        })
        .collect()
}

impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> Clone for NetabaseSwarmEvent<D> {
    fn clone(&self) -> Self {
        match &self.0 {
            SwarmEvent::Behaviour(nbe) => NetabaseSwarmEvent(SwarmEvent::Behaviour(nbe.clone())),
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => NetabaseSwarmEvent(SwarmEvent::ConnectionEstablished {
                peer_id: *peer_id,
                connection_id: *connection_id,
                endpoint: endpoint.clone(),
                num_established: *num_established,
                concurrent_dial_errors: concurrent_dial_errors
                    .as_ref()
                    .map(multi_trans_error_cloner),
                established_in: *established_in,
            }),
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => NetabaseSwarmEvent(SwarmEvent::ConnectionClosed {
                peer_id: *peer_id,
                connection_id: *connection_id,
                endpoint: endpoint.clone(),
                num_established: *num_established,
                cause: cause.as_ref().map(|c| cause_cloner(c)),
            }),
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => NetabaseSwarmEvent(SwarmEvent::IncomingConnection {
                connection_id: *connection_id,
                local_addr: multiaddr_cloner(&local_addr),
                send_back_addr: multiaddr_cloner(&send_back_addr),
            }),
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
                peer_id,
            } => NetabaseSwarmEvent(SwarmEvent::IncomingConnectionError {
                connection_id: *connection_id,
                local_addr: multiaddr_cloner(&local_addr),
                send_back_addr: multiaddr_cloner(&send_back_addr),
                error: match error {
                    swarm::ListenError::Aborted => swarm::ListenError::Aborted,
                    swarm::ListenError::WrongPeerId { obtained, endpoint } => {
                        swarm::ListenError::WrongPeerId {
                            obtained: *obtained,
                            endpoint: endpoint.clone(),
                        }
                    }
                    swarm::ListenError::LocalPeerId { address } => {
                        swarm::ListenError::LocalPeerId {
                            address: multiaddr_cloner(address),
                        }
                    }
                    swarm::ListenError::Denied { cause: _ } => swarm::ListenError::Denied {
                        cause: ConnectionDenied::new(std::io::Error::new(
                            std::io::ErrorKind::ConnectionRefused,
                            "Connection denied",
                        )),
                    },
                    swarm::ListenError::Transport(transport_error) => {
                        swarm::ListenError::Transport(match transport_error {
                            TransportError::MultiaddrNotSupported(multiaddr) => {
                                TransportError::MultiaddrNotSupported(multiaddr_cloner(multiaddr))
                            }
                            TransportError::Other(e) => {
                                TransportError::Other(std::io::Error::from(e.kind()))
                            }
                        })
                    }
                },
                peer_id: *peer_id,
            }),
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => NetabaseSwarmEvent(SwarmEvent::OutgoingConnectionError {
                connection_id: *connection_id,
                peer_id: *peer_id,
                error: match error {
                    swarm::DialError::LocalPeerId { address } => swarm::DialError::LocalPeerId {
                        address: multiaddr_cloner(address),
                    },
                    swarm::DialError::NoAddresses => swarm::DialError::NoAddresses,
                    swarm::DialError::DialPeerConditionFalse(peer_condition) => {
                        swarm::DialError::DialPeerConditionFalse(*peer_condition)
                    }
                    swarm::DialError::Aborted => swarm::DialError::Aborted,
                    swarm::DialError::WrongPeerId { obtained, address } => {
                        swarm::DialError::WrongPeerId {
                            obtained: *obtained,
                            address: multiaddr_cloner(address),
                        }
                    }
                    swarm::DialError::Denied { cause: _ } => swarm::DialError::Denied {
                        cause: ConnectionDenied::new(std::io::Error::new(
                            std::io::ErrorKind::ConnectionRefused,
                            "Connection denied",
                        )),
                    },
                    swarm::DialError::Transport(items) => {
                        swarm::DialError::Transport(multi_trans_error_cloner(items))
                    }
                },
            }),
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => NetabaseSwarmEvent(SwarmEvent::NewListenAddr {
                listener_id: *listener_id,
                address: multiaddr_cloner(address),
            }),
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => NetabaseSwarmEvent(SwarmEvent::ExpiredListenAddr {
                listener_id: *listener_id,
                address: multiaddr_cloner(address),
            }),
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => NetabaseSwarmEvent(SwarmEvent::ListenerClosed {
                listener_id: *listener_id,
                addresses: addresses
                    .iter()
                    .map(|addr| multiaddr_cloner(addr))
                    .collect(),
                reason: match reason {
                    Ok(_) => Ok(()),
                    Err(r) => Err(std::io::Error::from(r.kind())),
                },
            }),
            SwarmEvent::ListenerError { listener_id, error } => {
                NetabaseSwarmEvent(SwarmEvent::ListenerError {
                    listener_id: *listener_id,
                    error: std::io::Error::from(error.kind()),
                })
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => NetabaseSwarmEvent(SwarmEvent::Dialing {
                peer_id: *peer_id,
                connection_id: *connection_id,
            }),
            SwarmEvent::NewExternalAddrCandidate { address } => {
                NetabaseSwarmEvent(SwarmEvent::NewExternalAddrCandidate {
                    address: multiaddr_cloner(address),
                })
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                NetabaseSwarmEvent(SwarmEvent::ExternalAddrConfirmed {
                    address: multiaddr_cloner(address),
                })
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                NetabaseSwarmEvent(SwarmEvent::ExternalAddrExpired {
                    address: multiaddr_cloner(address),
                })
            }
            SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                NetabaseSwarmEvent(SwarmEvent::NewExternalAddrOfPeer {
                    peer_id: *peer_id,
                    address: multiaddr_cloner(address),
                })
            }
            _ => {
                panic!("Not handled yet");
            }
        }
    }
}
