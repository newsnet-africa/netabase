use libp2p::{Multiaddr, identity::Keypair};
use tokio::task::JoinHandle;

use crate::{
    netabase_trait::{NetabaseSchema, NetabaseSchemaKey},
    network::{
        event_loop::event_loop,
        event_messages::{
            command_messages::{
                CommandResponse, CommandWithResponse, NetabaseCommand,
                database_commands::DatabaseCommand,
            },
            swarm_messages::NetabaseEvent,
        },
        generate_swarm, generate_swarm_with_config,
    },
};

pub mod config;
pub mod database;
pub mod netabase_trait;
pub mod network;
pub mod traits;

// Re-export commonly used configuration types for easier access
pub use config::{BehaviourConfig, NetabaseConfig, NetabaseSwarmConfig};

/// Main Netabase instance that manages the P2P network and database operations
///
/// This struct provides the main interface to the Netabase system, managing both
/// the libp2p swarm and the distributed database operations. It runs the network
/// stack in a background task and provides channels for command and event communication.
///
/// # Type Parameters
///
/// * `K` - Key type that implements `NetabaseSchemaKey`
/// * `V` - Value type that implements `NetabaseSchema`
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use netabase::Netabase;
///
/// // Create with default configuration
/// let netabase = Netabase::default();
///
/// // Create with custom configuration
/// let config = NetabaseConfig::default();
/// let netabase = Netabase::new(config);
///
/// // Create with just a keypair
/// let keypair = Keypair::generate_ed25519();
/// let netabase = Netabase::with_keypair(keypair);
/// ```
pub struct Netabase<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema + std::fmt::Debug> {
    swarm_thread: JoinHandle<()>,
    pub swarm_event_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
    pub swarm_command_sender: tokio::sync::mpsc::UnboundedSender<CommandWithResponse<K, V>>,
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
    pub fn new_test(test_number: usize) -> Self {
        let (command_sender, command_receiver) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
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
    /// Create a new Netabase instance with custom configuration
    ///
    /// This method creates a new Netabase instance using the provided configuration
    /// for both swarm settings and behavior configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for the Netabase instance
    ///
    /// # Returns
    ///
    /// Returns a new `Netabase` instance or panics if swarm creation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase::{Netabase, NetabaseConfig};
    /// use netabase::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
    ///
    /// let config = NetabaseConfig::default();
    /// let netabase: Netabase<MyKey, MyValue> = Netabase::new(config);
    /// ```
    pub fn new(config: NetabaseConfig) -> Self {
        let (command_sender, command_receiver) =
            tokio::sync::mpsc::unbounded_channel::<CommandWithResponse<K, V>>();
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(20);

        let bootstrap_nodes = config.swarm_config().bootstrap_nodes().to_vec();

        let swarm_thread: JoinHandle<()> = tokio::spawn(async move {
            let mut swarm =
                generate_swarm_with_config(&config).expect("Failed to create swarm with config");

            // Add configured bootstrap nodes
            for (peer_id, addr) in bootstrap_nodes {
                if let Ok(peer_id) = peer_id.parse() {
                    swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                }
            }

            // If no bootstrap nodes configured, use default ones
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

            event_loop(&mut swarm, event_sender, command_receiver).await;
        });

        Self {
            swarm_thread,
            swarm_event_listener: event_receiver,
            swarm_command_sender: command_sender,
        }
    }

    /// Create a new Netabase instance with a specific keypair
    ///
    /// This method creates a Netabase instance using the provided keypair
    /// and default configuration for everything else.
    ///
    /// # Arguments
    ///
    /// * `keypair` - The keypair to use for the node identity
    ///
    /// # Returns
    ///
    /// Returns a new `Netabase` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase::Netabase;
    /// use libp2p::identity::Keypair;
    ///
    /// let keypair = Keypair::generate_ed25519();
    /// let netabase: Netabase<MyKey, MyValue> = Netabase::with_keypair(keypair);
    /// ```
    pub fn with_keypair(keypair: Keypair) -> Self {
        let config = NetabaseConfig::builder()
            .swarm_config(
                NetabaseSwarmConfig::builder()
                    .identity(Some(keypair))
                    .build()
                    .expect("Valid swarm config with keypair"),
            )
            .behaviour_config(BehaviourConfig::default())
            .build()
            .expect("Valid netabase config with keypair");

        Self::new(config)
    }

    /// Create a new Netabase instance with a custom database path
    ///
    /// This method creates a Netabase instance using the provided database path
    /// and default configuration for everything else.
    ///
    /// # Arguments
    ///
    /// * `database_path` - The path where the database should be stored
    ///
    /// # Returns
    ///
    /// Returns a new `Netabase` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase::Netabase;
    ///
    /// let netabase: Netabase<MyKey, MyValue> = Netabase::with_database_path("./my_custom_db");
    /// ```
    pub fn with_database_path<P: AsRef<str>>(database_path: P) -> Self {
        let config = NetabaseConfig::builder()
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(
                BehaviourConfig::builder()
                    .database_path(database_path.as_ref().to_string())
                    .build()
                    .expect("Valid behaviour config with database path"),
            )
            .build()
            .expect("Valid netabase config with database path");

        Self::new(config)
    }

    /// Create a production-ready Netabase instance
    ///
    /// This method creates a Netabase instance with configuration optimized
    /// for production use, including disabled mDNS, higher connection limits,
    /// and longer timeouts.
    ///
    /// # Arguments
    ///
    /// * `keypair` - The keypair to use for the node identity
    /// * `database_path` - The path where the database should be stored
    /// * `listen_addresses` - The addresses to listen on
    ///
    /// # Returns
    ///
    /// Returns a new `Netabase` instance configured for production.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase::Netabase;
    /// use libp2p::identity::Keypair;
    ///
    /// let keypair = Keypair::generate_ed25519();
    /// let listen_addrs = vec![
    ///     "/ip4/0.0.0.0/tcp/4001".parse().unwrap(),
    ///     "/ip4/0.0.0.0/udp/4001/quic-v1".parse().unwrap(),
    /// ];
    /// let netabase: Netabase<MyKey, MyValue> = Netabase::production(
    ///     keypair,
    ///     "./production_db",
    ///     listen_addrs
    /// );
    /// ```
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
            .mdns_enabled(false) // Disable mDNS in production
            .listen_addresses(listen_addresses)
            .user_agent("NewsNet-Production/1.0.0".to_string())
            .build()
            .expect("Valid production swarm config");

        let behaviour_config = BehaviourConfig::builder()
            .database_path(database_path.as_ref().to_string())
            .protocol_version("/newsnet/1.0.0".to_string())
            .agent_version("NewsNet/1.0.0".to_string())
            .build()
            .expect("Valid production behaviour config");

        let config = NetabaseConfig::builder()
            .swarm_config(swarm_config)
            .behaviour_config(behaviour_config)
            .build()
            .expect("Valid production netabase config");

        Self::new(config)
    }

    /// Close the Netabase instance and clean up resources
    ///
    /// This method sends a close command to the swarm and waits for the
    /// background thread to terminate. It should be called when the
    /// Netabase instance is no longer needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// let netabase = Netabase::default();
    /// // ... use netabase ...
    /// netabase.close().await;
    /// ```
    pub async fn close(self) {
        let (response_sender, _) = tokio::sync::oneshot::channel();
        let command_with_response = CommandWithResponse {
            command: NetabaseCommand::Close,
            response_sender,
        };
        let _ = self.swarm_command_sender.send(command_with_response);
        let _ = self.swarm_thread.await;
    }

    // Database API methods

    /// Store a key-value pair in the distributed database
    ///
    /// This method stores the provided key-value pair in the distributed hash table
    /// using the Kademlia DHT protocol.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store
    /// * `value` - The value to store
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `NetabaseError` on failure.
    pub async fn put(&self, key: K, value: V) -> Result<(), NetabaseError> {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        let command = NetabaseCommand::Database(DatabaseCommand::Put { key, value });

        let command_with_response = CommandWithResponse {
            command,
            response_sender,
        };

        // Send command with response channel to swarm thread
        self.swarm_command_sender
            .send(command_with_response)
            .map_err(|e| NetabaseError::SendError(e.to_string()))?;

        // Wait for response
        let response = response_receiver
            .await
            .map_err(|e| NetabaseError::ReceiveError(e.to_string()))?;

        self.handle_response(response)
    }

    /// Retrieve a value by key from the distributed database
    ///
    /// This method queries the distributed hash table for the value associated
    /// with the provided key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(value))` if found, `Ok(None)` if not found,
    /// or a `NetabaseError` on failure.
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

    /// Delete a key-value pair from the distributed database
    ///
    /// This method removes the key-value pair from the distributed hash table
    /// by storing an empty value (tombstone).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `NetabaseError` on failure.
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

    /// Check if a key exists in the distributed database
    ///
    /// This method checks whether the provided key exists in the distributed
    /// hash table without retrieving the full value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the key exists, `Ok(false)` if not,
    /// or a `NetabaseError` on failure.
    pub async fn contains(&self, key: K) -> Result<bool, NetabaseError> {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        let command = NetabaseCommand::Database(DatabaseCommand::Contains { key });

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
                crate::network::event_messages::command_messages::DatabaseResponse::ExistsResult(
                    exists,
                ),
            ) => Ok(exists),
            CommandResponse::Error(msg) => Err(NetabaseError::OperationError(msg)),
            _ => Err(NetabaseError::UnexpectedResponse),
        }
    }

    // Helper methods

    /// Handle a generic command response
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
    /// Create a new Netabase instance with default configuration
    ///
    /// This creates a Netabase instance using `NetabaseConfig::default()`, which provides
    /// sensible defaults for development and testing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase::Netabase;
    /// use netabase::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
    ///
    /// let netabase: Netabase<MyKey, MyValue> = Netabase::default();
    /// ```
    fn default() -> Self {
        Self::new(NetabaseConfig::default())
    }
}
