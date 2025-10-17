use libp2p::{Multiaddr, PeerId, mdns::Event as MdnsEvent};
use netabase_store::traits::definition::NetabaseDefinition;

/// Handle mDNS behaviour events
pub fn handle_mdns_event<D: NetabaseDefinition + Send + Sync + 'static>(mdns_event: MdnsEvent) {
    match mdns_event {
        MdnsEvent::Discovered(peer_addresses) => {
            handle_discovered::<D>(peer_addresses);
        }
        MdnsEvent::Expired(peer_addresses) => {
            handle_expired::<D>(peer_addresses);
        }
    }
}

/// Handle discovered peers via mDNS
fn handle_discovered<D: NetabaseDefinition + Send + Sync + 'static>(peer_addresses: Vec<(PeerId, Multiaddr)>) {
    for (peer_id, _) in peer_addresses {
        let peer_short = format!("{}", peer_id).chars().take(8).collect::<String>();
        println!("🔍 Discovered peer {} via mDNS\n", peer_short);
    }
}

/// Handle expired peer addresses from mDNS
fn handle_expired<D: NetabaseDefinition + Send + Sync + 'static>(_peer_addresses: Vec<(PeerId, Multiaddr)>) {
    // Silent - peer expiration is normal in P2P networks
}
