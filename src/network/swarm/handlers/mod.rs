use netabase_store::traits::NetabaseSchema;

#[cfg(feature = "native")]
use libp2p::{Swarm, futures::StreamExt};

#[cfg(feature = "native")]
use crate::network::{
    behaviour::{NetabaseBehaviour, clone_impl::NetabaseSwarmEvent},
    swarm::handlers::command_events::{Command, handle_command_events},
};

#[cfg(feature = "native")]
pub mod command_events;
#[cfg(feature = "native")]
pub mod swarm_events;

// Native implementation with full swarm event loop
#[cfg(feature = "native")]
pub async fn start_swarm_loop<S: NetabaseSchema>(
    mut swarm: Swarm<NetabaseBehaviour<S>>,
    swarm_event_sender: tokio::sync::broadcast::Sender<NetabaseSwarmEvent<S>>,
    mut command_event_listener: tokio::sync::mpsc::Receiver<command_events::Command<S>>,
) {
    loop {
        tokio::select! {
            Some(command) = command_event_listener.recv() => {
                println!("Swarm Received command: {command:?}");
                handle_command_events(&mut swarm, command);
            },
            Some(event) = swarm.next() => {
                println!("Swarm Event received: {event:?}");
                let event = NetabaseSwarmEvent(event);
                let result = swarm_event_sender.send(event.clone());
                println!("Sending Event: {result:?}");
                swarm_events::handle_swarm_events(event);
            }
        }
    }
}

// WASM placeholder - networking not yet implemented
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub async fn start_swarm_loop<S: NetabaseSchema>(
    _swarm: (),
    _swarm_event_sender: (),
    _command_event_listener: (),
) {
    // WASM swarm event loop would be implemented here
    // For WebSocket/WebRTC based networking
    panic!("WASM networking is not yet implemented");
}
