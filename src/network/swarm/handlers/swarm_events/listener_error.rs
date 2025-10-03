use libp2p::core::transport::ListenerId;
use netabase_store::traits::NetabaseSchema;

pub fn handle_listener_error<S: NetabaseSchema>(listener_id: ListenerId, error: std::io::Error) {
    // TODO: Implement listener error handling
    println!(
        "Listener error: listener_id: {:?}, error: {:?}",
        listener_id, error
    );

    // Log the specific error type for debugging
    eprintln!("Listener {:?} encountered error: {}", listener_id, error);
}
