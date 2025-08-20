use libp2p::Multiaddr;
use std::{path::PathBuf, time::Duration};

pub struct NetabaseConfig {
    pub storage_path: PathBuf,
    pub listen_addresses: Vec<Multiaddr>,
    pub bootstrap_addresses: Vec<Multiaddr>,
    pub kad_replication_factor: usize,
    pub kad_query_timeout: Duration,
    pub connection_idle_timeout: Duration,
    pub enable_mdns: bool,
}

impl Default for NetabaseConfig {
    fn default() -> Self {
        Self {
            storage_path: std::env::temp_dir().join("netabase"),
            listen_addresses: vec![],
            bootstrap_addresses: vec![],
            kad_replication_factor: 20,
            kad_query_timeout: Duration::from_secs(60),
            connection_idle_timeout: Duration::from_secs(300),
            enable_mdns: true,
        }
    }
}
