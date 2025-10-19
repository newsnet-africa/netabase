use libp2p::{
    PeerId,
    identify::{Event as IdentifyEvent, Info, UpgradeError},
    swarm::{ConnectionId, StreamUpgradeError},
};
use netabase_store::traits::definition::NetabaseDefinitionTrait;

/// Handle Identify behaviour events
pub fn handle_identify_event<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    identify_event: IdentifyEvent,
) {
    match identify_event {
        IdentifyEvent::Received { peer_id, .. } => {
            handle_received::<D>(peer_id);
        }
        IdentifyEvent::Sent { .. } => {
            // Silent
        }
        IdentifyEvent::Pushed { .. } => {
            // Silent
        }
        IdentifyEvent::Error { peer_id, error, .. } => {
            handle_error::<D>(peer_id, error);
        }
    }
}

/// Handle received identification information
fn handle_received<D: NetabaseDefinitionTrait + Send + Sync + 'static>(peer_id: PeerId) {
    let peer_short = format!("{}", peer_id).chars().take(8).collect::<String>();
    println!("🤝 Connected to peer {}\n", peer_short);
}

/// Handle identification errors
fn handle_error<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    peer_id: PeerId,
    error: StreamUpgradeError<UpgradeError>,
) {
    eprintln!("⚠️  Identify error with peer {:?}: {:?}", peer_id, error);
}
