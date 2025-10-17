use libp2p::{Multiaddr, core::transport::ListenerId};
use netabase_store::traits::definition::NetabaseDefinition;

pub fn handle_expired_listen_addr<D: NetabaseDefinition + Send + Sync + 'static>(listener_id: ListenerId, address: Multiaddr) {
    // TODO: Implement expired listen address handling
    println!(
        "Listen address expired: listener_id: {:?}, address: {:?}",
        listener_id, address
    );
}
