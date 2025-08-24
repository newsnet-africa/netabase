use libp2p::{Swarm, mdns};

use crate::network::behaviour::NetabaseBehaviour;

pub(super) fn handle_mdns_events(event: mdns::Event, swarm: &mut Swarm<NetabaseBehaviour>) {
    match event {
        mdns::Event::Discovered(items) => {
            for (peer_id, multi) in items {
                eprintln!("Dialing {peer_id}, or {multi}");
                let dial_result = swarm.dial(peer_id).map_err(|_| swarm.dial(multi));
                eprintln!("Dial Result: {dial_result:?}");
            }
        }
        mdns::Event::Expired(items) => todo!(),
    }
}
