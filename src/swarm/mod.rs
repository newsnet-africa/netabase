use libp2p::{
    Swarm,
    futures::{StreamExt, channel::mpsc, select},
};
use swarm_config::swarm_init;

use crate::config::behaviour::NetabaseBehaviour;

pub mod swarm_config;

pub async fn run_swarm(
    swarm: &mut Swarm<NetabaseBehaviour>,
    command_listener: mpsc::Receiver<Command>,
) {
    loop {
        select! {
            event = swarm.select_next_some() => {
                 match event {
                    libp2p::swarm::SwarmEvent::Behaviour(_) => todo!(),
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => todo!(),
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => todo!(),
                    libp2p::swarm::SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => todo!(),
                    libp2p::swarm::SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error } => todo!(),
                    libp2p::swarm::SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => todo!(),
                    libp2p::swarm::SwarmEvent::NewListenAddr { listener_id, address } => todo!(),
                    libp2p::swarm::SwarmEvent::ExpiredListenAddr { listener_id, address } => todo!(),
                    libp2p::swarm::SwarmEvent::ListenerClosed { listener_id, addresses, reason } => todo!(),
                    libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => todo!(),
                    libp2p::swarm::SwarmEvent::Dialing { peer_id, connection_id } => todo!(),
                    libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
                    libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
                    libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => todo!(),
                    libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => todo!(),
                    _ => todo!(),
                }
            }
            command = command_listener.select_next_some() => {
            }
        }
    }
}
