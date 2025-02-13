pub mod action_events;
pub mod libp2p_events;

use libp2p::{
    Swarm,
    futures::{StreamExt, channel::mpsc::Receiver},
};
use libp2p_events::handle_libp2p_events;
use tokio::{
    select,
    sync::{
        broadcast::{self, Sender},
        mpsc,
    },
};

use crate::{
    config::behaviour::{NetEvent, NetabaseBehaviour},
    swarm::swarm_config::swarm_init,
};

use super::messages::swarm_actions::SwarmAction;

pub async fn run_swarm(
    swarm: &mut Swarm<NetabaseBehaviour>,
    swarm_event_sender: &mut broadcast::Sender<NetEvent>,
    swarm_action_receiver: &mut mpsc::Receiver<SwarmAction>,
) {
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().expect("Parse erruh"))
        .expect("listen erruh");

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(libp2p::kad::Mode::Server));
    loop {
        select! {
            swarm_event = swarm.select_next_some() => {
                handle_libp2p_events(swarm, swarm_event, swarm_event_sender);
            }
            swarm_action = swarm_action_receiver.recv() => {
                swarm_action.expect("Action Receiver Error").execute(swarm);
            }
        }
    }
}
