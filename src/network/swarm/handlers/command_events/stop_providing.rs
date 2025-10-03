use libp2p::Swarm;
use netabase_store::traits::{NetabaseKeys, NetabaseSchema};

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_stop_providing<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    key: S::Keys,
) {
    println!("StopProviding command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Call the libp2p Kademlia API with the converted key
            swarm.behaviour_mut().kad.stop_providing(&record_key);
            println!("StopProviding: Provider registration stopped successfully");
        }
        Err(conversion_error) => {
            println!(
                "Failed to convert key to kad::RecordKey: {:?}",
                conversion_error
            );
        }
    }

    // Note: This command doesn't have a response channel as it's fire-and-forget
}
