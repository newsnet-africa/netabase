use libp2p::{Multiaddr, identity::Keypair};
use tokio::task::JoinHandle;

use crate::network::{
    event_loop::event_loop,
    event_messages::{
        command_messages::{
            CommandResponse, CommandWithResponse, NetabaseCommand,
            database_commands::DatabaseCommand,
        },
        swarm_messages::NetabaseEvent,
    },
    generate_swarm, generate_swarm_with_config,
};

pub mod config;
pub mod database;
pub mod netabase_trait;
pub mod network;
pub mod traits;

pub use config::{
    BehaviourConfig, DefaultBehaviourConfig, DefaultNetabaseConfig, KadStoreConfig, NetabaseConfig,
    NetabaseSwarmConfig,
};
pub use netabase_trait::{NetabaseSchema, NetabaseSchemaKey};

pub struct Netabase<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema + std::fmt::Debug> {
    swarm_thread: Option<JoinHandle<()>>,
    pub swarm_event_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
    pub swarm_command_sender: tokio::sync::mpsc::UnboundedSender<CommandWithResponse<K, V>>,
    config: Option<DefaultNetabaseConfig>,
}

#[derive(Debug, thiserror::Error)]
pub enum NetabaseError {
    #[error("Channel send error: {0}")]
    SendError(String),
    #[error("Response receive error: {0}")]
    ReceiveError(String),
    #[error("Operation error: {0}")]
    OperationError(String),
    #[error("Unexpected response type")]
    UnexpectedResponse,
}

impl<
    K: NetabaseSchemaKey + std::fmt::Debug + 'static,
    V: NetabaseSchema + std::fmt::Debug + 'static,
> Netabase<K, V>
{
    pub fn new_test(test_number: usize, server: bool) -> Self {
        let config = crate::config::DefaultNetabaseConfig::builder()
            .swarm_config(crate::config::NetabaseSwarmConfig::default())
            .behaviour_config(
                crate::config::DefaultBehaviourConfig::builder()
                    .store_config(KadStoreConfig::sled_store(format!(
                        "./test/database{test_number}"
                    )))
                    .protocol_version("/p2p/newsnet/0.0.0".to_string())
                    .build()
                    .expect("Default BehaviourConfig should be valid"),
            )
            .build()
            .expect("Default NetabaseConfig should be valid");

        let (command_sender, _) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
        let (_, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);

        let mut instance = Self {
            swarm_thread: None,
            swarm_event_listener: event_receiver,
            swarm_command_sender: command_sender,
            config: Some(config),
        };

        // Start the swarm for backward compatibility
        instance
            .start_swarm_test(test_number, server)
            .expect("Failed to start test swarm");

        instance
    }

    pub fn new_test_with_mdns_auto_connect(
        test_number: usize,
        server: bool,
        auto_connect: bool,
    ) -> Self {
        let config = crate::config::DefaultNetabaseConfig::builder()
            .swarm_config(
                crate::config::NetabaseSwarmConfig::builder()
                    .mdns_auto_connect(auto_connect)
                    .build()
                    .expect("Valid swarm config"),
            )
            .behaviour_config(
                crate::config::DefaultBehaviourConfig::builder()
                    .store_config(KadStoreConfig::sled_store(format!(
                        "./test/database{test_number}"
                    )))
                    .protocol_version("/p2p/newsnet/0.0.0".to_string())
                    .build()
                    .expect("Default BehaviourConfig should be valid"),
            )
            .build()
            .expect("Default NetabaseConfig should be valid");

        let (command_sender, _) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
        let (_, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);

        let mut instance = Self {
            swarm_thread: None,
            swarm_event_listener: event_receiver,
            swarm_command_sender: command_sender,
            config: Some(config),
        };

        // Start the swarm automatically
        instance
            .start_swarm_test(test_number, server)
            .expect("Failed to start test swarm");

        instance
    }

    pub fn start_swarm_test(&mut self, test_number: usize, server: bool) -> Result<(), String> {
        if self.swarm_thread.is_some() {
            return Err("Swarm is already running".to_string());
        }

        let config = self.config.clone().ok_or("No config available")?;

        let (command_sender, command_receiver) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);

        self.swarm_command_sender = command_sender;
        self.swarm_event_listener = event_receiver;
        let swarm_thread: JoinHandle<()> = tokio::spawn(async move {
            const BOOTNODES: [&str; 4] = [
                "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
                "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
                "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
                "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
            ];
            let mut swarm =
                generate_swarm(test_number).expect("Eventually this thread should return result");

            // Set Kademlia mode based on server parameter
            if server {
                swarm
                    .behaviour_mut()
                    .kad
                    .set_mode(Some(::macro_exports::__netabase_libp2p_kad::Mode::Server));
            } else {
                swarm
                    .behaviour_mut()
                    .kad
                    .set_mode(Some(::macro_exports::__netabase_libp2p_kad::Mode::Client));
            }
            let res = swarm.listen_on(
                "/ip4/0.0.0.0/udp/0/quic-v1"
                    .parse()
                    .expect("multiaddr erruh"),
            );

            println!("{res:?}");
            for peer in BOOTNODES {
                swarm.behaviour_mut().kad.add_address(
                    &peer.parse().expect("Parse Erruh"),
                    "/dnsaddr/bootstrap.libp2p.io".parse().expect("ParseErruh"),
                );
            }
            event_loop(&mut swarm, event_sender, command_receiver, &config).await;
        });
        self.swarm_thread = Some(swarm_thread);
        Ok(())
    }
    pub fn new(config: DefaultNetabaseConfig) -> Self {
        let (command_sender, _) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
        let (_, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);

        Self {
            swarm_thread: None,
            swarm_event_listener: event_receiver,
            swarm_command_sender: command_sender,
            config: Some(config),
        }
    }

    pub fn new_with_auto_start(config: DefaultNetabaseConfig) -> Self {
        let mut instance = Self::new(config);
        instance.start_swarm().expect("Failed to start swarm");
        instance
    }

    pub fn start_swarm(&mut self) -> Result<(), String> {
        if self.swarm_thread.is_some() {
            return Err("Swarm is already running".to_string());
        }

        let config = self.config.clone().ok_or("No config available")?;

        let (command_sender, command_receiver) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);

        self.swarm_command_sender = command_sender;
        self.swarm_event_listener = event_receiver;

        let bootstrap_nodes = config.swarm_config().bootstrap_nodes().to_vec();

        let swarm_thread: JoinHandle<()> = tokio::spawn(async move {
            let mut swarm =
                generate_swarm_with_config(&config).expect("Failed to create swarm with config");

            for (peer_id, addr) in bootstrap_nodes {
                if let Ok(peer_id) = peer_id.parse() {
                    swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                }
            }

            if config.swarm_config().bootstrap_nodes().is_empty() {
                const DEFAULT_BOOTNODES: [&str; 4] = [
                    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
                    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
                    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
                    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
                ];
                for peer in DEFAULT_BOOTNODES {
                    swarm.behaviour_mut().kad.add_address(
                        &peer.parse().expect("Parse Error"),
                        "/dnsaddr/bootstrap.libp2p.io".parse().expect("Parse Error"),
                    );
                }
            }

            event_loop(&mut swarm, event_sender, command_receiver, &config).await;
        });

        self.swarm_thread = Some(swarm_thread);
        Ok(())
    }

    pub fn with_keypair(keypair: Keypair) -> Self {
        let config = DefaultNetabaseConfig::builder()
            .swarm_config(
                NetabaseSwarmConfig::builder()
                    .identity(Some(keypair))
                    .build()
                    .expect("Valid swarm config with keypair"),
            )
            .behaviour_config(DefaultBehaviourConfig::default())
            .build()
            .expect("Valid netabase config with keypair");

        Self::new_with_auto_start(config)
    }

    pub fn with_database_path<P: AsRef<str>>(database_path: P) -> Self {
        let config = DefaultNetabaseConfig::builder()
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(
                DefaultBehaviourConfig::builder()
                    .store_config(KadStoreConfig::sled_store(database_path))
                    .build()
                    .expect("Valid behaviour config with database path"),
            )
            .build()
            .expect("Valid netabase config with database path");

        Self::new_with_auto_start(config)
    }

    pub fn production<P: AsRef<str>>(
        keypair: Keypair,
        database_path: P,
        listen_addresses: Vec<Multiaddr>,
    ) -> Self {
        use std::time::Duration;

        let swarm_config = NetabaseSwarmConfig::builder()
            .identity(Some(keypair))
            .connection_timeout(Duration::from_secs(30))
            .idle_connection_timeout(Duration::from_secs(120))
            .max_connections_per_peer(Some(8))
            .max_pending_connections(Some(1024))
            .max_negotiating_inbound_streams(Some(512))
            .mdns_enabled(false)
            .listen_addresses(listen_addresses)
            .user_agent("NewsNet-Production/1.0.0".to_string())
            .build()
            .expect("Valid production swarm config");

        let behaviour_config = DefaultBehaviourConfig::builder()
            .store_config(KadStoreConfig::sled_store(database_path))
            .protocol_version("/newsnet/1.0.0".to_string())
            .agent_version("NewsNet/1.0.0".to_string())
            .build()
            .expect("Valid production behaviour config");

        let config = DefaultNetabaseConfig::builder()
            .swarm_config(swarm_config)
            .behaviour_config(behaviour_config)
            .build()
            .expect("Valid production netabase config");

        Self::new_with_auto_start(config)
    }

    pub async fn close(mut self) {
        let (response_sender, _) = tokio::sync::oneshot::channel();
        let command_with_response = CommandWithResponse {
            command: NetabaseCommand::Close,
            response_sender,
        };
        let _ = self.swarm_command_sender.send(command_with_response);
        if let Some(thread) = self.swarm_thread.take() {
            let _ = thread.await;
        }
    }

    pub async fn put(&self, key: K, value: V) -> Result<(), NetabaseError> {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        let command = NetabaseCommand::Database(DatabaseCommand::Put { key, value });

        let command_with_response = CommandWithResponse {
            command,
            response_sender,
        };

        self.swarm_command_sender
            .send(command_with_response)
            .map_err(|e| NetabaseError::SendError(e.to_string()))?;

        let response = response_receiver
            .await
            .map_err(|e| NetabaseError::ReceiveError(e.to_string()))?;

        self.handle_response(response)
    }

    pub async fn get(&self, key: K) -> Result<Option<V>, NetabaseError> {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        let command = NetabaseCommand::Database(DatabaseCommand::Get { key });

        let command_with_response = CommandWithResponse {
            command,
            response_sender,
        };

        self.swarm_command_sender
            .send(command_with_response)
            .map_err(|e| NetabaseError::SendError(e.to_string()))?;

        let response = response_receiver
            .await
            .map_err(|e| NetabaseError::ReceiveError(e.to_string()))?;

        match response {
            CommandResponse::Database(
                crate::network::event_messages::command_messages::DatabaseResponse::GetResult(
                    result,
                ),
            ) => Ok(result),
            CommandResponse::Error(msg) => Err(NetabaseError::OperationError(msg)),
            _ => Err(NetabaseError::UnexpectedResponse),
        }
    }

    pub async fn delete(&self, key: K) -> Result<(), NetabaseError> {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        let command = NetabaseCommand::Database(DatabaseCommand::Delete { key });

        let command_with_response = CommandWithResponse {
            command,
            response_sender,
        };

        self.swarm_command_sender
            .send(command_with_response)
            .map_err(|e| NetabaseError::SendError(e.to_string()))?;

        let response = response_receiver
            .await
            .map_err(|e| NetabaseError::ReceiveError(e.to_string()))?;

        self.handle_response(response)
    }

    fn handle_response(&self, response: CommandResponse<K, V>) -> Result<(), NetabaseError> {
        match response {
            CommandResponse::Success => Ok(()),
            CommandResponse::Error(msg) => Err(NetabaseError::OperationError(msg)),
            _ => Err(NetabaseError::UnexpectedResponse),
        }
    }
}

impl<
    K: NetabaseSchemaKey + std::fmt::Debug + 'static,
    V: NetabaseSchema + std::fmt::Debug + 'static,
> Default for Netabase<K, V>
{
    fn default() -> Self {
        let config = DefaultNetabaseConfig::builder()
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(DefaultBehaviourConfig::default())
            .build()
            .expect("Valid default netabase config");
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DefaultBehaviourConfig, NetabaseSwarmConfig};

    #[test]
    fn test_mdns_auto_connect_config_creation() {
        let config = DefaultNetabaseConfig::builder()
            .swarm_config(
                NetabaseSwarmConfig::builder()
                    .mdns_auto_connect(true)
                    .build()
                    .expect("Valid swarm config"),
            )
            .behaviour_config(DefaultBehaviourConfig::default())
            .build()
            .expect("Valid config");

        // Verify the auto connect setting is preserved
        assert!(config.swarm_config().mdns_auto_connect());
    }

    #[test]
    fn test_mdns_auto_connect_config_disabled() {
        let config = DefaultNetabaseConfig::builder()
            .swarm_config(
                NetabaseSwarmConfig::builder()
                    .mdns_auto_connect(false)
                    .build()
                    .expect("Valid swarm config"),
            )
            .behaviour_config(DefaultBehaviourConfig::default())
            .build()
            .expect("Valid config");

        // Verify the auto connect setting is disabled
        assert!(!config.swarm_config().mdns_auto_connect());
    }

    #[test]
    fn test_default_mdns_auto_connect_setting() {
        let default_config = NetabaseSwarmConfig::default();

        // Default should be false for auto connect
        assert!(!default_config.mdns_auto_connect());

        // But mDNS should be enabled by default
        assert!(default_config.mdns_enabled());
    }

    #[test]
    fn test_mdns_auto_connect_builder_pattern() {
        let config_enabled = NetabaseSwarmConfig::builder()
            .mdns_enabled(true)
            .mdns_auto_connect(true)
            .build()
            .expect("Valid config");

        assert!(config_enabled.mdns_enabled());
        assert!(config_enabled.mdns_auto_connect());

        let config_disabled = NetabaseSwarmConfig::builder()
            .mdns_enabled(true)
            .mdns_auto_connect(false)
            .build()
            .expect("Valid config");

        assert!(config_disabled.mdns_enabled());
        assert!(!config_disabled.mdns_auto_connect());
    }
}
