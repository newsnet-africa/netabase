use libp2p::Swarm;
use libp2p::futures::StreamExt;

use crate::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
use crate::network::behaviour::NetabaseBehaviour;
use crate::network::event_loop::handle_behaviour_events::handle_behaviour_event;
use crate::network::event_messages::command_messages::NetabaseCommand;
use crate::network::event_messages::swarm_messages::NetabaseEvent;
pub mod handle_behaviour_events;

pub async fn event_loop<K: NetabaseSchemaKey, V: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour>,
    mut event_sender: tokio::sync::broadcast::Sender<NetabaseEvent>,
    mut command_receiver: tokio::sync::mpsc::UnboundedReceiver<NetabaseCommand<K, V>>,
) {
    let list_res = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().expect("Multi Parse erruh"));
    println!("Starting loop: Listen res: {list_res:?}");
    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                let sent_event = NetabaseEvent(event);
                event_sender.send(sent_event.clone());
                match sent_event.0 {
                    libp2p::swarm::SwarmEvent::Behaviour(event) => handle_behaviour_event(event),
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => {},
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {},
                    libp2p::swarm::SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {},
                    libp2p::swarm::SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error, peer_id } => {},
                    libp2p::swarm::SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => {},
                    libp2p::swarm::SwarmEvent::NewListenAddr { listener_id, address } => {},
                    libp2p::swarm::SwarmEvent::ExpiredListenAddr { listener_id, address } => {},
                    libp2p::swarm::SwarmEvent::ListenerClosed { listener_id, addresses, reason } => {},
                    libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => {},
                    libp2p::swarm::SwarmEvent::Dialing { peer_id, connection_id } => {},
                    libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => {},
                    libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => {},
                    libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => {},
                    libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {},
                    _ => {},
                }
            },
            command = command_receiver.recv() => {
                break;
            }
        }
    }
}
