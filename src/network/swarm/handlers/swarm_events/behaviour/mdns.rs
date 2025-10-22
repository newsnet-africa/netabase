use libp2p::{Multiaddr, PeerId, Swarm, mdns::Event as MdnsEvent};
use netabase_store::traits::definition::NetabaseDefinitionTrait;

use crate::network::{behaviour::NetabaseBehaviour, config::NetabaseConfig};

/// Handle mDNS behaviour events
pub fn handle_mdns_event<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    config: NetabaseConfig,
    mdns_event: MdnsEvent,
) where
    D: netabase_store::convert::ToIVec,
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
{
    match mdns_event {
        MdnsEvent::Discovered(peer_addresses) => {
            handle_discovered::<D>(swarm, config, &peer_addresses);
        }
        MdnsEvent::Expired(peer_addresses) => {
            handle_expired::<D>(peer_addresses);
        }
    }
}

/// Handle discovered peers via mDNS
fn handle_discovered<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    config: NetabaseConfig,
    peer_addresses: &Vec<(PeerId, Multiaddr)>,
) where
    D: netabase_store::convert::ToIVec,
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
{
    if config.dht_discovery.mdns_discovery.auto_connect.is_some() {
        for (peer_id, multiaddr) in peer_addresses {
            let peer_short = format!("{}", peer_id).chars().take(8).collect::<String>();
            println!("🔍 Discovered peer {} via mDNS\n", peer_short);
            // Add the peer to Kademlia routing table
            swarm
                .behaviour_mut()
                .kad
                .add_address(peer_id, multiaddr.clone());
            // Dial the peer to establish connection
            if let Err(e) = swarm.dial(*peer_id) {
                eprintln!("Failed to dial mDNS peer {}: {:?}", peer_id, e);
            }

            // Bootstrap after discovering peers to join the DHT network
            if !peer_addresses.is_empty()
                && let Err(e) = swarm.behaviour_mut().kad.bootstrap()
            {
                eprintln!("Failed to bootstrap Kademlia: {:?}", e);
            } else {
                println!("Bootstrapped!")
            }

            if let Err(e) = swarm.dial(*peer_id) {
                eprintln!("Failed to dial mDNS peer {}: {:?}", peer_id, e);
            }
        }

        // Bootstrap after discovering peer_addresses to join the DHT network
        if !peer_addresses.is_empty()
            && let Err(e) = swarm.behaviour_mut().kad.bootstrap()
        {
            eprintln!("Failed to bootstrap Kademlia: {:?}", e);
        } else {
            println!("Bootstrapped!")
        }
    }
}

/// Handle expired peer addresses from mDNS
fn handle_expired<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    _peer_addresses: Vec<(PeerId, Multiaddr)>,
) where
    D: netabase_store::convert::ToIVec,
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
{
    // Silent - peer expiration is normal in P2P networks
}
