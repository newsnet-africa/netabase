use libp2p::{Multiaddr, core::transport::ListenerId};
use netabase_store::traits::definition::NetabaseDefinition;

pub fn handle_new_listen_addr<D: NetabaseDefinition + Send + Sync + 'static>(_listener_id: ListenerId, address: Multiaddr) {
    println!("🎧 Listening on {}", address);
}
