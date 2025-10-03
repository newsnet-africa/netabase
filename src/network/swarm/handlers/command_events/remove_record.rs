use libp2p::Swarm;
use netabase_store::traits::{NetabaseKeys, NetabaseSchema};

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_remove_record<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    key: S::Keys,
) {
    println!("RemoveRecord command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Call the libp2p Kademlia API with the converted key
            swarm.behaviour_mut().kad.remove_record(&record_key);
            println!("RemoveRecord: Record removal requested successfully");
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
