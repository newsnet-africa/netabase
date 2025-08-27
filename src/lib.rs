use tokio::task::JoinHandle;

use crate::{
    netabase_trait::{NetabaseKeyRegistery, NetabaseRegistery},
    network::{
        event_loop::event_loop,
        event_messages::{command_messages::NetabaseCommand, swarm_messages::NetabaseEvent},
        generate_swarm,
    },
};

pub mod database;
pub mod netabase_trait;
pub mod network;

pub struct Netabase<K: NetabaseKeyRegistery, V: NetabaseRegistery> {
    swarm_thread: JoinHandle<()>,
    pub swarm_event_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
    pub swarm_command_sender: tokio::sync::mpsc::Sender<NetabaseCommand<K, V>>,
}

impl<K: NetabaseKeyRegistery, V: NetabaseRegistery> Netabase<K, V> {
    pub fn new_test(test_number: usize) -> Self {
        let swarm_thread: JoinHandle<()> = tokio::spawn(async move {
            const BOOTNODES: [&str; 4] = [
                "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
                "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
                "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
                "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
            ];
            let mut swarm =
                generate_swarm(test_number).expect("Eventually this thread should return result");
            for peer in BOOTNODES {
                swarm.behaviour_mut().kad.add_address(
                    &peer.parse().expect("Parse Erruh"),
                    "/dnsaddr/bootstrap.libp2p.io".parse().expect("ParseErruh"),
                );
            }
            event_loop(&mut swarm).await
        });
        Self {
            swarm_thread,
            swarm_event_listener: todo!(),
            swarm_command_sender: todo!(),
        }
    }
    pub async fn close(self) {
        let _ = self.swarm_thread.await;
    }
}
