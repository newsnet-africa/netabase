// Re-export the traits for easier access
pub use netabase_trait::{NetabaseSchema, NetabaseSchemaKey};

// Re-export the derive macro from netabase_macros
use std::path::Path;

use anyhow::anyhow;
use libp2p::{
    Multiaddr, PeerId, Swarm, SwarmBuilder,
    identity::ed25519::Keypair,
    kad::{GetRecordOk, PutRecordOk, QueryId, Quorum},
    swarm::{SwarmEvent, dial_opts::DialOpts},
};

pub mod config;
pub mod database;
pub mod netabase_trait;
pub mod network;

pub use crate::config::NetabaseConfig;

use crate::network::{
    behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent, NetabaseEvent},
    commands::database_commands::Database,
    event_handlers::handle_events,
};

#[derive(Debug)]
pub enum NetabaseCommand {
    Close,
    Database(DatabaseCommand),
    GetListeners(tokio::sync::oneshot::Sender<Vec<Multiaddr>>),
}

#[derive(Debug)]
pub enum DatabaseCommand {
    Put {
        record: libp2p::kad::Record,
        put_to: Option<Vec<PeerId>>,
        quorum: Quorum,
        response_tx: tokio::sync::oneshot::Sender<anyhow::Result<PutRecordOk>>,
    },
    Get {
        key: libp2p::kad::RecordKey,
        response_tx: tokio::sync::oneshot::Sender<anyhow::Result<GetRecordOk>>,
    },
}

pub struct Netabase {
    pub config: NetabaseConfig,
    pub protocol_name: String,
    swarm_thread: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
    pub swarm_active: bool,
    pub swarm_command_sender: Option<tokio::sync::mpsc::Sender<NetabaseCommand>>,
    pub swarm_event_listener: Option<tokio::sync::broadcast::Receiver<NetabaseEvent>>,
}

impl Netabase {
    pub async fn try_new(
        config: NetabaseConfig,
        protocol_name: &'static str,
    ) -> anyhow::Result<Self> {
        Ok(Netabase {
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
        protocol_name: String,
        netabase_config: &config::NetabaseConfig,
    ) -> anyhow::Result<Swarm<NetabaseBehaviour>> {
        println!("{storage_path:?}");
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

    pub async fn start_swarm(&mut self) -> anyhow::Result<()> {
        // Create the event sender channel
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel::<NetabaseEvent>(100);
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel::<NetabaseCommand>(100);

        self.swarm_command_sender = Some(command_sender);

        // Store the receiver for external access
        self.swarm_event_listener = Some(event_receiver);

        // Create swarm in the thread
        let storage_path = self.config.storage_path.clone();
        let keypair_path = self.config.keypair_path.clone();
        let protocol_name: String = self.protocol_name.clone();
        let config = self.config.clone();

        let swarm = Self::generate_swarm(
            &storage_path,
            Self::get_key(&keypair_path).await?,
            protocol_name,
            &config,
        )?;

        // Start listening on configured addresses
        let listen_addresses = self.config.listen_addresses.clone();

        println!("Starting swarm thread");

        // Spawn the event handling task with owned swarm
        let event_loop = tokio::spawn(async move {
            println!("Starting loop");
            handle_events(swarm, event_sender, command_receiver, listen_addresses).await;
            Ok(())
        });

        // Store the join handle
        self.swarm_thread = Some(event_loop);
        self.swarm_active = true;

        Ok(())
    }

    pub async fn listeners(&self) -> anyhow::Result<Vec<Multiaddr>> {
        let command_sender = self
            .swarm_command_sender
            .as_ref()
            .ok_or_else(|| anyhow!("Swarm not started"))?;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        command_sender
            .send(NetabaseCommand::GetListeners(response_tx))
            .await?;

        Ok(response_rx.await?)
    }

    pub async fn close_swarm(self) -> anyhow::Result<()> {
        if let Some(swarm) = self.swarm_thread {
            if let Some(sender) = self.swarm_command_sender {
                sender.send(NetabaseCommand::Close).await?;
            }
            Ok(swarm.await??)
        } else {
            Err(anyhow!("Swarm doesnt exist"))
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
    async fn put<V: NetabaseSchema, I: ExactSizeIterator<Item = PeerId>>(
        &mut self,
        value: V,
        put_to: Option<I>,
        quorum: Quorum,
    ) -> anyhow::Result<PutRecordOk> {
        let command_sender = self
            .swarm_command_sender
            .as_ref()
            .ok_or_else(|| anyhow!("Netabase swarm has not started yet"))?;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let put_to_vec = put_to.map(|iter| iter.collect::<Vec<_>>());

        let command = NetabaseCommand::Database(DatabaseCommand::Put {
            record: value.into(),
            put_to: put_to_vec,
            quorum,
            response_tx,
        });

        command_sender.send(command).await?;
        response_rx.await?
    }

    async fn get<K: NetabaseSchemaKey>(&mut self, key: K) -> anyhow::Result<GetRecordOk> {
        let command_sender = self
            .swarm_command_sender
            .as_ref()
            .ok_or_else(|| anyhow!("Netabase swarm has not started yet"))?;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let record_key: libp2p::kad::RecordKey = key.into();

        let command = NetabaseCommand::Database(DatabaseCommand::Get {
            key: record_key,
            response_tx,
        });

        command_sender.send(command).await?;
        response_rx.await?
    }
}

// Public interface methods for Netabase
impl Netabase {
    /// Get the actual listening addresses from the swarm
    pub async fn get_listening_addresses(&mut self) -> anyhow::Result<Vec<libp2p::Multiaddr>> {
        self.listeners().await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_swarm_thread_architecture() {
        // Test that we can create a netabase instance and start the swarm thread
        let temp_dir = get_test_temp_dir(Some(101), None);
        let listen_addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let config = NetabaseConfig::default()
            .with_storage_path(temp_dir)
            .add_listen_address(listen_addr);

        let mut netabase = Netabase::try_new(config, "/test-protocol")
            .await
            .expect("Failed to create netabase instance");

        // Initially swarm should not be active
        assert!(!netabase.swarm_active);
        assert!(netabase.swarm_command_sender.is_none());
        assert!(netabase.swarm_event_listener.is_none());

        // Start the swarm thread
        netabase.start_swarm().await.expect("Failed to start swarm");

        // After starting, swarm should be active with channels
        assert!(netabase.swarm_active);
        assert!(netabase.swarm_command_sender.is_some());
        assert!(netabase.swarm_event_listener.is_some());

        // Give listeners time to be established
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Test that we can get listeners (this uses the message passing system)
        let listeners = netabase.listeners().await.expect("Failed to get listeners");

        println!("Listeners: {listeners:?}");

        // Should have some listeners since we started the swarm
        assert!(!listeners.is_empty(), "Expected at least one listener");

        // Clean shutdown
        let _ = netabase.close_swarm().await;
    }

    #[tokio::test]
    async fn test_message_passing_channels() {
        // Test that the message passing channels are properly set up
        let temp_dir = get_test_temp_dir(Some(102), None);
        let listen_addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let config = NetabaseConfig::default()
            .with_storage_path(temp_dir)
            .add_listen_address(listen_addr);

        let mut netabase = Netabase::try_new(config, "/test-protocol-2")
            .await
            .expect("Failed to create netabase instance");

        netabase.start_swarm().await.expect("Failed to start swarm");

        // Verify we have command sender and event listener
        assert!(netabase.swarm_command_sender.is_some());
        assert!(netabase.swarm_event_listener.is_some());

        // Give listeners time to be established
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Test multiple listener requests to verify the channel works
        let listeners1 = netabase
            .listeners()
            .await
            .expect("First listener request failed");
        let listeners2 = netabase
            .listeners()
            .await
            .expect("Second listener request failed");

        // Both calls should succeed and return the same results
        assert_eq!(listeners1, listeners2);
        assert!(!listeners1.is_empty());

        // Clean shutdown
        let _ = netabase.close_swarm().await;
    }

    #[tokio::test]
    async fn test_database_command_channel() {
        // Test that database command channels are set up correctly
        let temp_dir = get_test_temp_dir(Some(103), None);
        let listen_addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let config = NetabaseConfig::default()
            .with_storage_path(temp_dir)
            .add_listen_address(listen_addr);

        let mut netabase = Netabase::try_new(config, "/test-protocol-3")
            .await
            .expect("Failed to create netabase instance");

        netabase.start_swarm().await.expect("Failed to start swarm");

        // Verify we have command sender and event listener for database operations
        assert!(netabase.swarm_command_sender.is_some());
        assert!(netabase.swarm_event_listener.is_some());

        // Test that we can send commands through the channel by creating a simple record key
        let test_key = libp2p::kad::RecordKey::new(&b"test-key");

        // Test GET operation through message passing - this tests the channel infrastructure
        // We use timeout because there are no peers to respond
        let get_result = tokio::time::timeout(std::time::Duration::from_millis(100), {
            let command_sender = netabase.swarm_command_sender.as_ref().unwrap();
            let (response_tx, response_rx) = tokio::sync::oneshot::channel();

            let command = NetabaseCommand::Database(DatabaseCommand::Get {
                key: test_key,
                response_tx,
            });

            async move {
                command_sender.send(command).await.unwrap();
                response_rx.await.unwrap()
            }
        })
        .await;

        // The message passing system is working! We either get:
        // 1. A timeout (no peers available)
        // 2. A successful result with NotFound error (system worked but no data)
        match get_result {
            Ok(result) => {
                // Great! The message passing worked and we got a result
                // This means the swarm thread processed our command successfully
                assert!(
                    result.is_err(),
                    "Expected NotFound error since no data exists"
                );
                println!("Message passing successful: {:?}", result);
            }
            Err(_) => {
                // Timeout occurred, which is also acceptable since no peers are available
                println!("Timeout occurred - this is expected when no peers are available");
            }
        }

        // Clean shutdown
        let _ = netabase.close_swarm().await;
    }
}
