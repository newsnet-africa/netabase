use tokio::task::JoinHandle;

use crate::{
    netabase_trait::{NetabaseSchema, NetabaseSchemaKey},
    network::{
        event_loop::event_loop,
        event_messages::{command_messages::NetabaseCommand, swarm_messages::NetabaseEvent},
        generate_swarm,
    },
};

pub mod database;
pub mod netabase_trait;
pub mod network;

pub struct Netabase<K: NetabaseSchemaKey, V: NetabaseSchema> {
    swarm_thread: JoinHandle<()>,
    pub swarm_event_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
    pub swarm_command_sender: tokio::sync::mpsc::UnboundedSender<NetabaseCommand<K, V>>,
}

impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> Netabase<K, V> {
    pub fn new_test(test_number: usize) -> Self {
        let (command_sender, command_receiver) =
            tokio::sync::mpsc::unbounded_channel::<NetabaseCommand<K, V>>();
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);
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
            event_loop(&mut swarm, event_sender, command_receiver).await;
        });
        Self {
            swarm_thread,
            swarm_event_listener: event_receiver,
            swarm_command_sender: command_sender,
        }
    }
    pub async fn close(self) {
        self.swarm_command_sender.send(NetabaseCommand::Close);
        self.swarm_thread.await;
    }
}
