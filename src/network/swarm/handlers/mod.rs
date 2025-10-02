use libp2p::{Swarm, futures::StreamExt};
use netabase_store::traits::NetabaseSchema;

use crate::network::{
    behaviour::{NetabaseBehaviour, clone_impl::NetabaseSwarmEvent},
    swarm::handlers::command_events::Command,
};
pub mod command_events;
pub mod swarm_events;

pub async fn start_swarm_loop<S: NetabaseSchema>(
    mut swarm: Swarm<NetabaseBehaviour<S>>,
    swarm_event_sender: tokio::sync::broadcast::Sender<NetabaseSwarmEvent<S>>,
    mut command_event_listener: tokio::sync::mpsc::Receiver<command_events::Command<S>>,
) {
    loop {
        tokio::select! {
            Some(command) = command_event_listener.recv() => {
                println!("Swarm Received command: {command:?}");
                handle_command_events(command);
            },
            Some(event) = swarm.next() => {
                println!("Swarm Event received: {event:?}");
                handle_swarm_events(NetabaseSwarmEvent(event));
            }

        }
    }
}

pub fn handle_swarm_events<S: NetabaseSchema>(event: NetabaseSwarmEvent<S>) {}
pub fn handle_command_events<S: NetabaseSchema>(command: Command<S>) {}
