use netabase_store::traits::definition::NetabaseDefinition;

#[cfg(feature = "native")]
use libp2p::{Swarm, futures::StreamExt};

#[cfg(feature = "native")]
use crate::network::{
    behaviour::{NetabaseBehaviour, clone_impl::NetabaseSwarmEvent},
    swarm::handlers::command_events::handle_command_events,
};

#[cfg(feature = "native")]
pub mod command_events;
#[cfg(feature = "native")]
pub mod swarm_events;

// Native implementation with full swarm event loop
#[cfg(feature = "native")]
pub(crate) async fn start_swarm_loop<D: NetabaseDefinition + Send + Sync + 'static>(
    mut swarm: Swarm<NetabaseBehaviour<D>>,
    swarm_event_sender: tokio::sync::broadcast::Sender<NetabaseSwarmEvent<D>>,
    mut command_event_listener: tokio::sync::mpsc::Receiver<command_events::Command<D>>,
)
where
    D: netabase_store::traits::convert::ToIVec,
{
    loop {
        tokio::select! {
            Some(command) = command_event_listener.recv() => {
                handle_command_events(&mut swarm, command);
            },
            Some(event) = swarm.next() => {
                // Handle mDNS peer discovery by adding peers to Kademlia
                #[cfg(feature = "native")]
                if let libp2p::swarm::SwarmEvent::Behaviour(
                    crate::network::behaviour::NetabaseBehaviourEvent::Mdns(
                        libp2p::mdns::Event::Discovered(peers)
                    )
                ) = &event {
                    for (peer_id, multiaddr) in peers {
                        // Add the peer to Kademlia routing table
                        swarm.behaviour_mut().kad.add_address(peer_id, multiaddr.clone());
                        // Dial the peer to establish connection
                        if let Err(e) = swarm.dial(peer_id.clone()) {
                            eprintln!("Failed to dial mDNS peer {}: {:?}", peer_id, e);
                        }
                    }

                    // Bootstrap after discovering peers to join the DHT network
                    if !peers.is_empty() {
                        if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
                            eprintln!("Failed to bootstrap Kademlia: {:?}", e);
                        }
                    }
                }

                let event = NetabaseSwarmEvent(event);
                let _ = swarm_event_sender.send(event.clone());
                swarm_events::handle_swarm_events(event);
            }
        }
    }
}

// WASM placeholder - networking not yet implemented
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub(crate) async fn start_swarm_loop<D: NetabaseDefinition + Send + Sync + 'static>(
    _swarm: (),
    _swarm_event_sender: (),
    _command_event_listener: (),
) {
    // WASM swarm event loop would be implemented here
    // For WebSocket/WebRTC based networking
    panic!("WASM networking is not yet implemented");
}
