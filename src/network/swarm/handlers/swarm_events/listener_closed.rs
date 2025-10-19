use libp2p::{Multiaddr, core::transport::ListenerId};
use netabase_store::traits::definition::NetabaseDefinitionTrait;

pub fn handle_listener_closed<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    listener_id: ListenerId,
    addresses: Vec<Multiaddr>,
    reason: Result<(), std::io::Error>,
) {
    // TODO: Implement listener closed handling
    println!(
        "Listener closed: listener_id: {:?}, addresses: {:?}",
        listener_id, &addresses
    );

    match reason {
        Ok(()) => {
            println!("Listener closed gracefully");
        }
        Err(error) => {
            println!("Listener closed due to error: {:?}", error);
        }
    }
}
