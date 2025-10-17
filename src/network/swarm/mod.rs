use std::time::Duration;

use libp2p::Swarm;
use netabase_store::traits::{definition::NetabaseDefinition, convert::ToIVec};

use crate::network::behaviour::NetabaseBehaviour;

pub mod handlers;

// Native implementation with full networking support
#[cfg(feature = "native")]
pub fn generate_swarm<D: NetabaseDefinition + Send + Sync + 'static>() -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
    generate_swarm_with_name::<D>(None)
}

#[cfg(feature = "native")]
pub fn generate_swarm_with_name<D: NetabaseDefinition + Send + Sync + 'static>(
    name: Option<String>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
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
pub fn generate_swarm<D: NetabaseDefinition + Send + Sync + 'static>() -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
    generate_swarm_with_name::<D>(None)
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub fn generate_swarm_with_name<D: NetabaseDefinition + Send + Sync + 'static>(
    _name: Option<String>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
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
pub async fn setup_swarm<D: NetabaseDefinition + Send + Sync + 'static>(
    mut swarm: Swarm<NetabaseBehaviour<D>>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    Ok(swarm)
}

// WASM swarm setup - placeholder for future implementation
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub async fn setup_swarm<D: NetabaseDefinition + Send + Sync + 'static>(
    _swarm: Swarm<NetabaseBehaviour<D>>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
    // WASM networking setup would go here
    // - Connect to WebSocket relay nodes
    // - Setup WebRTC signaling
    // - Configure DHT for browser environment

    Err(anyhow::anyhow!("WASM swarm setup is not yet implemented"))
}
