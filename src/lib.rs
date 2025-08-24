// Re-export the traits for easier access
pub use netabase_trait::{NetabaseSchema, NetabaseSchemaKey};

// Re-export the derive macro from netabase_macros
pub use netabase_macros::{NetabaseSchema, schema};
use std::{path::Path, sync::Arc, time::Duration};

use anyhow::anyhow;
use libp2p::{
    Multiaddr, PeerId, Swarm, SwarmBuilder,
    identity::ed25519::Keypair,
    kad::{GetRecordOk, PutRecordOk, Quorum},
    swarm::{SwarmEvent, dial_opts::DialOpts},
};

pub mod config;
pub mod database;
pub mod netabase_trait;
pub mod network;

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

pub enum NetabaseCommand {
    Close,
    Database,
}

enum DatabaseCommand<K: NetabaseSchemaKey, V: NetabaseSchema> {
    Put(K, V),
    Get(K),
    Delete(K),
}

pub struct Netabase {
    pub config: NetabaseConfig,
    pub swarm: Arc<tokio::sync::Mutex<Swarm<NetabaseBehaviour>>>,
    pub protocol_name: String,
    swarm_thread: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
    pub swarm_active: bool, // Might become swarm state
    pub swarm_command_sender: Option<tokio::sync::mpsc::Sender<NetabaseCommand>>,
    pub swarm_event_listener: Option<tokio::sync::broadcast::Receiver<NetabaseEvent>>,
}

impl Netabase {
    pub async fn try_new(
        config: NetabaseConfig,
        protocol_name: &'static str,
    ) -> anyhow::Result<Self> {
        Ok(Netabase {
            swarm: Arc::new(Mutex::new(Self::generate_swarm(
                &config.storage_path,
                Self::get_key(&config.keypair_path).await?,
                protocol_name,
                &config,
            )?)),
            swarm_thread: None,
            swarm_command_sender: None,
            swarm_event_listener: None,
            config,
            protocol_name: protocol_name.to_string(),
            swarm_active: false,
        })
    }

    pub async fn try_new_default(protocol_name: &'static str) -> anyhow::Result<Self> {
        Self::try_new(
            NetabaseConfig::default().with_storage_path(format!(".{protocol_name}").into()),
            protocol_name,
        )
        .await
    }

    async fn get_key<P: AsRef<Path>>(path: P) -> anyhow::Result<Keypair> {
        let bytes_result = tokio::fs::read(&path).await;
        match bytes_result {
            Ok(mut bytes) => Ok(Keypair::try_from_bytes(bytes.as_mut_slice())?),
            Err(_) => {
                let kp = Keypair::generate();
                tokio::fs::write(path, kp.to_bytes()).await;
                Ok(kp)
            }
        }
    }

    fn generate_swarm(
        storage_path: &Path,
        local_key: Keypair,
        protocol_name: &'static str,
        netabase_config: &config::NetabaseConfig,
    ) -> anyhow::Result<Swarm<NetabaseBehaviour>> {
        eprintln!("{storage_path:?}");
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
                NetabaseBehaviour::new(storage_path, k, protocol_name, netabase_config.clone())
                    .expect("Fix later")
            })?
            .with_swarm_config(|cfg| {
                let mut config = cfg
                    .with_idle_connection_timeout(netabase_config.swarm.connection_idle_timeout)
                    .with_notify_handler_buffer_size(
                        netabase_config.swarm.notify_handler_buffer_size,
                    )
                    .with_per_connection_event_buffer_size(
                        netabase_config.swarm.per_connection_event_buffer_size,
                    )
                    .with_dial_concurrency_factor(netabase_config.swarm.dial_concurrency_factor)
                    .with_max_negotiating_inbound_streams(
                        netabase_config.swarm.max_negotiating_inbound_streams,
                    );

                if let Some(version) = netabase_config.swarm.substream_upgrade_protocol_override {
                    config = config.with_substream_upgrade_protocol_override(version);
                }

                config
            })
            .build())
    }
    pub fn start_swarm(&mut self) -> anyhow::Result<()> {
        // Create the event sender channel
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(100);
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel::<NetabaseCommand>(100);

        self.swarm_command_sender = Some(command_sender);

        // Store the receiver for external access
        self.swarm_event_listener = Some(event_receiver);

        // Start listening on configured addresses
        let listen_addresses = self.config.listen_addresses.clone();
        let bootstrap_addresses = self.config.bootstrap_addresses.clone();
        let swarm_clone = Arc::clone(&self.swarm);
        let swarm_clone_for_events = Arc::clone(&self.swarm);

        // Set up listening and bootstrap connections
        tokio::spawn(async move {
            {
                let mut swarm_guard = swarm_clone.lock().await;

                // Start listening on all configured addresses
                for addr in listen_addresses {
                    if let Err(e) = (swarm_guard).listen_on(addr.clone()) {
                        eprintln!("Failed to listen on {}: {}", addr, e);
                    } else {
                        println!("Listening on: {}", addr);
                    }
                }

                // Connect to bootstrap peers and add them to Kademlia routing table
                for bootstrap_addr in bootstrap_addresses {
                    println!("Attempting to dial bootstrap peer: {}", bootstrap_addr);

                    // Extract PeerID from the multiaddr for Kademlia
                    if let Some(peer_id) = extract_peer_id_from_multiaddr(&bootstrap_addr) {
                        // Add to Kademlia routing table for better discovery
                        (&mut *swarm_guard)
                            .behaviour_mut()
                            .kad
                            .add_address(&peer_id, bootstrap_addr.clone());
                        println!("Added peer {} to Kademlia routing table", peer_id);
                    }

                    if let Err(e) = (&mut *swarm_guard).dial(bootstrap_addr.clone()) {
                        eprintln!("Failed to dial bootstrap peer {}: {}", bootstrap_addr, e);
                    } else {
                        println!("Successfully initiated connection to: {}", bootstrap_addr);
                    }
                }
            } // Release the lock here
        });

        // Spawn the event handling task
        let event_loop = tokio::spawn(async move {
            handle_events(swarm_clone_for_events, event_sender, command_receiver).await;
            Ok(())
        });

        // Store the join handle
        self.swarm_thread = Some(event_loop);

        Ok(())
    }

    pub async fn listeners(&self) -> Vec<Multiaddr> {
        // use iters instead
        self.swarm
            .lock()
            .await
            .listeners()
            .cloned()
            .collect::<Vec<Multiaddr>>()
    }

    pub async fn close_swarm(self) -> anyhow::Result<()> {
        if let Some(swarm) = self.swarm_thread {
            self.swarm_command_sender
                .unwrap()
                .send(NetabaseCommand::Close)
                .await?;

            Ok(swarm.await??)
        } else {
            Err(anyhow!("Swarm doesnt exist"))
        }
    }

    async fn inner_put<V: NetabaseSchema, I: ExactSizeIterator<Item = PeerId>>(
        &mut self,
        swarm: Arc<Mutex<Swarm<NetabaseBehaviour>>>,
        quorum: libp2p::kad::Quorum,
        mut result_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
        value: V,
        put_to: Option<I>,
    ) -> anyhow::Result<PutRecordOk> {
        let query_id = {
            let mut swarm_guard = swarm.lock().await;
            match put_to {
                None => (&mut *swarm_guard)
                    .behaviour_mut()
                    .kad
                    .put_record(value.into(), quorum)?, //TODO: allow users to config timeout
                Some(peers) => (&mut *swarm_guard).behaviour_mut().kad.put_record_to(
                    value.into(),
                    peers,
                    quorum,
                ),
            }
        };

        loop {
            //TODO: Consider adding a timeout here, but I'm like 50% sure that Kad does this for us anyways
            if let Ok(NetabaseEvent(SwarmEvent::Behaviour(NetabaseBehaviourEvent::Kad(
                libp2p::kad::Event::OutboundQueryProgressed {
                    id,
                    result,
                    stats: _,
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
    async fn inner_get<V: NetabaseSchemaKey>(
        //TODO: make a mcaro for put and get
        &mut self,
        swarm: Arc<Mutex<Swarm<NetabaseBehaviour>>>,
        mut result_listener: tokio::sync::broadcast::Receiver<NetabaseEvent>,
        value: V,
    ) -> anyhow::Result<GetRecordOk> {
        let query_id = {
            let mut swarm_guard = swarm.lock().await;
            (&mut *swarm_guard)
                .behaviour_mut()
                .kad
                .get_record(value.into()) //TODO: allow users to config timeout
        };

        loop {
            //TODO: Consider adding a timeout here, but I'm like 50% sure that Kad does this for us anyways
            if let Ok(NetabaseEvent(SwarmEvent::Behaviour(NetabaseBehaviourEvent::Kad(
                libp2p::kad::Event::OutboundQueryProgressed {
                    id,
                    result,
                    stats: _,
                    step,
                },
            )))) = result_listener.recv().await
                && id.eq(&query_id)
                && step.last
                && let libp2p::kad::QueryResult::GetRecord(res) = result
            {
                match res {
                    Ok(put_ok) => return Ok(put_ok),
                    Err(put_err) => return Err(put_err.into()),
                }
            }
        }
    }
}

// Public interface methods for Netabase
impl Netabase {
    pub async fn put<V: NetabaseSchema, I: ExactSizeIterator<Item = PeerId>>(
        &mut self,
        value: V,
        put_to: Option<I>,
        quorum: Quorum,
    ) -> anyhow::Result<PutRecordOk> {
        Database::put(self, value, put_to, quorum).await
    }

    pub async fn get<K: NetabaseSchemaKey>(&mut self, key: K) -> anyhow::Result<GetRecordOk> {
        Database::get(self, key).await
    }
}

impl Database for Netabase {
    async fn put<V: netabase_trait::NetabaseSchema, I: ExactSizeIterator<Item = PeerId>>(
        &mut self,
        value: V,
        put_to: Option<I>,
        quorum: Quorum,
    ) -> anyhow::Result<PutRecordOk> {
        match &self.swarm_event_listener {
            Some(listener) => {
                self.inner_put(
                    self.swarm.clone(),
                    quorum,
                    listener.resubscribe(),
                    value,
                    put_to,
                )
                .await
            }
            None => Err(anyhow!("Netabase swarm has not started yet")),
        }
    }
    async fn get<K: NetabaseSchemaKey>(&mut self, key: K) -> anyhow::Result<GetRecordOk> {
        match &self.swarm_event_listener {
            Some(listener) => {
                self.inner_get(self.swarm.clone(), listener.resubscribe(), key)
                    .await
            }
            None => Err(anyhow!("Netabase swarm has not started yet")),
        }
    }
}

// Public interface methods for Netabase
impl Netabase {
    /// Get the actual listening addresses from the swarm
    pub async fn get_listening_addresses(&mut self) -> Vec<libp2p::Multiaddr> {
        let swarm_guard = self.swarm.lock().await;
        (&*swarm_guard).listeners().cloned().collect()
    }
}

/// Initialize logging for the application
/// This function can be called multiple times safely - it will only initialize once
pub fn init_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        env_logger::init();
    });
}

/// Get a temporary directory for testing purposes
pub fn get_test_temp_dir(test_number: Option<u64>, _suffix: Option<&str>) -> std::path::PathBuf {
    let test_id = test_number.unwrap_or_else(|| rand::random::<u64>());
    std::env::temp_dir().join(format!("netabase_test_{}", test_id))
}

/// Helper function to extract PeerID from multiaddr
fn extract_peer_id_from_multiaddr(addr: &libp2p::Multiaddr) -> Option<PeerId> {
    use libp2p::multiaddr::Protocol;

    for protocol in addr.iter() {
        if let Protocol::P2p(hash) = protocol {
            return Some(PeerId::from(hash));
        }
    }
    None
}
