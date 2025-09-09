use libp2p::{Swarm, mdns};

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_mdns_event(
    event: mdns::Event,
    swarm: &mut Swarm<NetabaseBehaviour>,
    auto_connect_enabled: bool,
) {
    println!("MDNS Event: {event:?}");
    match event {
        mdns::Event::Discovered(items) => {
            if auto_connect_enabled {
                for (peer_id, multiaddr) in items {
                    println!(
                        "Adding discovered mDNS peer {peer_id} with address {multiaddr} to Kademlia table"
                    );
                    // Add the peer address to the Kademlia routing table
                    swarm.behaviour_mut().kad.add_address(&peer_id, multiaddr);
                }
            }
        }
        mdns::Event::Expired(items) => {
            if auto_connect_enabled {
                for (peer_id, multiaddr) in items {
                    println!(
                        "Removing expired mDNS peer {peer_id} with address {multiaddr} from Kademlia table"
                    );
                    // Remove the peer address from the Kademlia routing table
                    swarm
                        .behaviour_mut()
                        .kad
                        .remove_address(&peer_id, &multiaddr);
                }
            }
        }
    }
}
