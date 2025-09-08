//! Configuration module for Netabase
//!
//! This module provides comprehensive configuration structures for both the libp2p swarm
//! and the network behaviors (Kademlia DHT, Identify, and mDNS).
//!
//! # Examples
//!
//! ## Basic Configuration
//!
//! ```rust
//! use netabase::config::{NetabaseConfig, BehaviourConfig, NetabaseSwarmConfig};
//! use std::time::Duration;
//!
//! // Create a basic configuration with defaults
//! let config = NetabaseConfig::default();
//!
//! // Use the configuration to create a swarm
//! // let swarm = generate_swarm_with_config(&config)?;
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust
//! use netabase::config::{NetabaseConfig, BehaviourConfig, NetabaseSwarmConfig};
//! use libp2p::{kad, identify, mdns};
//! use std::time::Duration;
//!
//! // Create custom behavior configuration
//! let mut kad_config = kad::Config::default();
//! kad_config.set_query_timeout(Duration::from_secs(60));
//!
//! let identify_config = identify::Config::new("/myapp/1.0.0".to_string(), keypair.public())
//!     .with_agent_version("MyApp/1.0.0".to_string());
//!
//! let behaviour_config = BehaviourConfig::builder()
//!     .kad_config(Some(kad_config))
//!     .identify_config(Some(identify_config))
//!     .protocol_version("/myapp/1.0.0".to_string())
//!     .database_path("./custom_db".to_string())
//!     .build()
//!     .expect("Valid behaviour config");
//!
//! // Create custom swarm configuration
//! let swarm_config = NetabaseSwarmConfig::builder()
//!     .connection_timeout(Duration::from_secs(45))
//!     .dns(true)
//!     .quic_enabled(true)
//!     .mdns_enabled(true)
//!     .max_connections_per_peer(Some(3))
//!     .build()
//!     .expect("Valid swarm config");
//!
//! // Combine into main configuration
//! let config = NetabaseConfig::builder()
//!     .swarm_config(swarm_config)
//!     .behaviour_config(behaviour_config)
//!     .build()
//!     .expect("Valid netabase config");
//! ```
//!
//! ## Production Configuration Example
//!
//! ```rust
//! use netabase::config::{NetabaseConfig, BehaviourConfig, NetabaseSwarmConfig};
//! use libp2p::{kad, identify, mdns, identity::Keypair};
//! use std::time::Duration;
//!
//! // Production-ready configuration
//! let keypair = Keypair::generate_ed25519();
//!
//! let mut kad_config = kad::Config::default();
//! kad_config.set_query_timeout(Duration::from_secs(120));
//! kad_config.set_replication_factor(20.try_into().unwrap());
//!
//! let behaviour_config = BehaviourConfig::builder()
//!     .kad_config(Some(kad_config))
//!     .protocol_version("/newsnet/1.0.0".to_string())
//!     .agent_version("NewsNet/1.0.0".to_string())
//!     .database_path("./production_db".to_string())
//!     .build()
//!     .unwrap();
//!
//! let swarm_config = NetabaseSwarmConfig::builder()
//!     .identity(Some(keypair))
//!     .connection_timeout(Duration::from_secs(30))
//!     .idle_connection_timeout(Duration::from_secs(60))
//!     .max_connections_per_peer(Some(5))
//!     .max_pending_connections(Some(512))
//!     .quic_enabled(true)
//!     .mdns_enabled(false) // Disable mDNS in production
//!     .listen_addresses(vec![
//!         "/ip4/0.0.0.0/tcp/4001".parse().unwrap(),
//!         "/ip4/0.0.0.0/udp/4001/quic-v1".parse().unwrap(),
//!     ])
//!     .build()
//!     .unwrap();
//!
//! let config = NetabaseConfig::builder()
//!     .swarm_config(swarm_config)
//!     .behaviour_config(behaviour_config)
//!     .build()
//!     .unwrap();
//! ```

use derive_builder::Builder;
use libp2p::{Multiaddr, identify, identity::Keypair, kad, mdns, noise, quic, tcp, yamux};
use std::time::Duration;

/// Main configuration structure for Netabase
///
/// This structure contains all configuration options for both the libp2p swarm
/// and the network behaviors used by Netabase.
#[derive(Builder, Clone, Default)]
pub struct NetabaseConfig {
    /// Configuration for the libp2p swarm
    swarm_config: NetabaseSwarmConfig,
    /// Configuration for network behaviors (Kademlia, Identify, mDNS)
    behaviour_config: BehaviourConfig,
}

/// Configuration for the libp2p swarm
///
/// This structure contains all configuration options that affect the libp2p swarm
/// behavior, including transport settings, connection limits, and network addresses.
#[derive(Builder, Clone)]
pub struct NetabaseSwarmConfig {
    /// Connection timeout for establishing connections
    #[builder(default = "Duration::from_secs(30)")]
    connection_timeout: Duration,

    /// Enable DNS resolution
    #[builder(default = "true")]
    dns: bool,

    /// Optional identity keypair for the node
    #[builder(default = "None")]
    identity: Option<Keypair>,

    /// TCP transport configuration
    #[builder(default = "tcp::Config::default()")]
    tcp_config: tcp::Config,

    /// Noise protocol configuration for encryption
    #[builder(default)]
    noise_config: Option<noise::Config>,

    /// Yamux multiplexing configuration
    #[builder(default = "yamux::Config::default()")]
    yamux_config: yamux::Config,

    /// Enable QUIC transport
    #[builder(default = "true")]
    quic_enabled: bool,

    /// QUIC transport configuration
    #[builder(default)]
    quic_config: Option<quic::Config>,

    /// Enable mDNS for local peer discovery
    #[builder(default = "true")]
    mdns_enabled: bool,

    /// Enable relay functionality
    #[builder(default = "false")]
    relay_enabled: bool,

    /// External addresses to announce
    #[builder(default = "Vec::new()")]
    external_addresses: Vec<Multiaddr>,

    /// Addresses to listen on
    #[builder(default = "vec![\"/ip4/0.0.0.0/tcp/0\".parse().unwrap()]")]
    listen_addresses: Vec<Multiaddr>,

    /// Maximum number of concurrent inbound streams being negotiated
    #[builder(default = "Some(256)")]
    max_negotiating_inbound_streams: Option<usize>,

    /// Maximum number of established connections per peer
    #[builder(default = "Some(1)")]
    max_connections_per_peer: Option<u32>,

    /// Maximum number of pending connections
    #[builder(default = "Some(256)")]
    max_pending_connections: Option<u32>,

    /// Connection idle timeout
    #[builder(default = "Duration::from_secs(30)")]
    idle_connection_timeout: Duration,

    /// Enable connection limits
    #[builder(default = "true")]
    connection_limits_enabled: bool,

    /// Bootstrap nodes to connect to
    #[builder(default = "Vec::new()")]
    bootstrap_nodes: Vec<(String, Multiaddr)>,

    /// Custom user agent string
    #[builder(default = "\"netabase/0.1.0\".to_string()")]
    user_agent: String,
}

/// Configuration for network behaviors
///
/// This structure contains configuration options for the various libp2p behaviors
/// used by Netabase: Kademlia DHT, Identify protocol, and mDNS discovery.
#[derive(Builder, Clone)]
pub struct BehaviourConfig {
    /// Kademlia DHT configuration
    #[builder(default)]
    kad_config: Option<kad::Config>,

    /// Identify protocol configuration
    #[builder(default)]
    identify_config: Option<identify::Config>,

    /// mDNS configuration for local peer discovery
    #[builder(default)]
    mdns_config: Option<mdns::Config>,

    /// Protocol version string for identify
    #[builder(default = "\"/p2p/newsnet/0.1.0\".to_string()")]
    protocol_version: String,

    /// Agent version string for identify
    #[builder(default = "\"netabase/0.1.0\".to_string()")]
    agent_version: String,

    /// Database path for Kademlia store
    #[builder(default = "\"./database\".to_string()")]
    database_path: String,
}

impl Default for NetabaseSwarmConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            dns: true,
            identity: None,
            tcp_config: tcp::Config::default(),
            noise_config: None,
            yamux_config: yamux::Config::default(),
            quic_enabled: true,
            quic_config: None,
            mdns_enabled: true,
            relay_enabled: false,
            external_addresses: Vec::new(),
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()],
            max_negotiating_inbound_streams: Some(256),
            max_connections_per_peer: Some(1),
            max_pending_connections: Some(256),
            idle_connection_timeout: Duration::from_secs(30),
            connection_limits_enabled: true,
            bootstrap_nodes: Vec::new(),
            user_agent: "netabase/0.1.0".to_string(),
        }
    }
}

impl Default for BehaviourConfig {
    fn default() -> Self {
        Self {
            kad_config: None,
            identify_config: None,
            mdns_config: None,
            protocol_version: "/p2p/newsnet/0.1.0".to_string(),
            agent_version: "netabase/0.1.0".to_string(),
            database_path: "./database".to_string(),
        }
    }
}

impl NetabaseSwarmConfig {
    /// Create a new builder for NetabaseSwarmConfig
    pub fn builder() -> NetabaseSwarmConfigBuilder {
        NetabaseSwarmConfigBuilder::default()
    }

    /// Get connection timeout
    pub fn connection_timeout(&self) -> Duration {
        self.connection_timeout
    }

    /// Check if DNS is enabled
    pub fn dns_enabled(&self) -> bool {
        self.dns
    }

    /// Get the identity keypair
    pub fn identity(&self) -> &Option<Keypair> {
        &self.identity
    }

    /// Get TCP configuration
    pub fn tcp_config(&self) -> &tcp::Config {
        &self.tcp_config
    }

    /// Get Yamux configuration
    pub fn yamux_config(&self) -> &yamux::Config {
        &self.yamux_config
    }

    /// Check if QUIC is enabled
    pub fn quic_enabled(&self) -> bool {
        self.quic_enabled
    }

    /// Check if mDNS is enabled
    pub fn mdns_enabled(&self) -> bool {
        self.mdns_enabled
    }

    /// Check if relay is enabled
    pub fn relay_enabled(&self) -> bool {
        self.relay_enabled
    }

    /// Get external addresses
    pub fn external_addresses(&self) -> &[Multiaddr] {
        &self.external_addresses
    }

    /// Get listen addresses
    pub fn listen_addresses(&self) -> &[Multiaddr] {
        &self.listen_addresses
    }

    /// Get bootstrap nodes
    pub fn bootstrap_nodes(&self) -> &[(String, Multiaddr)] {
        &self.bootstrap_nodes
    }

    /// Get user agent string
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Get idle connection timeout
    pub fn idle_connection_timeout(&self) -> Duration {
        self.idle_connection_timeout
    }

    /// Get maximum negotiating inbound streams
    pub fn max_negotiating_inbound_streams(&self) -> Option<usize> {
        self.max_negotiating_inbound_streams
    }

    /// Get maximum connections per peer
    pub fn max_connections_per_peer(&self) -> Option<u32> {
        self.max_connections_per_peer
    }

    /// Get maximum pending connections
    pub fn max_pending_connections(&self) -> Option<u32> {
        self.max_pending_connections
    }
}

impl NetabaseConfig {
    /// Create a new builder for NetabaseConfig
    pub fn builder() -> NetabaseConfigBuilder {
        NetabaseConfigBuilder::default()
    }

    /// Get swarm configuration
    pub fn swarm_config(&self) -> &NetabaseSwarmConfig {
        &self.swarm_config
    }

    /// Get behaviour configuration
    pub fn behaviour_config(&self) -> &BehaviourConfig {
        &self.behaviour_config
    }
}

impl BehaviourConfig {
    /// Create a new builder for BehaviourConfig
    pub fn builder() -> BehaviourConfigBuilder {
        BehaviourConfigBuilder::default()
    }

    /// Get Kademlia configuration
    pub fn kad_config(&self) -> &Option<kad::Config> {
        &self.kad_config
    }

    /// Get Identify configuration
    pub fn identify_config(&self) -> &Option<identify::Config> {
        &self.identify_config
    }

    /// Get mDNS configuration
    pub fn mdns_config(&self) -> &Option<mdns::Config> {
        &self.mdns_config
    }

    /// Get protocol version
    pub fn protocol_version(&self) -> &str {
        &self.protocol_version
    }

    /// Get agent version
    pub fn agent_version(&self) -> &str {
        &self.agent_version
    }

    /// Get database path
    pub fn database_path(&self) -> &str {
        &self.database_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::{identify, identity::Keypair, kad, mdns};
    use std::time::Duration;

    #[test]
    fn test_default_netabase_config() {
        let config = NetabaseConfig::default();

        // Test swarm config defaults
        assert_eq!(
            config.swarm_config().connection_timeout(),
            Duration::from_secs(30)
        );
        assert!(config.swarm_config().dns_enabled());
        assert!(config.swarm_config().quic_enabled());
        assert!(config.swarm_config().mdns_enabled());
        assert_eq!(config.swarm_config().user_agent(), "netabase/0.1.0");

        // Test behaviour config defaults
        assert_eq!(
            config.behaviour_config().protocol_version(),
            "/p2p/newsnet/0.1.0"
        );
        assert_eq!(config.behaviour_config().agent_version(), "netabase/0.1.0");
        assert_eq!(config.behaviour_config().database_path(), "./database");
    }

    #[test]
    fn test_netabase_config_builder() {
        let swarm_config = NetabaseSwarmConfig::builder()
            .connection_timeout(Duration::from_secs(45))
            .dns(false)
            .quic_enabled(true)
            .user_agent("test-agent/1.0.0".to_string())
            .build()
            .expect("Should build valid swarm config");

        let behaviour_config = BehaviourConfig::builder()
            .protocol_version("/test/1.0.0".to_string())
            .agent_version("TestAgent/1.0.0".to_string())
            .database_path("./test_db".to_string())
            .build()
            .expect("Should build valid behaviour config");

        let config = NetabaseConfig::builder()
            .swarm_config(swarm_config)
            .behaviour_config(behaviour_config)
            .build()
            .expect("Should build valid netabase config");

        assert_eq!(
            config.swarm_config().connection_timeout(),
            Duration::from_secs(45)
        );
        assert!(!config.swarm_config().dns_enabled());
        assert_eq!(config.swarm_config().user_agent(), "test-agent/1.0.0");
        assert_eq!(config.behaviour_config().protocol_version(), "/test/1.0.0");
        assert_eq!(config.behaviour_config().database_path(), "./test_db");
    }

    #[test]
    fn test_swarm_config_with_identity() {
        let keypair = Keypair::generate_ed25519();
        let config = NetabaseSwarmConfig::builder()
            .identity(Some(keypair.clone()))
            .connection_timeout(Duration::from_secs(60))
            .max_connections_per_peer(Some(5))
            .build()
            .expect("Should build valid swarm config with identity");

        assert!(config.identity().is_some());
        assert_eq!(config.connection_timeout(), Duration::from_secs(60));
        assert_eq!(config.max_connections_per_peer(), Some(5));
    }

    #[test]
    fn test_behaviour_config_with_custom_configs() {
        let mut kad_config = kad::Config::default();
        kad_config.set_query_timeout(Duration::from_secs(120));

        let keypair = Keypair::generate_ed25519();
        let identify_config = identify::Config::new("/custom/1.0.0".to_string(), keypair.public())
            .with_agent_version("CustomAgent/1.0.0".to_string());

        let mdns_config = mdns::Config::default();

        let behaviour_config = BehaviourConfig::builder()
            .kad_config(Some(kad_config))
            .identify_config(Some(identify_config))
            .mdns_config(Some(mdns_config))
            .protocol_version("/custom/1.0.0".to_string())
            .build()
            .expect("Should build valid behaviour config with custom configs");

        assert!(behaviour_config.kad_config().is_some());
        assert!(behaviour_config.identify_config().is_some());
        assert!(behaviour_config.mdns_config().is_some());
        assert_eq!(behaviour_config.protocol_version(), "/custom/1.0.0");
    }

    #[test]
    fn test_swarm_config_getters() {
        let config = NetabaseSwarmConfig::builder()
            .idle_connection_timeout(Duration::from_secs(90))
            .max_negotiating_inbound_streams(Some(512))
            .max_pending_connections(Some(1024))
            .build()
            .expect("Should build valid swarm config");

        assert_eq!(config.idle_connection_timeout(), Duration::from_secs(90));
        assert_eq!(config.max_negotiating_inbound_streams(), Some(512));
        assert_eq!(config.max_pending_connections(), Some(1024));
    }

    #[test]
    fn test_multiaddr_parsing() {
        let listen_addresses = vec![
            "/ip4/0.0.0.0/tcp/4001".parse().unwrap(),
            "/ip4/0.0.0.0/udp/4001/quic-v1".parse().unwrap(),
        ];

        let config = NetabaseSwarmConfig::builder()
            .listen_addresses(listen_addresses.clone())
            .build()
            .expect("Should build valid swarm config with listen addresses");

        assert_eq!(config.listen_addresses().len(), 2);
        assert_eq!(config.listen_addresses(), &listen_addresses[..]);
    }

    #[test]
    fn test_bootstrap_nodes_configuration() {
        let bootstrap_nodes = vec![
            (
                "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".to_string(),
                "/dnsaddr/bootstrap.libp2p.io".parse().unwrap(),
            ),
            (
                "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa".to_string(),
                "/dnsaddr/bootstrap.libp2p.io".parse().unwrap(),
            ),
        ];

        let config = NetabaseSwarmConfig::builder()
            .bootstrap_nodes(bootstrap_nodes.clone())
            .build()
            .expect("Should build valid swarm config with bootstrap nodes");

        assert_eq!(config.bootstrap_nodes().len(), 2);
        assert_eq!(config.bootstrap_nodes(), &bootstrap_nodes[..]);
    }
}
