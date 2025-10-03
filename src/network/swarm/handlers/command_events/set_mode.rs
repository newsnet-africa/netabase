use libp2p::{Swarm, kad::Mode};
use netabase_store::traits::NetabaseSchema;

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_set_mode<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    mode: Option<Mode>,
) {
    println!("SetMode command: mode={:?}", mode);

    // Call the libp2p Kademlia API
    swarm.behaviour_mut().kad.set_mode(mode);

    println!("SetMode: Mode updated successfully");

    // Note: This command doesn't have a response channel as it's fire-and-forget
}
