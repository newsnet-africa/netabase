use libp2p::{Swarm, kad::Mode};
use netabase_store::traits::definition::NetabaseDefinition;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_set_mode<D: NetabaseDefinition + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    mode: Option<Mode>,
) {
    println!("SetMode command: mode={:?}", mode);

    // Call the libp2p Kademlia API
    swarm.behaviour_mut().kad.set_mode(mode);

    println!("SetMode: Mode updated successfully");

    // Note: This command doesn't have a response channel as it's fire-and-forget
}
