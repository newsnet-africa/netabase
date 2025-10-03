use libp2p::{Swarm, kad};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_put_record<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    record: S,
    response_channel: Sender<Result<kad::QueryResult, kad::store::Error>>,
) {
    println!("PutRecord command: record={:?}", record);

    // Convert NetabaseSchema to libp2p::kad::Record
    match record.to_record() {
        Ok(kad_record) => {
            // Call the libp2p Kademlia API with the converted record
            match swarm
                .behaviour_mut()
                .kad
                .put_record(kad_record, kad::Quorum::One)
            {
                Ok(query_id) => {
                    // Store the response channel for when the query completes
                    super::super::swarm_events::behaviour::kad::store_query_response_channel(
                        query_id,
                        response_channel,
                    );
                    println!(
                        "PutRecord: Query started with ID {:?}, response will be sent via event loop",
                        query_id
                    );
                }
                Err(store_error) => {
                    // Send the error immediately
                    if let Err(_) = response_channel.send(Err(store_error)) {
                        println!("Failed to send PutRecord error response - receiver dropped");
                    }
                }
            }
        }
        Err(conversion_error) => {
            println!(
                "Failed to convert record to kad::Record: {:?}",
                conversion_error
            );
            // Convert NetabaseError to store::Error (this is a simplification)
            if let Err(_) = response_channel.send(Err(kad::store::Error::MaxRecords)) {
                println!("Failed to send PutRecord conversion error response - receiver dropped");
            }
        }
    }
}
