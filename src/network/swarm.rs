use std::{path::Path, time::Duration};

use libp2p::{Swarm, SwarmBuilder, futures::StreamExt, identity::Keypair};

use crate::network::behaviour::NetabaseBehaviour;

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

async fn handle_event(swarm: &mut Swarm<NetabaseBehaviour>) {
    loop {
        let event = swarm.select_next_some().await;

        match event {
            libp2p::swarm::SwarmEvent::Behaviour(behaviour) => match behaviour {
                super::behaviour::NetabaseBehaviourEvent::Kad(kad) => match kad {
                    libp2p::kad::Event::InboundRequest { request } => todo!(),
                    libp2p::kad::Event::OutboundQueryProgressed {
                        id,
                        result,
                        stats,
                        step,
                    } => todo!(),
                    libp2p::kad::Event::RoutingUpdated {
                        peer,
                        is_new_peer,
                        addresses,
                        bucket_range,
                        old_peer,
                    } => todo!(),
                    libp2p::kad::Event::UnroutablePeer { peer } => todo!(),
                    libp2p::kad::Event::RoutablePeer { peer, address } => todo!(),
                    libp2p::kad::Event::PendingRoutablePeer { peer, address } => todo!(),
                    libp2p::kad::Event::ModeChanged { new_mode } => todo!(),
                },
                super::behaviour::NetabaseBehaviourEvent::Identify(ident) => match ident {
                    libp2p::identify::Event::Received {
                        connection_id,
                        peer_id,
                        info,
                    } => todo!(),
                    libp2p::identify::Event::Sent {
                        connection_id,
                        peer_id,
                    } => todo!(),
                    libp2p::identify::Event::Pushed {
                        connection_id,
                        peer_id,
                        info,
                    } => todo!(),
                    libp2p::identify::Event::Error {
                        connection_id,
                        peer_id,
                        error,
                    } => todo!(),
                },
            },
            libp2p::swarm::SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => todo!(),
            libp2p::swarm::SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => todo!(),
            libp2p::swarm::SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => todo!(),
            libp2p::swarm::SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
                peer_id,
            } => todo!(),
            libp2p::swarm::SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => todo!(),
            libp2p::swarm::SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => todo!(),
            libp2p::swarm::SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => todo!(),
            libp2p::swarm::SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => todo!(),
            libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => todo!(),
            libp2p::swarm::SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => todo!(),
            libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
            libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
            libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => todo!(),
            libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => todo!(),
            _ => todo!(),
        }
    }
}
