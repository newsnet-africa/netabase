use libp2p::Swarm;
use netabase_store::traits::definition::{NetabaseDefinition, NetabaseDefinitionKey};

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_stop_providing<D: NetabaseDefinition>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    key: D::Keys,
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
