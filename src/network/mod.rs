//! Network module for Netabase
//!
//! This module provides functionality for creating and configuring libp2p swarms
//! with the Netabase behavior stack (Kademlia DHT, Identify, mDNS).
//!
//! # Functions
//!
//! - [`generate_swarm`] - Creates a swarm with test configuration
//! - [`generate_swarm_with_config`] - Creates a swarm with custom configuration

use libp2p::{Swarm, noise, tcp, yamux};

use crate::{config::NetabaseConfig, network::behaviour::NetabaseBehaviour};

pub mod behaviour;
pub mod event_loop;
pub mod event_messages;

/// Creates a libp2p swarm with default test configuration
///
/// This function creates a swarm using hardcoded defaults suitable for testing.
/// For production use or custom configuration, use [`generate_swarm_with_config`] instead.
///
/// # Arguments
///
/// * `path` - Test number used to create unique database path
///
/// # Returns
///
/// Returns a configured libp2p `Swarm<NetabaseBehaviour>` or an error if creation fails.
///
/// # Example
///
/// ```rust
/// use netabase::network::generate_swarm;
///
/// let swarm = generate_swarm(1)?;
/// ```
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

/// Creates a libp2p swarm with custom configuration
///
/// This function creates a swarm using the provided `NetabaseConfig`, allowing
/// full customization of both swarm and behavior settings.
///
/// # Arguments
///
/// * `config` - The configuration to use for swarm and behavior creation
///
/// # Returns
///
/// Returns a configured libp2p `Swarm<NetabaseBehaviour>` or an error if creation fails.
///
/// # Example
///
/// ```rust
/// use netabase::{config::NetabaseConfig, network::generate_swarm_with_config};
/// use std::time::Duration;
///
/// let config = NetabaseConfig::builder()
///     .swarm_config(
///         NetabaseSwarmConfig::builder()
///             .connection_timeout(Duration::from_secs(30))
///             .build()?
///     )
///     .behaviour_config(BehaviourConfig::default())
///     .build()?;
///
/// let swarm = generate_swarm_with_config(&config)?;
/// ```
pub fn generate_swarm_with_config(
    config: &NetabaseConfig,
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
