use netabase_store::traits::definition::NetabaseDefinitionTrait;

#[cfg(feature = "native")]
use libp2p::{Swarm, futures::StreamExt};

#[cfg(feature = "native")]
use crate::network::{
    behaviour::{NetabaseBehaviour, clone_impl::NetabaseSwarmEvent},
    swarm::handlers::command_events::handle_command_events,
};

#[cfg(feature = "native")]
pub(crate) mod command_events;
#[cfg(feature = "native")]
pub(crate) mod swarm_events;

// Native implementation with full swarm event loop
#[cfg(feature = "native")]
pub(crate) async fn start_swarm_loop<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    config: crate::network::config::NetabaseConfig,
    mut swarm: Swarm<NetabaseBehaviour<D>>,
    swarm_event_sender: tokio::sync::broadcast::Sender<NetabaseSwarmEvent<D>>,
    mut command_event_listener: tokio::sync::mpsc::Receiver<command_events::Command<D>>,
) where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
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
    loop {
        tokio::select! {
            Some(command) = command_event_listener.recv() => {
                handle_command_events(&mut swarm, command);
            },
            Some(event) = swarm.next() => {
                let event = NetabaseSwarmEvent(event);
                let _ = swarm_event_sender.send(event.clone());
                swarm_events::handle_swarm_events(config.clone(), &mut swarm, event);

                // TODO: Poll outgoing Paxos queue after handling events
                // This will be implemented when we add the request-response protocol
            },
        }
    }
}

// WASM placeholder - networking not yet implemented
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub(crate) async fn start_swarm_loop<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    _swarm: (),
    _swarm_event_sender: (),
    _command_event_listener: (),
) {
    // WASM swarm event loop would be implemented here
    // For WebSocket/WebRTC based networking
    panic!("WASM networking is not yet implemented");
}
