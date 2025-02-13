use libp2p::{
    Swarm, SwarmBuilder,
    swarm::{self, NetworkBehaviour},
};

use crate::{
    config::behaviour::NetabaseBehaviour,
    swarm::messages::{self, swarm_actions::SwarmAction},
};

impl SwarmAction {
    pub fn execute(&self, swarm: &mut Swarm<NetabaseBehaviour>) {
        match self {
            SwarmAction::KadAction(kad_behaviour_methods) => match kad_behaviour_methods {
                messages::swarm_actions::KadBehaviourMethods::AddAddress(peer_id, multiaddr) => {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(peer_id, multiaddr.clone());
                }
                messages::swarm_actions::KadBehaviourMethods::BOOTSTRAP => {
                    swarm.behaviour_mut().kademlia.bootstrap();
                }
                messages::swarm_actions::KadBehaviourMethods::GetProviders(key) => {
                    swarm.behaviour_mut().kademlia.get_providers(key.to_owned());
                }
                messages::swarm_actions::KadBehaviourMethods::GetRecord(key) => {
                    swarm.behaviour_mut().kademlia.get_record(key.to_owned());
                }
                messages::swarm_actions::KadBehaviourMethods::PutRecord(record, quorum) => {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .put_record(record.to_owned(), quorum.to_owned())
                        .expect("Put Record Error");
                }
                messages::swarm_actions::KadBehaviourMethods::PutRecordTo(
                    record,
                    peer_ids,
                    quorum,
                ) => {
                    swarm.behaviour_mut().kademlia.put_record_to(
                        record.to_owned(),
                        peer_ids.iter().map(|item| item.to_owned()).to_owned(),
                        quorum.to_owned(),
                    );
                }
                messages::swarm_actions::KadBehaviourMethods::RemoveAddress(peer_id, multiaddr) => {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .remove_address(peer_id, multiaddr);
                }
                messages::swarm_actions::KadBehaviourMethods::RemovePeer(peer_id) => {
                    swarm.behaviour_mut().kademlia.remove_peer(peer_id);
                }
                messages::swarm_actions::KadBehaviourMethods::SetMode(mode) => {
                    swarm.behaviour_mut().kademlia.set_mode(*mode);
                }
                messages::swarm_actions::KadBehaviourMethods::StartProviding(key) => {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .start_providing(key.to_owned())
                        .expect("Providing failed");
                }
                messages::swarm_actions::KadBehaviourMethods::StopProviding(key) => {
                    swarm.behaviour_mut().kademlia.stop_providing(key);
                }
            },
            SwarmAction::MdnsAction(mdns_behaviour_methods) => match mdns_behaviour_methods {
                messages::swarm_actions::MdnsBehaviourMethods::DiscoveredNodes => {
                    swarm.behaviour_mut().mdns.discovered_nodes();
                }
            },
            SwarmAction::Ping => {}
            SwarmAction::Test => {
                println!("GOT TEST");
            }
        }
    }
}
