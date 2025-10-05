use libp2p::{Swarm, kad::QueryResult};
use netabase_store::traits::{NetabaseKeys, NetabaseSchema};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_get_record<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    key: S::Keys,
    response_channel: Sender<QueryResult>,
) {
    println!("GetRecord command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Call the libp2p Kademlia API with the converted key
            let query_id = swarm.behaviour_mut().kad.get_record(record_key);

            // Store the response channel for when the query completes
            super::super::swarm_events::behaviour::kad::store_query_response_channel(
                query_id,
                response_channel,
            );

            println!(
                "GetRecord: Query started with ID {:?}, response will be sent via event loop",
                query_id
            );
        }
        Err(conversion_error) => {
            println!(
                "Failed to convert key to kad::RecordKey: {:?}",
                conversion_error
            );
            // Send an error response using a placeholder key
            let error_result = QueryResult::GetRecord(Err(libp2p::kad::GetRecordError::NotFound {
                key: libp2p::kad::RecordKey::new(&b"conversion_error"),
                closest_peers: vec![],
            }));
            if let Err(_) = response_channel.send(error_result) {
                println!("Failed to send GetRecord conversion error response - receiver dropped");
            }
        }
    }
}
