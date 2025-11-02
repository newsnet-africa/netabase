use std::time::Duration;

use libp2p::Swarm;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) mod handlers;

// Native implementation with full networking support
#[cfg(feature = "native")]
pub(crate) fn _generate_swarm<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    backend: crate::network::config::StorageBackend,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: netabase_store::convert::ToIVec,
    D::Keys: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
{
    generate_swarm_with_name::<D>(None, backend)
}

#[cfg(feature = "native")]
pub(crate) fn generate_swarm_with_name<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    name: Option<String>,
    backend: crate::network::config::StorageBackend,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: netabase_store::convert::ToIVec,
    D::Keys: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
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
            NetabaseBehaviour::new_with_config(kp, name.clone(), backend)
                .expect("Failed to build behaviour")
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(120)))
        .build())
}

// WASM implementation with limited networking capabilities
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub fn generate_swarm<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>()
-> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: ToIVec,
    D::Keys: ToIVec,
{
    generate_swarm_with_name::<D>(None)
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub fn generate_swarm_with_name<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
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
pub(crate) async fn setup_swarm<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    mut swarm: Swarm<NetabaseBehaviour<D>>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<D>>>
where
    D: netabase_store::convert::ToIVec,
    D::Keys: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
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
pub async fn setup_swarm<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
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
