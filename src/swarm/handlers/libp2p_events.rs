use libp2p::{Swarm, swarm::SwarmEvent};
use net_event::net_event_handler;
use tokio::sync::broadcast;

use crate::config::behaviour::{NetEvent, NetabaseBehaviour};

pub fn handle_libp2p_events(
    swarm: &mut Swarm<NetabaseBehaviour>,
    event: SwarmEvent<NetEvent>,
    sender: &mut broadcast::Sender<NetEvent>,
) {
    // println!("{event:?}");
    match event {
        SwarmEvent::Behaviour(net_event) => {
            sender
                .send(net_event.clone())
                .expect("Failed to Send event");
            net_event_handler(swarm, net_event);
        }
        // SwarmEvent::ConnectionEstablished {
        //     peer_id,
        //     connection_id,
        //     endpoint,
        //     num_established,
        //     concurrent_dial_errors,
        //     established_in,
        // } => todo!(),
        // SwarmEvent::ConnectionClosed {
        //     peer_id,
        //     connection_id,
        //     endpoint,
        //     num_established,
        //     cause,
        // } => todo!(),
        // SwarmEvent::IncomingConnection {
        //     connection_id,
        //     local_addr,
        //     send_back_addr,
        // } => todo!(),
        // SwarmEvent::IncomingConnectionError {
        //     connection_id,
        //     local_addr,
        //     send_back_addr,
        //     error,
        // } => todo!(),
        // SwarmEvent::OutgoingConnectionError {
        //     connection_id,
        //     peer_id,
        //     error,
        // } => todo!(),
        // SwarmEvent::NewListenAddr {
        //     listener_id,
        //     address,
        // } => todo!(),
        // SwarmEvent::ExpiredListenAddr {
        //     listener_id,
        //     address,
        // } => todo!(),
        // SwarmEvent::ListenerClosed {
        //     listener_id,
        //     addresses,
        //     reason,
        // } => todo!(),
        // SwarmEvent::ListenerError { listener_id, error } => todo!(),
        // SwarmEvent::Dialing {
        //     peer_id,
        //     connection_id,
        // } => todo!(),
        // SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
        // SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
        // SwarmEvent::ExternalAddrExpired { address } => todo!(),
        // SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => todo!(),
        _ => {}
    }
}

mod net_event {
    use libp2p::{Swarm, kad, mdns};

    use crate::config::behaviour::{NetEvent, NetabaseBehaviour};

    pub fn net_event_handler(swarm: &mut Swarm<NetabaseBehaviour>, event: NetEvent) {
        match event {
            NetEvent::Kad(event) => {
                kad_event_handler(event);
            }
            NetEvent::Mdns(event) => {
                mdns_event_handler(swarm, event);
            }
            NetEvent::Ping {
                peer_id,
                connection,
                result,
            } => {}
        }
    }

    fn kad_event_handler(event: kad::Event) {
        // println!("{event:?}");
        // match event {
        //     kad::Event::InboundRequest { request } => todo!(),
        //     kad::Event::OutboundQueryProgressed {
        //         id,
        //         result,
        //         stats,
        //         step,
        //     } => todo!(),
        //     kad::Event::RoutingUpdated {
        //         peer,
        //         is_new_peer,
        //         addresses,
        //         bucket_range,
        //         old_peer,
        //     } => todo!(),
        //     kad::Event::UnroutablePeer { peer } => todo!(),
        //     kad::Event::RoutablePeer { peer, address } => todo!(),
        //     kad::Event::PendingRoutablePeer { peer, address } => todo!(),
        //     kad::Event::ModeChanged { new_mode } => todo!(),
        // }
    }

    fn mdns_event_handler(swarm: &mut Swarm<NetabaseBehaviour>, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(items) => {
                for (peer_id, multiaddress) in items {
                    swarm
                        .dial(peer_id)
                        .inspect_err(|_| {
                            swarm.dial(multiaddress.clone()).expect("Dial error");
                        })
                        .expect("Dial Error");

                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multiaddress.clone());
                }
            }
            mdns::Event::Expired(items) => {}
        }
    }
}
