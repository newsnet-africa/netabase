use libp2p::{Multiaddr, core::transport::ListenerId};
use netabase_store::traits::NetabaseSchema;

pub fn handle_new_listen_addr<S: NetabaseSchema>(listener_id: ListenerId, address: Multiaddr) {
    // TODO: Implement new listen address handling
    println!(
        "New listen address: listener_id: {:?}, address: {:?}",
        listener_id, address
    );
}
