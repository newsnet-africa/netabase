use std::time::Duration;

use libp2p::{Swarm, noise, tcp, yamux};

use crate::config::behaviour::NetabaseBehaviour;

pub fn swarm_init() -> Result<Swarm<NetabaseBehaviour>, Box<dyn std::error::Error>> {
    Ok(libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(NetabaseBehaviour::new)?
        .with_swarm_config(|conf| conf.with_idle_connection_timeout(Duration::from_secs(500)))
        .build())
}
