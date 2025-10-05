use libp2p::{
    Swarm,
    kad::{NoKnownPeers, QueryResult},
};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

use super::super::swarm_events::behaviour::kad::store_query_response_channel;
use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_bootstrap<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    response_channel: Sender<Result<QueryResult, NoKnownPeers>>,
) {
    println!("Bootstrap command received");

    // Call the libp2p Kademlia API
    match swarm.behaviour_mut().kad.bootstrap() {
        Ok(query_id) => {
            // Store the response channel for when the query completes
            store_query_response_channel(query_id, response_channel);
            println!(
                "Bootstrap: Query started with ID {:?}, response will be sent via event loop",
                query_id
            );
        }
        Err(no_known_peers) => {
            // Send the error immediately
            if let Err(_) = response_channel.send(Err(no_known_peers)) {
                println!("Failed to send Bootstrap error response - receiver dropped");
            }
        }
    }
}
