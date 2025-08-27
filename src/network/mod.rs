use libp2p::{Swarm, noise, tcp, yamux};

use crate::network::behaviour::NetabaseBehaviour;

pub mod behaviour;
pub mod event_loop;
pub mod event_messages;

pub fn generate_swarm(path: usize) -> anyhow::Result<Swarm<NetabaseBehaviour>> {
    Ok(libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| NetabaseBehaviour::new_test(key, path))?
        .build())
}
