use libp2p::{Swarm, noise, tcp, yamux};

use crate::{config::DefaultNetabaseConfig, network::behaviour::NetabaseBehaviour};

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

pub fn generate_swarm_with_config(
    config: &DefaultNetabaseConfig,
) -> anyhow::Result<Swarm<NetabaseBehaviour>> {
    let swarm_config = config.swarm_config();
    let behaviour_config = config.behaviour_config();

    let swarm_builder = if let Some(identity) = &swarm_config.identity() {
        libp2p::SwarmBuilder::with_existing_identity(identity.clone())
    } else {
        libp2p::SwarmBuilder::with_new_identity()
    };

    let swarm_builder = swarm_builder.with_tokio().with_tcp(
        swarm_config.tcp_config().clone(),
        noise::Config::new,
        || swarm_config.yamux_config().clone(),
    )?;

    let swarm_builder = swarm_builder.with_quic();

    let swarm = swarm_builder
        .with_behaviour(|key| {
            NetabaseBehaviour::new(key, behaviour_config)
                .expect("Failed to create NetabaseBehaviour")
        })?
        .with_swarm_config(|config| {
            let mut swarm_config_builder =
                config.with_idle_connection_timeout(swarm_config.idle_connection_timeout());

            if let Some(max_streams) = swarm_config.max_negotiating_inbound_streams() {
                swarm_config_builder =
                    swarm_config_builder.with_max_negotiating_inbound_streams(max_streams);
            }

            swarm_config_builder
        })
        .build();

    Ok(swarm)
}
