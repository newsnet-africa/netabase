use libp2p::{Multiaddr, core::upgrade::Version};
use std::{num::NonZero, path::PathBuf, time::Duration};

/// Configuration for the Swarm
#[derive(Clone, Debug)]
pub struct SwarmConfig {
    /// How long to keep a connection alive once it is idling
    pub connection_idle_timeout: Duration,
    /// Number of events from the NetworkBehaviour to the ConnectionHandler that can be buffered
    pub notify_handler_buffer_size: NonZero<usize>,
    /// Size of the buffer for events sent by a ConnectionHandler to the NetworkBehaviour
    pub per_connection_event_buffer_size: usize,
    /// Number of addresses concurrently dialed for a single outbound connection attempt
    pub dial_concurrency_factor: NonZero<u8>,
    /// Override for the substream upgrade protocol to use
    pub substream_upgrade_protocol_override: Option<Version>,
    /// Maximum number of inbound streams concurrently negotiating on a connection
    pub max_negotiating_inbound_streams: usize,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            connection_idle_timeout: Duration::from_secs(10), // libp2p default
            notify_handler_buffer_size: NonZero::new(8).unwrap(),
            per_connection_event_buffer_size: 7,
            dial_concurrency_factor: NonZero::new(8).unwrap(),
            substream_upgrade_protocol_override: None,
            max_negotiating_inbound_streams: 128,
        }
    }
}

impl SwarmConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_connection_idle_timeout(mut self, timeout: Duration) -> Self {
        self.connection_idle_timeout = timeout;
        self
    }

    pub fn with_notify_handler_buffer_size(mut self, size: NonZero<usize>) -> Self {
        self.notify_handler_buffer_size = size;
        self
    }

    pub fn with_per_connection_event_buffer_size(mut self, size: usize) -> Self {
        self.per_connection_event_buffer_size = size;
        self
    }

    pub fn with_dial_concurrency_factor(mut self, factor: NonZero<u8>) -> Self {
        self.dial_concurrency_factor = factor;
        self
    }

    pub fn with_substream_upgrade_protocol_override(mut self, version: Version) -> Self {
        self.substream_upgrade_protocol_override = Some(version);
        self
    }

    pub fn with_max_negotiating_inbound_streams(mut self, max: usize) -> Self {
        self.max_negotiating_inbound_streams = max;
        self
    }
}

/// Configuration for Kademlia DHT
#[derive(Clone, Debug)]
pub struct KademliaConfig {
    /// Kademlia replication factor (how many nodes to replicate data to)
    pub replication_factor: usize,
    /// Timeout for Kademlia queries
    pub query_timeout: Duration,
}

impl Default for KademliaConfig {
    fn default() -> Self {
        Self {
            replication_factor: 20,
            query_timeout: Duration::from_secs(60),
        }
    }
}

impl KademliaConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_replication_factor(mut self, factor: usize) -> Self {
        self.replication_factor = factor;
        self
    }

    pub fn with_query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }
}

/// Configuration for Identify protocol
#[derive(Clone, Debug)]
pub struct IdentifyConfig {
    /// Agent version to send to peers
    pub agent_version: String,
    /// Protocol version
    pub protocol_version: String,
    /// Interval at which identification requests are sent to peers after the initial request
    pub interval: Duration,
    /// Whether new or expired listen addresses should trigger an active push
    pub push_listen_addr_updates: bool,
    /// Size of the LRU cache for discovered peers
    pub cache_size: usize,
    /// Whether to prevent sending out our listen addresses
    pub hide_listen_addrs: bool,
}

impl Default for IdentifyConfig {
    fn default() -> Self {
        Self {
            agent_version: format!("netabase/{}", env!("CARGO_PKG_VERSION")),
            protocol_version: "netabase/1.0.0".to_string(),
            interval: Duration::from_secs(300), // 5 minutes
            push_listen_addr_updates: true,
            cache_size: 100,
            hide_listen_addrs: false,
        }
    }
}

impl IdentifyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_agent_version(mut self, version: String) -> Self {
        self.agent_version = version;
        self
    }

    pub fn with_protocol_version(mut self, version: String) -> Self {
        self.protocol_version = version;
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_push_listen_addr_updates(mut self, enabled: bool) -> Self {
        self.push_listen_addr_updates = enabled;
        self
    }

    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }

    pub fn with_hide_listen_addrs(mut self, hide: bool) -> Self {
        self.hide_listen_addrs = hide;
        self
    }
}

/// Configuration for mDNS
#[derive(Clone, Debug)]
pub struct MdnsConfig {
    /// Enable mDNS discovery
    pub enabled: bool,
    /// TTL to use for mdns records
    pub ttl: Duration,
    /// Interval at which to poll the network for new peers
    pub query_interval: Duration,
    /// Use IPv6 instead of IPv4
    pub enable_ipv6: bool,
}

impl Default for MdnsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: Duration::from_secs(600),           // 10 minutes
            query_interval: Duration::from_secs(60), // 1 minute
            enable_ipv6: false,
        }
    }
}

impl MdnsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_query_interval(mut self, interval: Duration) -> Self {
        self.query_interval = interval;
        self
    }

    pub fn with_ipv6_enabled(mut self, enabled: bool) -> Self {
        self.enable_ipv6 = enabled;
        self
    }
}

/// Main configuration for Netabase
#[derive(Clone, Debug)]
pub struct NetabaseConfig {
    /// Path where database files will be stored
    pub storage_path: PathBuf,
    /// Path where the keypair will be stored
    pub keypair_path: PathBuf,
    /// Addresses to listen on
    pub listen_addresses: Vec<Multiaddr>,
    /// Bootstrap peer addresses to connect to
    pub bootstrap_addresses: Vec<Multiaddr>,
    /// Swarm configuration
    pub swarm: SwarmConfig,
    /// Kademlia configuration
    pub kademlia: KademliaConfig,
    /// Identify protocol configuration
    pub identify: IdentifyConfig,
    /// mDNS configuration
    pub mdns: MdnsConfig,
}

impl Default for NetabaseConfig {
    fn default() -> Self {
        Self {
            storage_path: std::env::temp_dir().join("netabase"),
            keypair_path: std::env::temp_dir().join("netabase_keypair"),
            listen_addresses: vec![],
            bootstrap_addresses: vec![],
            swarm: SwarmConfig::default(),
            kademlia: KademliaConfig::default(),
            identify: IdentifyConfig::default(),
            mdns: MdnsConfig::default(),
        }
    }
}

impl NetabaseConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = path;
        self
    }

    pub fn with_keypair_path(mut self, path: PathBuf) -> Self {
        self.keypair_path = path;
        self
    }

    pub fn with_listen_addresses(mut self, addresses: Vec<Multiaddr>) -> Self {
        self.listen_addresses = addresses;
        self
    }

    pub fn add_listen_address(mut self, address: Multiaddr) -> Self {
        self.listen_addresses.push(address);
        self
    }

    pub fn with_bootstrap_addresses(mut self, addresses: Vec<Multiaddr>) -> Self {
        self.bootstrap_addresses = addresses;
        self
    }

    pub fn add_bootstrap_address(mut self, address: Multiaddr) -> Self {
        self.bootstrap_addresses.push(address);
        self
    }

    pub fn with_swarm_config(mut self, config: SwarmConfig) -> Self {
        self.swarm = config;
        self
    }

    pub fn with_kademlia_config(mut self, config: KademliaConfig) -> Self {
        self.kademlia = config;
        self
    }

    pub fn with_identify_config(mut self, config: IdentifyConfig) -> Self {
        self.identify = config;
        self
    }

    pub fn with_mdns_config(mut self, config: MdnsConfig) -> Self {
        self.mdns = config;
        self
    }

    // Convenience methods for backward compatibility with the old flat structure
    pub fn with_kad_replication_factor(mut self, factor: usize) -> Self {
        self.kademlia.replication_factor = factor;
        self
    }

    pub fn with_kad_query_timeout(mut self, timeout: Duration) -> Self {
        self.kademlia.query_timeout = timeout;
        self
    }

    pub fn with_mdns_enabled(mut self, enabled: bool) -> Self {
        self.mdns.enabled = enabled;
        self
    }

    pub fn with_connection_idle_timeout(mut self, timeout: Duration) -> Self {
        self.swarm.connection_idle_timeout = timeout;
        self
    }

    pub fn with_notify_handler_buffer_size(mut self, size: NonZero<usize>) -> Self {
        self.swarm.notify_handler_buffer_size = size;
        self
    }

    pub fn with_per_connection_event_buffer_size(mut self, size: usize) -> Self {
        self.swarm.per_connection_event_buffer_size = size;
        self
    }

    pub fn with_dial_concurrency_factor(mut self, factor: NonZero<u8>) -> Self {
        self.swarm.dial_concurrency_factor = factor;
        self
    }

    pub fn with_substream_upgrade_protocol_override(mut self, version: Version) -> Self {
        self.swarm.substream_upgrade_protocol_override = Some(version);
        self
    }

    pub fn with_max_negotiating_inbound_streams(mut self, max: usize) -> Self {
        self.swarm.max_negotiating_inbound_streams = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;
    use libp2p::core::upgrade::Version;
    use libp2p::identity::Keypair;
    use std::num::NonZero;
    use std::time::Duration;

    #[test]
    fn test_swarm_config_default() {
        let config = SwarmConfig::default();
        assert_eq!(config.connection_idle_timeout, Duration::from_secs(10));
        assert_eq!(config.notify_handler_buffer_size, NonZero::new(8).unwrap());
        assert_eq!(config.per_connection_event_buffer_size, 7);
        assert_eq!(config.dial_concurrency_factor, NonZero::new(8).unwrap());
        assert_eq!(config.substream_upgrade_protocol_override, None);
        assert_eq!(config.max_negotiating_inbound_streams, 128);
    }

    #[test]
    fn test_swarm_config_builder() {
        let config = SwarmConfig::new()
            .with_connection_idle_timeout(Duration::from_secs(30))
            .with_notify_handler_buffer_size(NonZero::new(16).unwrap())
            .with_per_connection_event_buffer_size(10)
            .with_dial_concurrency_factor(NonZero::new(4).unwrap())
            .with_substream_upgrade_protocol_override(Version::V1)
            .with_max_negotiating_inbound_streams(256);

        assert_eq!(config.connection_idle_timeout, Duration::from_secs(30));
        assert_eq!(config.notify_handler_buffer_size, NonZero::new(16).unwrap());
        assert_eq!(config.per_connection_event_buffer_size, 10);
        assert_eq!(config.dial_concurrency_factor, NonZero::new(4).unwrap());
        assert_eq!(
            config.substream_upgrade_protocol_override,
            Some(Version::V1)
        );
        assert_eq!(config.max_negotiating_inbound_streams, 256);
    }

    #[test]
    fn test_kademlia_config_default() {
        let config = KademliaConfig::default();
        assert_eq!(config.replication_factor, 20);
        assert_eq!(config.query_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_kademlia_config_builder() {
        let config = KademliaConfig::new()
            .with_replication_factor(10)
            .with_query_timeout(Duration::from_secs(30));

        assert_eq!(config.replication_factor, 10);
        assert_eq!(config.query_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_identify_config_default() {
        let config = IdentifyConfig::default();
        assert_eq!(
            config.agent_version,
            format!("netabase/{}", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(config.protocol_version, "netabase/1.0.0");
        assert_eq!(config.interval, Duration::from_secs(300));
        assert_eq!(config.push_listen_addr_updates, true);
        assert_eq!(config.cache_size, 100);
        assert_eq!(config.hide_listen_addrs, false);
    }

    #[test]
    fn test_identify_config_builder() {
        let config = IdentifyConfig::new()
            .with_agent_version("custom-agent/1.0.0".to_string())
            .with_protocol_version("custom-protocol/2.0.0".to_string())
            .with_interval(Duration::from_secs(600))
            .with_push_listen_addr_updates(false)
            .with_cache_size(200)
            .with_hide_listen_addrs(true);

        assert_eq!(config.agent_version, "custom-agent/1.0.0");
        assert_eq!(config.protocol_version, "custom-protocol/2.0.0");
        assert_eq!(config.interval, Duration::from_secs(600));
        assert_eq!(config.push_listen_addr_updates, false);
        assert_eq!(config.cache_size, 200);
        assert_eq!(config.hide_listen_addrs, true);
    }

    #[test]
    fn test_mdns_config_default() {
        let config = MdnsConfig::default();
        assert_eq!(config.enabled, true);
        assert_eq!(config.ttl, Duration::from_secs(600));
        assert_eq!(config.query_interval, Duration::from_secs(60));
        assert_eq!(config.enable_ipv6, false);
    }

    #[test]
    fn test_mdns_config_builder() {
        let config = MdnsConfig::new()
            .with_enabled(false)
            .with_ttl(Duration::from_secs(300))
            .with_query_interval(Duration::from_secs(120))
            .with_ipv6_enabled(true);

        assert_eq!(config.enabled, false);
        assert_eq!(config.ttl, Duration::from_secs(300));
        assert_eq!(config.query_interval, Duration::from_secs(120));
        assert_eq!(config.enable_ipv6, true);
    }

    #[test]
    fn test_netabase_config_default() {
        let config = NetabaseConfig::default();
        assert_eq!(config.storage_path, std::env::temp_dir().join("netabase"));
        assert_eq!(
            config.keypair_path,
            std::env::temp_dir().join("netabase_keypair")
        );
        assert!(config.listen_addresses.is_empty());
        assert!(config.bootstrap_addresses.is_empty());

        // Test that nested configs are properly initialized
        assert_eq!(
            config.swarm.connection_idle_timeout,
            Duration::from_secs(10)
        );
        assert_eq!(config.kademlia.replication_factor, 20);
        assert_eq!(
            config.identify.agent_version,
            format!("netabase/{}", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(config.mdns.enabled, true);
    }

    #[test]
    fn test_netabase_config_builder() {
        let storage_path = PathBuf::from("/custom/storage");
        let keypair_path = PathBuf::from("/custom/keypair");
        let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let keypair = PeerId::random();
        let mut str_addr = String::from("/ip4/192.168.1.100/tcp/4001/p2p/");
        str_addr.push_str(&keypair.to_string());
        let addy: Multiaddr = str_addr.parse().unwrap();

        let config = NetabaseConfig::new()
            .with_storage_path(storage_path.clone())
            .with_keypair_path(keypair_path.clone())
            .add_listen_address(listen_addr.clone())
            .add_bootstrap_address(addy.clone())
            .with_swarm_config(
                SwarmConfig::new().with_connection_idle_timeout(Duration::from_secs(60)),
            )
            .with_kademlia_config(KademliaConfig::new().with_replication_factor(10))
            .with_mdns_config(MdnsConfig::new().with_enabled(false));

        assert_eq!(config.storage_path, storage_path);
        assert_eq!(config.keypair_path, keypair_path);
        assert_eq!(config.listen_addresses, vec![listen_addr]);
        assert_eq!(config.bootstrap_addresses, vec![addy]);
        assert_eq!(
            config.swarm.connection_idle_timeout,
            Duration::from_secs(60)
        );
        assert_eq!(config.kademlia.replication_factor, 10);
        assert!(!config.mdns.enabled);
    }

    #[test]
    fn test_netabase_config_convenience_methods() {
        let config = NetabaseConfig::new()
            .with_kad_replication_factor(15)
            .with_kad_query_timeout(Duration::from_secs(45))
            .with_mdns_enabled(false)
            .with_connection_idle_timeout(Duration::from_secs(20))
            .with_notify_handler_buffer_size(NonZero::new(12).unwrap())
            .with_per_connection_event_buffer_size(15)
            .with_dial_concurrency_factor(NonZero::new(6).unwrap())
            .with_substream_upgrade_protocol_override(Version::V1)
            .with_max_negotiating_inbound_streams(512);

        // Test that convenience methods properly set nested config values
        assert_eq!(config.kademlia.replication_factor, 15);
        assert_eq!(config.kademlia.query_timeout, Duration::from_secs(45));
        assert_eq!(config.mdns.enabled, false);
        assert_eq!(
            config.swarm.connection_idle_timeout,
            Duration::from_secs(20)
        );
        assert_eq!(
            config.swarm.notify_handler_buffer_size,
            NonZero::new(12).unwrap()
        );
        assert_eq!(config.swarm.per_connection_event_buffer_size, 15);
        assert_eq!(
            config.swarm.dial_concurrency_factor,
            NonZero::new(6).unwrap()
        );
        assert_eq!(
            config.swarm.substream_upgrade_protocol_override,
            Some(Version::V1)
        );
        assert_eq!(config.swarm.max_negotiating_inbound_streams, 512);
    }

    #[test]
    fn test_config_cloning() {
        let original = NetabaseConfig::new()
            .with_storage_path(PathBuf::from("/test"))
            .with_kad_replication_factor(25);

        let cloned = original.clone();

        assert_eq!(original.storage_path, cloned.storage_path);
        assert_eq!(
            original.kademlia.replication_factor,
            cloned.kademlia.replication_factor
        );
    }

    #[test]
    fn test_with_listen_addresses_replaces_all() {
        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let addr2: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
        let addr3: Multiaddr = "/ip6/::1/tcp/0".parse().unwrap();

        let config = NetabaseConfig::new()
            .add_listen_address(addr1)
            .with_listen_addresses(vec![addr2.clone(), addr3.clone()]);

        assert_eq!(config.listen_addresses, vec![addr2, addr3]);
    }

    #[test]
    fn test_with_bootstrap_addresses_replaces_all() {
        let keypair = PeerId::random();
        let mut str_addr = String::from("/ip4/192.168.1.100/tcp/4001/p2p/");
        str_addr.push_str(&keypair.to_string());
        let addr1: Multiaddr = str_addr.parse().unwrap();
        let keypair = PeerId::random();
        let mut str_addr = String::from("/ip4/192.168.1.100/tcp/4001/p2p/");
        str_addr.push_str(&keypair.to_string());
        let addr2: Multiaddr = str_addr.parse().unwrap();
        let keypair = PeerId::random();
        let mut str_addr = String::from("/ip4/192.168.1.100/tcp/4001/p2p/");
        str_addr.push_str(&keypair.to_string());
        let addr3: Multiaddr = str_addr.parse().unwrap();

        let config = NetabaseConfig::new()
            .add_bootstrap_address(addr1)
            .with_bootstrap_addresses(vec![addr2.clone(), addr3.clone()]);

        assert_eq!(config.bootstrap_addresses, vec![addr2, addr3]);
    }
}
