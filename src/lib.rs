#![feature(duration_constructors_lite)]
#![feature(inherent_associated_types)]

use std::{path::Path, sync::Arc, time::Duration};

use libp2p::{Swarm, SwarmBuilder, identity::ed25519::Keypair, swarm::SwarmEvent};
use rand;

pub mod config;
pub mod database;
pub mod netabase_trait;
pub mod network;

use derive_builder::Builder;

// Re-export the derive macro from netabase_macros
pub use netabase_macros::NetabaseSchema;
use tokio::sync::Mutex;

use crate::{
    config::NetabaseConfig,
    network::behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent},
};

enum NetabaseCommand {
    Database(DatabaseCommand<Vec<u8>, Vec<u8>>),
}

enum DatabaseCommand<K: Into<Vec<u8>>, V: Into<Vec<u8>>> {
    Put(K, V),
    Get(K),
    Delete(K),
}

pub struct Netabase {
    _config: NetabaseConfig,
    swarm: Arc<tokio::sync::Mutex<Swarm<NetabaseBehaviour>>>,
    public_key: Option<libp2p::identity::ed25519::PublicKey>,
    swarm_thread: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
    swarm_sender: Option<tokio::sync::mpsc::Sender<NetabaseCommand>>,
    swarm_event_listener: Option<tokio::sync::mpsc::Receiver<NetabaseCommand>>,
}

impl Netabase {
    type TryNewError = anyhow::Error;
    pub fn try_new(_config: NetabaseConfig, k: &Keypair) -> Result<Self, Self::TryNewError> {
        Ok(Netabase {
            swarm: Arc::new(Mutex::new(Self::generate_swarm(
                &_config.storage_path,
                k.clone(),
            )?)),
            swarm_thread: None,
            swarm_sender: None,
            public_key: Some(k.public()),
            swarm_event_listener: None,
            _config,
        })
    }
    fn generate_swarm<P: AsRef<Path>>(
        storage_path: P,
        local_key: Keypair,
    ) -> anyhow::Result<Swarm<NetabaseBehaviour>> {
        Ok(SwarmBuilder::with_existing_identity(local_key.into())
            .with_tokio()
            .with_tcp(
                Default::default(),
                (libp2p::tls::Config::new, libp2p::noise::Config::new),
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            .with_dns()?
            .with_behaviour(|k| NetabaseBehaviour::new(storage_path, k).expect("Fix later"))?
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_mins(5)))
            .build())
    }

    fn start_swarm(&self) {
        let (tx, rx) = tokio::sync::mpsc::channel::<SwarmEvent<NetabaseBehaviour>>(100);
        let event_loop = tokio::spawn();
    }
}

/// Initialize logging for the application
pub fn init_logging() {
    env_logger::init();
}

/// Get a temporary directory for testing purposes
pub fn get_test_temp_dir(test_number: Option<u64>, _suffix: Option<&str>) -> std::path::PathBuf {
    let test_id = test_number.unwrap_or_else(|| rand::random::<u64>());
    std::env::temp_dir().join(format!("netabase_test_{}", test_id))
}
