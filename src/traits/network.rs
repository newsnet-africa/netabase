use crate::netabase_trait::{
    NetabaseRegistery, NetabaseRegistryKey, NetabaseSchema, NetabaseSchemaKey,
};
use async_trait::async_trait;
use libp2p::{Multiaddr, PeerId};

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Errors that can occur during network operations
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed to peer {peer_id}: {message}")]
    ConnectionFailed { peer_id: PeerId, message: String },

    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: PeerId },

    #[error("Network timeout: {operation}")]
    Timeout { operation: String },

    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Network is not started")]
    NetworkNotStarted,

    #[error("Network is already started")]
    NetworkAlreadyStarted,

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Transport error: {message}")]
    TransportError { message: String },

    #[error("DHT error: {message}")]
    DhtError { message: String },

    #[error("Gossipsub error: {message}")]
    GossipsubError { message: String },
}

impl Clone for NetworkError {
    fn clone(&self) -> Self {
        match self {
            NetworkError::ConnectionFailed { peer_id, message } => NetworkError::ConnectionFailed {
                peer_id: *peer_id,
                message: message.clone(),
            },
            NetworkError::PeerNotFound { peer_id } => {
                NetworkError::PeerNotFound { peer_id: *peer_id }
            }
            NetworkError::Timeout { operation } => NetworkError::Timeout {
                operation: operation.clone(),
            },
            NetworkError::InvalidAddress { address } => NetworkError::InvalidAddress {
                address: address.clone(),
            },
            NetworkError::ProtocolError { message } => NetworkError::ProtocolError {
                message: message.clone(),
            },
            NetworkError::AuthenticationFailed { message } => NetworkError::AuthenticationFailed {
                message: message.clone(),
            },
            NetworkError::NetworkNotStarted => NetworkError::NetworkNotStarted,
            NetworkError::NetworkAlreadyStarted => NetworkError::NetworkAlreadyStarted,
            NetworkError::SerializationError { source } => NetworkError::SerializationError {
                source: format!("{}", source).into(),
            },
            NetworkError::TransportError { message } => NetworkError::TransportError {
                message: message.clone(),
            },
            NetworkError::DhtError { message } => NetworkError::DhtError {
                message: message.clone(),
            },
            NetworkError::GossipsubError { message } => NetworkError::GossipsubError {
                message: message.clone(),
            },
        }
    }
}

/// Kademlia DHT mode configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KademliaDhtMode {
    /// Server mode - actively participates in DHT, stores records, responds to queries
    Server,
    /// Client mode - can query DHT but doesn't store records or respond to queries
    Client,
    /// Auto mode - dynamically switches based on network conditions
    Auto,
}

impl Default for KademliaDhtMode {
    fn default() -> Self {
        KademliaDhtMode::Server
    }
}

/// Configuration for network operations
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addresses: Vec<Multiaddr>,
    pub bootstrap_nodes: Vec<(String, Multiaddr)>,
    pub connection_timeout: std::time::Duration,
    pub idle_connection_timeout: std::time::Duration,
    pub max_connections_per_peer: Option<u32>,
    pub max_pending_connections: Option<u32>,
    pub max_negotiating_inbound_streams: Option<u32>,
    pub user_agent: String,
    pub protocol_version: String,
    pub mdns_enabled: bool,
    pub gossipsub_enabled: bool,
    pub kademlia_enabled: bool,
    pub kademlia_dht_mode: KademliaDhtMode,
    pub identify_enabled: bool,
    pub ping_enabled: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
                "/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap(),
            ],
            bootstrap_nodes: vec![],
            connection_timeout: std::time::Duration::from_secs(10),
            idle_connection_timeout: std::time::Duration::from_secs(60),
            max_connections_per_peer: Some(4),
            max_pending_connections: Some(256),
            max_negotiating_inbound_streams: Some(256),
            user_agent: "NewsNet/1.0.0".to_string(),
            protocol_version: "/newsnet/1.0.0".to_string(),
            mdns_enabled: true,
            gossipsub_enabled: true,
            kademlia_enabled: true,
            kademlia_dht_mode: KademliaDhtMode::default(),
            identify_enabled: true,
            ping_enabled: true,
        }
    }
}

impl NetworkConfig {
    /// Create a new NetworkConfig builder
    pub fn builder() -> NetworkConfigBuilder {
        NetworkConfigBuilder::default()
    }

    /// Set the Kademlia DHT mode
    pub fn with_dht_mode(mut self, mode: KademliaDhtMode) -> Self {
        self.kademlia_dht_mode = mode;
        self
    }

    /// Configure for server mode (default)
    pub fn server_mode(mut self) -> Self {
        self.kademlia_dht_mode = KademliaDhtMode::Server;
        self
    }

    /// Configure for client mode
    pub fn client_mode(mut self) -> Self {
        self.kademlia_dht_mode = KademliaDhtMode::Client;
        self
    }

    /// Configure for auto mode
    pub fn auto_mode(mut self) -> Self {
        self.kademlia_dht_mode = KademliaDhtMode::Auto;
        self
    }

    /// Set listen addresses
    pub fn with_listen_addresses(mut self, addresses: Vec<Multiaddr>) -> Self {
        self.listen_addresses = addresses;
        self
    }

    /// Add a listen address
    pub fn add_listen_address(mut self, address: Multiaddr) -> Self {
        self.listen_addresses.push(address);
        self
    }

    /// Set bootstrap nodes
    pub fn with_bootstrap_nodes(mut self, nodes: Vec<(String, Multiaddr)>) -> Self {
        self.bootstrap_nodes = nodes;
        self
    }

    /// Add a bootstrap node
    pub fn add_bootstrap_node(mut self, peer_id: String, address: Multiaddr) -> Self {
        self.bootstrap_nodes.push((peer_id, address));
        self
    }

    /// Set user agent
    pub fn with_user_agent<S: AsRef<str>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.as_ref().to_string();
        self
    }

    /// Enable or disable mDNS
    pub fn with_mdns(mut self, enabled: bool) -> Self {
        self.mdns_enabled = enabled;
        self
    }

    /// Enable or disable Kademlia
    pub fn with_kademlia(mut self, enabled: bool) -> Self {
        self.kademlia_enabled = enabled;
        self
    }
}

/// Builder for NetworkConfig
#[derive(Debug, Clone)]
pub struct NetworkConfigBuilder {
    config: NetworkConfig,
}

impl Default for NetworkConfigBuilder {
    fn default() -> Self {
        Self {
            config: NetworkConfig::default(),
        }
    }
}

impl NetworkConfigBuilder {
    /// Set the Kademlia DHT mode
    pub fn dht_mode(mut self, mode: KademliaDhtMode) -> Self {
        self.config.kademlia_dht_mode = mode;
        self
    }

    /// Configure for server mode
    pub fn server_mode(mut self) -> Self {
        self.config.kademlia_dht_mode = KademliaDhtMode::Server;
        self
    }

    /// Configure for client mode
    pub fn client_mode(mut self) -> Self {
        self.config.kademlia_dht_mode = KademliaDhtMode::Client;
        self
    }

    /// Configure for auto mode
    pub fn auto_mode(mut self) -> Self {
        self.config.kademlia_dht_mode = KademliaDhtMode::Auto;
        self
    }

    /// Set listen addresses
    pub fn listen_addresses(mut self, addresses: Vec<Multiaddr>) -> Self {
        self.config.listen_addresses = addresses;
        self
    }

    /// Add a listen address
    pub fn add_listen_address(mut self, address: Multiaddr) -> Self {
        self.config.listen_addresses.push(address);
        self
    }

    /// Set bootstrap nodes
    pub fn bootstrap_nodes(mut self, nodes: Vec<(String, Multiaddr)>) -> Self {
        self.config.bootstrap_nodes = nodes;
        self
    }

    /// Add a bootstrap node
    pub fn add_bootstrap_node(mut self, peer_id: String, address: Multiaddr) -> Self {
        self.config.bootstrap_nodes.push((peer_id, address));
        self
    }

    /// Set user agent
    pub fn user_agent<S: AsRef<str>>(mut self, user_agent: S) -> Self {
        self.config.user_agent = user_agent.as_ref().to_string();
        self
    }

    /// Set protocol version
    pub fn protocol_version<S: AsRef<str>>(mut self, version: S) -> Self {
        self.config.protocol_version = version.as_ref().to_string();
        self
    }

    /// Enable or disable mDNS
    pub fn mdns(mut self, enabled: bool) -> Self {
        self.config.mdns_enabled = enabled;
        self
    }

    /// Enable or disable Kademlia
    pub fn kademlia(mut self, enabled: bool) -> Self {
        self.config.kademlia_enabled = enabled;
        self
    }

    /// Enable or disable Gossipsub
    pub fn gossipsub(mut self, enabled: bool) -> Self {
        self.config.gossipsub_enabled = enabled;
        self
    }

    /// Set connection timeout
    pub fn connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.connection_timeout = timeout;
        self
    }

    /// Set max connections per peer
    pub fn max_connections_per_peer(mut self, max: u32) -> Self {
        self.config.max_connections_per_peer = Some(max);
        self
    }

    /// Build the configuration
    pub fn build(self) -> NetworkConfig {
        self.config
    }
}

/// Information about a connected peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub connected_address: Option<Multiaddr>,
    pub connection_established: chrono::DateTime<chrono::Utc>,
    pub user_agent: Option<String>,
    pub protocol_version: Option<String>,
    pub supported_protocols: Vec<String>,
    pub is_connected: bool,
    pub connection_count: usize,
}

/// Network statistics and status information
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub local_peer_id: PeerId,
    pub listening_addresses: Vec<Multiaddr>,
    pub connected_peers: usize,
    pub pending_connections: usize,
    pub total_connections_established: u64,
    pub total_connections_closed: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub uptime: std::time::Duration,
    pub dht_routing_table_size: usize,
    pub gossipsub_topics: Vec<String>,
}

/// Events that can occur in the network
#[derive(Debug, Clone)]
pub enum NetworkEvent<K: NetabaseRegistryKey, V: NetabaseRegistery> {
    /// A new peer has been discovered
    PeerDiscovered { peer_info: PeerInfo },

    /// A peer has connected
    PeerConnected { peer_info: PeerInfo },

    /// A peer has disconnected
    PeerDisconnected {
        peer_id: PeerId,
        reason: Option<String>,
    },

    /// New listening address has been added
    NewListenAddr { address: Multiaddr },

    /// Listening address has expired
    ExpiredListenAddr { address: Multiaddr },

    /// A message has been received from a peer
    MessageReceived {
        peer_id: PeerId,
        message: NetworkMessage<K, V>,
    },

    /// A message broadcast was successful
    MessageBroadcasted {
        topic: String,
        message_size: usize,
        peer_count: usize,
    },

    /// DHT record was stored
    DhtRecordStored { key: String },

    /// DHT record was retrieved
    DhtRecordRetrieved { key: String },

    /// Bootstrap completed
    BootstrapCompleted { connected_peers: usize },

    /// Network error occurred
    NetworkError { error: NetworkError },
}

/// Types of messages that can be sent over the network
#[derive(Debug, Clone)]
pub enum NetworkMessage<K: NetabaseRegistryKey, V: NetabaseRegistery> {
    /// Request to store a value
    StoreRequest { key: K, value: V },

    /// Response to a store request
    StoreResponse { success: bool, message: String },

    /// Request to retrieve a value
    GetRequest { key: K },

    /// Response to a get request
    GetResponse { key: K, value: Option<V> },

    /// Announcement of a new value
    ValueAnnouncement { key: K, value: V },

    /// Heartbeat message
    Heartbeat { timestamp: u64 },

    /// Custom application message
    Custom { data: Vec<u8> },
}

/// Options for broadcasting messages
#[derive(Debug, Clone, Default)]
pub struct BroadcastOptions {
    pub topic: Option<String>,
    pub ttl: Option<std::time::Duration>,
    pub min_peers: Option<usize>,
    pub max_peers: Option<usize>,
    pub priority: MessagePriority,
}

/// Priority levels for messages
#[derive(Debug, Clone, Default)]
pub enum MessagePriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Core network operations trait
#[async_trait]
pub trait NetabaseNetwork<K: NetabaseRegistryKey, V: NetabaseRegistery>: Send + Sync {
    /// Initialize the network with the given configuration
    async fn initialize(&mut self, config: NetworkConfig) -> NetworkResult<()>;

    /// Check if the network is initialized
    fn is_initialized(&self) -> bool;

    /// Start the network services
    async fn start(&mut self) -> NetworkResult<()>;

    /// Stop the network services
    async fn stop(&mut self) -> NetworkResult<()>;

    /// Check if the network is running
    fn is_running(&self) -> bool;

    /// Get the local peer ID
    fn local_peer_id(&self) -> NetworkResult<PeerId>;

    /// Get current listening addresses
    fn listening_addresses(&self) -> NetworkResult<Vec<Multiaddr>>;

    /// Add a new listening address
    async fn add_listening_address(&mut self, address: Multiaddr) -> NetworkResult<()>;

    /// Remove a listening address
    async fn remove_listening_address(&mut self, address: &Multiaddr) -> NetworkResult<bool>;

    /// Connect to a specific peer
    async fn connect_peer(&mut self, peer_id: PeerId, address: Multiaddr) -> NetworkResult<()>;

    /// Disconnect from a specific peer
    async fn disconnect_peer(&mut self, peer_id: &PeerId) -> NetworkResult<()>;

    /// Get information about connected peers
    async fn connected_peers(&self) -> NetworkResult<Vec<PeerInfo>>;

    /// Get information about a specific peer
    async fn peer_info(&self, peer_id: &PeerId) -> NetworkResult<Option<PeerInfo>>;

    /// Send a direct message to a specific peer
    async fn send_message(
        &mut self,
        peer_id: &PeerId,
        message: NetworkMessage<K, V>,
    ) -> NetworkResult<()>;

    /// Broadcast a message to all connected peers
    async fn broadcast_message(
        &mut self,
        message: NetworkMessage<K, V>,
        options: BroadcastOptions,
    ) -> NetworkResult<()>;

    /// Subscribe to a gossipsub topic
    async fn subscribe_topic(&mut self, topic: &str) -> NetworkResult<()>;

    /// Unsubscribe from a gossipsub topic
    async fn unsubscribe_topic(&mut self, topic: &str) -> NetworkResult<()>;

    /// Get list of subscribed topics
    fn subscribed_topics(&self) -> NetworkResult<Vec<String>>;

    /// Publish a message to a gossipsub topic
    async fn publish_topic(&mut self, topic: &str, data: Vec<u8>) -> NetworkResult<()>;

    /// Store a record in the DHT
    async fn dht_put(&mut self, key: String, value: Vec<u8>) -> NetworkResult<()>;

    /// Retrieve a record from the DHT
    async fn dht_get(&mut self, key: &str) -> NetworkResult<Option<Vec<u8>>>;

    /// Add an address for a peer to the DHT
    async fn dht_add_address(&mut self, peer_id: PeerId, address: Multiaddr) -> NetworkResult<()>;

    /// Get addresses for a peer from the DHT
    async fn dht_get_addresses(&mut self, peer_id: &PeerId) -> NetworkResult<Vec<Multiaddr>>;

    /// Bootstrap the DHT with configured bootstrap nodes
    async fn bootstrap(&mut self) -> NetworkResult<()>;

    /// Get network statistics
    async fn stats(&self) -> NetworkResult<NetworkStats>;

    /// Set Kademlia DHT mode
    async fn set_dht_mode(&mut self, mode: KademliaDhtMode) -> NetworkResult<()>;

    /// Get current Kademlia DHT mode
    fn get_dht_mode(&self) -> NetworkResult<KademliaDhtMode>;

    /// Check if DHT is in server mode
    fn is_dht_server(&self) -> NetworkResult<bool>;

    /// Check if DHT is in client mode
    fn is_dht_client(&self) -> NetworkResult<bool>;

    /// Get network events receiver
    fn event_receiver(&self)
    -> NetworkResult<tokio::sync::broadcast::Receiver<NetworkEvent<K, V>>>;
}

/// Extension trait for advanced network operations
#[async_trait]
pub trait NetabaseNetworkExt<K: NetabaseRegistryKey, V: NetabaseRegistery>:
    NetabaseNetwork<K, V>
{
    /// Discover peers using mDNS
    async fn discover_mdns_peers(&mut self) -> NetworkResult<Vec<PeerInfo>>;

    /// Set custom protocols for the node
    async fn set_custom_protocols(&mut self, protocols: Vec<String>) -> NetworkResult<()>;

    /// Get the closest peers to a key in the DHT
    async fn dht_get_closest_peers(&mut self, key: &str) -> NetworkResult<Vec<PeerId>>;

    /// Perform a DHT query to find providers of a key
    async fn dht_get_providers(&mut self, key: &str) -> NetworkResult<Vec<PeerId>>;

    /// Announce that this node provides a key
    async fn dht_start_providing(&mut self, key: &str) -> NetworkResult<()>;

    /// Stop providing a key
    async fn dht_stop_providing(&mut self, key: &str) -> NetworkResult<()>;

    /// Ban a peer (prevent connections)
    async fn ban_peer(
        &mut self,
        peer_id: &PeerId,
        duration: std::time::Duration,
    ) -> NetworkResult<()>;

    /// Unban a peer
    async fn unban_peer(&mut self, peer_id: &PeerId) -> NetworkResult<()>;

    /// Get list of banned peers
    fn banned_peers(&self) -> NetworkResult<Vec<PeerId>>;

    /// Set connection limits
    async fn set_connection_limits(
        &mut self,
        max_connections: Option<u32>,
        max_pending: Option<u32>,
    ) -> NetworkResult<()>;

    /// Get current connection limits
    fn connection_limits(&self) -> NetworkResult<(Option<u32>, Option<u32>)>;

    /// Enable or disable specific network protocols
    async fn configure_protocols(&mut self, config: ProtocolConfig) -> NetworkResult<()>;

    /// Toggle DHT server/client mode based on conditions
    async fn toggle_dht_mode_auto(&mut self) -> NetworkResult<KademliaDhtMode>;

    /// Force DHT into server mode
    async fn force_dht_server_mode(&mut self) -> NetworkResult<()>;

    /// Force DHT into client mode
    async fn force_dht_client_mode(&mut self) -> NetworkResult<()>;

    /// Get DHT mode statistics
    async fn get_dht_mode_stats(&self) -> NetworkResult<DhtModeStats>;

    /// Monitor network health
    async fn health_check(&self) -> NetworkResult<NetworkHealth>;
}

/// Configuration for individual network protocols
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    pub mdns_enabled: Option<bool>,
    pub gossipsub_enabled: Option<bool>,
    pub kademlia_enabled: Option<bool>,
    pub kademlia_dht_mode: Option<KademliaDhtMode>,
    pub identify_enabled: Option<bool>,
    pub ping_enabled: Option<bool>,
    pub custom_protocols: Vec<String>,
}

/// DHT mode statistics
#[derive(Debug, Clone)]
pub struct DhtModeStats {
    pub current_mode: KademliaDhtMode,
    pub mode_switches_count: u64,
    pub time_in_server_mode: std::time::Duration,
    pub time_in_client_mode: std::time::Duration,
    pub records_stored: usize,
    pub queries_answered: u64,
    pub auto_mode_triggers: Vec<String>,
}

/// Network health status
#[derive(Debug, Clone)]
pub struct NetworkHealth {
    pub is_healthy: bool,
    pub connected_peer_count: usize,
    pub min_required_peers: usize,
    pub bootstrap_status: BootstrapStatus,
    pub dht_status: DhtStatus,
    pub dht_mode: KademliaDhtMode,
    pub issues: Vec<String>,
}

/// Bootstrap status
#[derive(Debug, Clone)]
pub enum BootstrapStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed { reason: String },
}

/// DHT status
#[derive(Debug, Clone)]
pub enum DhtStatus {
    NotInitialized,
    Initializing,
    Ready,
    ServerMode,
    ClientMode,
    SwitchingMode {
        from: KademliaDhtMode,
        to: KademliaDhtMode,
    },
    Error {
        message: String,
    },
}

/// Trait for handling network events
#[async_trait]
pub trait NetworkEventHandler<
    K: NetabaseRegistryKey + Send + Sync + 'static,
    V: NetabaseRegistery + Send + Sync + 'static,
>: Send + Sync
{
    /// Handle a network event
    async fn handle_event(&mut self, event: NetworkEvent<K, V>) -> NetworkResult<()>;

    /// Handle peer connection
    async fn on_peer_connected(&mut self, peer_info: PeerInfo) -> NetworkResult<()> {
        Ok(())
    }

    /// Handle peer disconnection
    async fn on_peer_disconnected(
        &mut self,
        peer_id: PeerId,
        reason: Option<String>,
    ) -> NetworkResult<()> {
        Ok(())
    }

    /// Handle incoming message
    async fn on_message_received(
        &mut self,
        peer_id: PeerId,
        message: NetworkMessage<K, V>,
    ) -> NetworkResult<()> {
        Ok(())
    }

    /// Handle network error
    async fn on_network_error(&mut self, error: NetworkError) -> NetworkResult<()> {
        Ok(())
    }
}
