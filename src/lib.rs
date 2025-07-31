#![feature(duration_constructors_lite)]

// Re-export the traits for easier access
pub use netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
use std::{path::Path, sync::Arc, time::Duration};

use anyhow::anyhow;
use libp2p::{
    Swarm, SwarmBuilder,
    identity::ed25519::Keypair,
    kad::{PutRecordOk, Quorum},
    swarm::SwarmEvent,
};

pub mod config;
pub mod database;
pub mod netabase_trait;
pub mod network;

use derive_builder::Builder;

// Re-export the derive macro from netabase_macros
use tokio::sync::Mutex;

use crate::{
    config::NetabaseConfig,
    network::{
        behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent, NetabaseEvent},
        commands::database_commands::Database,
        event_handlers::handle_events,
    },
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
    pub _config: NetabaseConfig,
    pub swarm: Arc<tokio::sync::Mutex<Swarm<NetabaseBehaviour>>>,
    pub public_key: Option<libp2p::identity::ed25519::PublicKey>,
    pub protocol_name: String,
    swarm_thread: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
    pub swarm_command_sender: Option<tokio::sync::mpsc::Sender<NetabaseCommand>>,
    pub swarm_event_listener: Option<tokio::sync::broadcast::Receiver<NetabaseEvent>>,
}

impl Netabase {
    pub fn try_new<P: ToString>(
        _config: NetabaseConfig,
        k: &Keypair,
        protocol_name: P,
    ) -> anyhow::Result<Self> {
        Ok(Netabase {
            swarm: Arc::new(Mutex::new(Self::generate_swarm(
                &_config.storage_path,
                k.clone(),
                protocol_name.to_string(),
            )?)),
            swarm_thread: None,
            swarm_command_sender: None,
            public_key: Some(k.public()),
            swarm_event_listener: None,
            _config,
            protocol_name: protocol_name.to_string(),
        })
    }
    fn generate_swarm<P: AsRef<Path>, Pr: ToString>(
        storage_path: P,
        local_key: Keypair,
        protocol_name: Pr,
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
            .with_behaviour(|k| {
                NetabaseBehaviour::new(storage_path, k, protocol_name.to_string())
                    .expect("Fix later")
            })?
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_mins(5)))
            .build())
    }
    pub fn start_swarm(&mut self) -> anyhow::Result<()> {
        // Create the event sender channel
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(100);
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel::<NetabaseCommand>(100);

        // Store the receiver for external access
        self.swarm_event_listener = Some(event_receiver);

        // Clone the Arc for the async task
        let swarm_clone = Arc::clone(&self.swarm);

        // Spawn the event handling task
        let event_loop = tokio::spawn(async move {
            handle_events(swarm_clone, event_sender).await;
            Ok(())
        });

        // Store the join handle
        self.swarm_thread = Some(event_loop);

        Ok(())
    }
    async fn inner_put<V: NetabaseSchema>(
        &mut self,
        swarm: Arc<Mutex<Swarm<NetabaseBehaviour>>>,
        quorum: libp2p::kad::Quorum,
        mut result_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
        value: V,
    ) -> anyhow::Result<PutRecordOk> {
        let query_id = swarm
            .lock()
            .await
            .behaviour_mut()
            .kad
            .put_record(value.into(), quorum)?; //TODO: allow users to config timeout

        loop {
            if let Ok(NetabaseEvent(SwarmEvent::Behaviour(NetabaseBehaviourEvent::Kad(
                libp2p::kad::Event::OutboundQueryProgressed {
                    id,
                    result,
                    stats,
                    step,
                },
            )))) = result_listener.recv().await
                && id.eq(&query_id)
                && step.last
                && let libp2p::kad::QueryResult::PutRecord(res) = result
            {
                match res {
                    Ok(put_ok) => return Ok(put_ok),
                    Err(put_err) => return Err(put_err.into()),
                }
            }
        }
    }
    // fn inner_get<K: NetabaseSchemaKey>
}

impl Database for Netabase {
    async fn put<V: netabase_trait::NetabaseSchema>(
        &mut self,
        value: V,
        quorum: Quorum,
    ) -> anyhow::Result<PutRecordOk> {
        match &self.swarm_event_listener {
            Some(listener) => {
                self.inner_put(self.swarm.clone(), quorum, listener.resubscribe(), value)
                    .await
            }
            None => Err(anyhow!("Netabase swarm has not started yet")),
        }
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
