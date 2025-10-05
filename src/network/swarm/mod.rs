use std::time::Duration;

use libp2p::Swarm;
use netabase_store::traits::NetabaseSchema;

use crate::network::{
    behaviour::{NetabaseBehaviour, clone_impl::NetabaseSwarmEvent},
    swarm::handlers::{command_events::Command, start_swarm_loop},
};

pub mod handlers;

// Native implementation with full networking support
#[cfg(feature = "native")]
pub fn generate_swarm<S: NetabaseSchema>() -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    generate_swarm_with_name::<S>(None)
}

#[cfg(feature = "native")]
pub fn generate_swarm_with_name<S: NetabaseSchema>(
    name: Option<String>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    use libp2p::{SwarmBuilder, tcp};

    Ok(SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|kp| {
            NetabaseBehaviour::new_with_name(kp, name.clone()).expect("Failed to build behaviour")
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(120)))
        .build())
}

// WASM implementation with limited networking capabilities
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub fn generate_swarm<S: NetabaseSchema>() -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    generate_swarm_with_name::<S>(None)
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub fn generate_swarm_with_name<S: NetabaseSchema>(
    _name: Option<String>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    // WASM implementation placeholder
    // In a real WASM environment, this would:
    // 1. Use websocket-websys for WebSocket connections
    // 2. Use webrtc-websys for direct peer connections
    // 3. Connect to relay/bootstrap nodes via WebSocket
    // 4. Handle signaling for WebRTC peer discovery

    // For now, return an error indicating WASM networking is not yet implemented
    Err(anyhow::anyhow!(
        "WASM networking is not yet implemented. Use netabase_store directly for local operations."
    ))
}

// Native swarm setup with listening capabilities
#[cfg(feature = "native")]
pub async fn setup_swarm<S: NetabaseSchema>(
    mut swarm: Swarm<NetabaseBehaviour<S>>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    Ok(swarm)
}

// WASM swarm setup - placeholder for future implementation
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub async fn setup_swarm<S: NetabaseSchema>(
    _swarm: Swarm<NetabaseBehaviour<S>>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    // WASM networking setup would go here
    // - Connect to WebSocket relay nodes
    // - Setup WebRTC signaling
    // - Configure DHT for browser environment

    Err(anyhow::anyhow!("WASM swarm setup is not yet implemented"))
}
