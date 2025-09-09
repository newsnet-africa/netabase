use derive_builder::Builder;
use libp2p::{Multiaddr, PeerId, identify, identity::Keypair, kad, mdns, noise, quic, tcp, yamux};
use std::time::Duration;

#[derive(Builder, Clone)]
pub struct NetabaseConfig {
    swarm_config: NetabaseSwarmConfig,
    behaviour_config: BehaviourConfig,
}

pub type DefaultNetabaseConfig = NetabaseConfig;

impl NetabaseConfig {
    pub fn with_memory_store(peer_id: PeerId) -> NetabaseConfigBuilder {
        let mut builder = NetabaseConfig::builder();
        builder
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(BehaviourConfig::with_memory_store(peer_id).build().unwrap());
        builder
    }

    pub fn with_sled_store<P: AsRef<str>>(path: P) -> NetabaseConfigBuilder {
        let mut builder = NetabaseConfig::builder();
        builder
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(BehaviourConfig::with_sled_store(path).build().unwrap());
        builder
    }

    pub fn with_sled_store_config<P: AsRef<str>>(
        path: P,
        config: crate::database::SledStoreConfig,
    ) -> NetabaseConfigBuilder {
        let mut builder = NetabaseConfig::builder();
        builder
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(
                BehaviourConfig::with_sled_store_config(path, config)
                    .build()
                    .unwrap(),
            );
        builder
    }
}

#[derive(Builder, Clone)]
pub struct NetabaseSwarmConfig {
    #[builder(default = "Duration::from_secs(30)")]
    connection_timeout: Duration,

    #[builder(default = "true")]
    dns: bool,

    #[builder(default = "None")]
    identity: Option<Keypair>,

    #[builder(default = "tcp::Config::default()")]
    tcp_config: tcp::Config,

    #[builder(default)]
    noise_config: Option<noise::Config>,

    #[builder(default = "yamux::Config::default()")]
    yamux_config: yamux::Config,

    #[builder(default = "true")]
    quic_enabled: bool,

    #[builder(default)]
    quic_config: Option<quic::Config>,

    #[builder(default = "true")]
    mdns_enabled: bool,

    #[builder(default = "false")]
    mdns_auto_connect: bool,

    #[builder(default = "false")]
    relay_enabled: bool,

    #[builder(default = "Vec::new()")]
    external_addresses: Vec<Multiaddr>,

    #[builder(default = "vec![\"/ip4/0.0.0.0/tcp/0\".parse().unwrap()]")]
    listen_addresses: Vec<Multiaddr>,

    #[builder(default = "Some(256)")]
    max_negotiating_inbound_streams: Option<usize>,

    #[builder(default = "Some(1)")]
    max_connections_per_peer: Option<u32>,

    #[builder(default = "Some(256)")]
    max_pending_connections: Option<u32>,

    #[builder(default = "Duration::from_secs(30)")]
    idle_connection_timeout: Duration,

    #[builder(default = "true")]
    connection_limits_enabled: bool,

    #[builder(default = "Vec::new()")]
    bootstrap_nodes: Vec<(String, Multiaddr)>,

    #[builder(default = "\"netabase/0.1.0\".to_string()")]
    user_agent: String,
}

#[derive(Clone)]
pub enum KadStoreConfig {
    MemoryStore {
        peer_id: PeerId,
        config: Option<kad::store::MemoryStoreConfig>,
    },
    SledStore {
        path: String,
        config: Option<crate::database::SledStoreConfig>,
    },
}

impl Default for KadStoreConfig {
    fn default() -> Self {
        KadStoreConfig::SledStore {
            path: "./database".to_string(),
            config: None,
        }
    }
}

impl KadStoreConfig {
    pub fn memory_store(peer_id: PeerId) -> Self {
        KadStoreConfig::MemoryStore {
            peer_id,
            config: None,
        }
    }

    pub fn memory_store_with_config(
        peer_id: PeerId,
        config: kad::store::MemoryStoreConfig,
    ) -> Self {
        KadStoreConfig::MemoryStore {
            peer_id,
            config: Some(config),
        }
    }

    pub fn sled_store<P: AsRef<str>>(path: P) -> Self {
        KadStoreConfig::SledStore {
            path: path.as_ref().to_string(),
            config: None,
        }
    }

    pub fn sled_store_with_config<P: AsRef<str>>(
        path: P,
        config: crate::database::SledStoreConfig,
    ) -> Self {
        KadStoreConfig::SledStore {
            path: path.as_ref().to_string(),
            config: Some(config),
        }
    }
}

#[derive(Builder, Clone)]
pub struct BehaviourConfig {
    #[builder(default)]
    kad_config: Option<kad::Config>,

    #[builder(default)]
    identify_config: Option<identify::Config>,

    #[builder(default)]
    mdns_config: Option<mdns::Config>,

    #[builder(default = "\"/p2p/newsnet/0.1.0\".to_string()")]
    protocol_version: String,

    #[builder(default = "\"netabase/0.1.0\".to_string()")]
    agent_version: String,

    #[builder(default = "KadStoreConfig::default()")]
    store_config: KadStoreConfig,
}

pub type DefaultBehaviourConfig = BehaviourConfig;

impl BehaviourConfig {
    pub fn with_memory_store(peer_id: PeerId) -> BehaviourConfigBuilder {
        let mut builder = BehaviourConfig::builder();
        builder.store_config(KadStoreConfig::memory_store(peer_id));
        builder
    }

    pub fn with_memory_store_config(
        peer_id: PeerId,
        config: kad::store::MemoryStoreConfig,
    ) -> BehaviourConfigBuilder {
        let mut builder = BehaviourConfig::builder();
        builder.store_config(KadStoreConfig::memory_store_with_config(peer_id, config));
        builder
    }

    pub fn with_sled_store<P: AsRef<str>>(path: P) -> BehaviourConfigBuilder {
        let mut builder = BehaviourConfig::builder();
        builder.store_config(KadStoreConfig::sled_store(path));
        builder
    }

    pub fn with_sled_store_config<P: AsRef<str>>(
        path: P,
        config: crate::database::SledStoreConfig,
    ) -> BehaviourConfigBuilder {
        let mut builder = BehaviourConfig::builder();
        builder.store_config(KadStoreConfig::sled_store_with_config(path, config));
        builder
    }
}

impl Default for DefaultBehaviourConfig {
    fn default() -> Self {
        Self {
            kad_config: None,
            identify_config: None,
            mdns_config: None,
            protocol_version: "/p2p/newsnet/0.1.0".to_string(),
            agent_version: "netabase/0.1.0".to_string(),
            store_config: KadStoreConfig::default(),
        }
    }
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
            mdns_auto_connect: false,
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

impl NetabaseSwarmConfig {
    pub fn builder() -> NetabaseSwarmConfigBuilder {
        NetabaseSwarmConfigBuilder::default()
    }

    pub fn connection_timeout(&self) -> Duration {
        self.connection_timeout
    }

    pub fn dns_enabled(&self) -> bool {
        self.dns
    }

    pub fn identity(&self) -> &Option<Keypair> {
        &self.identity
    }

    pub fn tcp_config(&self) -> &tcp::Config {
        &self.tcp_config
    }

    pub fn yamux_config(&self) -> &yamux::Config {
        &self.yamux_config
    }

    pub fn quic_enabled(&self) -> bool {
        self.quic_enabled
    }

    pub fn mdns_enabled(&self) -> bool {
        self.mdns_enabled
    }

    pub fn mdns_auto_connect(&self) -> bool {
        self.mdns_auto_connect
    }

    pub fn relay_enabled(&self) -> bool {
        self.relay_enabled
    }

    pub fn external_addresses(&self) -> &[Multiaddr] {
        &self.external_addresses
    }

    pub fn listen_addresses(&self) -> &[Multiaddr] {
        &self.listen_addresses
    }

    pub fn bootstrap_nodes(&self) -> &[(String, Multiaddr)] {
        &self.bootstrap_nodes
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn idle_connection_timeout(&self) -> Duration {
        self.idle_connection_timeout
    }

    pub fn max_negotiating_inbound_streams(&self) -> Option<usize> {
        self.max_negotiating_inbound_streams
    }

    pub fn max_connections_per_peer(&self) -> Option<u32> {
        self.max_connections_per_peer
    }

    pub fn max_pending_connections(&self) -> Option<u32> {
        self.max_pending_connections
    }
}

impl NetabaseConfig {
    pub fn builder() -> NetabaseConfigBuilder {
        NetabaseConfigBuilder::default()
    }

    pub fn swarm_config(&self) -> &NetabaseSwarmConfig {
        &self.swarm_config
    }

    pub fn behaviour_config(&self) -> &BehaviourConfig {
        &self.behaviour_config
    }
}

impl BehaviourConfig {
    pub fn builder() -> BehaviourConfigBuilder {
        BehaviourConfigBuilder::default()
    }

    pub fn kad_config(&self) -> &Option<kad::Config> {
        &self.kad_config
    }

    pub fn identify_config(&self) -> &Option<identify::Config> {
        &self.identify_config
    }

    pub fn mdns_config(&self) -> &Option<mdns::Config> {
        &self.mdns_config
    }

    pub fn protocol_version(&self) -> &str {
        &self.protocol_version
    }

    pub fn agent_version(&self) -> &str {
        &self.agent_version
    }

    pub fn store_config(&self) -> &KadStoreConfig {
        &self.store_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::{identify, identity::Keypair, kad, mdns};
    use std::time::Duration;

    #[test]
    fn test_default_netabase_config() {
        let config = DefaultNetabaseConfig::builder()
            .swarm_config(NetabaseSwarmConfig::default())
            .behaviour_config(DefaultBehaviourConfig::default())
            .build()
            .unwrap();

        assert_eq!(
            config.swarm_config().connection_timeout(),
            Duration::from_secs(30)
        );
        assert!(config.swarm_config().dns_enabled());
        assert!(config.swarm_config().quic_enabled());
        assert!(config.swarm_config().mdns_enabled());
        assert!(!config.swarm_config().mdns_auto_connect());
        assert_eq!(config.swarm_config().user_agent(), "netabase/0.1.0");

        assert_eq!(
            config.behaviour_config().protocol_version(),
            "/p2p/newsnet/0.1.0"
        );
        assert_eq!(config.behaviour_config().agent_version(), "netabase/0.1.0");
        match config.behaviour_config().store_config() {
            KadStoreConfig::SledStore { path, .. } => assert_eq!(path, "./database"),
            _ => panic!("Expected SledStore"),
        }
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
            .store_config(KadStoreConfig::sled_store("./test_db"))
            .build()
            .expect("Should build valid behaviour config");

        let config = DefaultNetabaseConfig::builder()
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
        match config.behaviour_config().store_config() {
            KadStoreConfig::SledStore { path, .. } => assert_eq!(path, "./test_db"),
            _ => panic!("Expected SledStore"),
        }
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
                "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJB16u19uLTa".to_string(),
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

    #[test]
    fn test_memory_store_config() {
        let peer_id = PeerId::random();
        let config = NetabaseConfig::with_memory_store(peer_id)
            .swarm_config(NetabaseSwarmConfig::default())
            .build()
            .expect("Should build valid config with memory store");

        match config.behaviour_config().store_config() {
            KadStoreConfig::MemoryStore {
                peer_id: store_peer_id,
                config,
            } => {
                assert_eq!(store_peer_id, &peer_id);
                assert!(config.is_none());
            }
            _ => panic!("Expected MemoryStore"),
        }
    }

    #[test]
    fn test_memory_store_with_custom_config() {
        let peer_id = PeerId::random();
        let memory_config = kad::store::MemoryStoreConfig::default();
        let behaviour_config =
            BehaviourConfig::with_memory_store_config(peer_id, memory_config.clone())
                .build()
                .expect("Should build valid behaviour config");

        match behaviour_config.store_config() {
            KadStoreConfig::MemoryStore {
                peer_id: store_peer_id,
                config,
            } => {
                assert_eq!(store_peer_id, &peer_id);
                assert!(config.is_some());
            }
            _ => panic!("Expected MemoryStore"),
        }
    }

    #[test]
    fn test_sled_store_with_custom_config() {
        let sled_config = crate::database::SledStoreConfig {
            max_records: 2048,
            max_value_bytes: 128 * 1024,
            max_provided_keys: 2048,
            max_providers_per_key: 20,
        };

        let behaviour_config =
            BehaviourConfig::with_sled_store_config("./custom_db", sled_config.clone())
                .build()
                .expect("Should build valid behaviour config");

        match behaviour_config.store_config() {
            KadStoreConfig::SledStore { path, config } => {
                assert_eq!(path, "./custom_db");
                assert!(config.is_some());
                if let Some(conf) = config {
                    assert_eq!(conf.max_records, 2048);
                    assert_eq!(conf.max_value_bytes, 128 * 1024);
                }
            }
            _ => panic!("Expected SledStore"),
        }
    }
}
