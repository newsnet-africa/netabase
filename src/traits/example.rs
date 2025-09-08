//! Example implementations and usage patterns for Netabase traits
//!
//! This module demonstrates how to implement and use the various Netabase traits
//! to create a modular, testable system with clear separation of concerns.

use crate::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
use crate::traits::network::NetabaseNetwork;
use crate::traits::prelude::*;
use crate::traits::{
    configuration::{
        ConfigurationBuilder, ConfigurationEvent, ConfigurationMetadata, ConfigurationOptions,
        ConfigurationSource, FileFormat, MergeStrategy, ValidationLevel,
    },
    core::{
        ComponentHealth, HealthStatus, NetabaseConfig, NetabaseEvent, PerformanceMetrics,
        SystemHealth, SystemState, SystemStats,
    },
    database::{DatabaseConfig, DatabaseEvent, DatabaseStats, DatabaseTransaction, QueryOptions},
    network::{
        BroadcastOptions, NetworkConfig, NetworkEvent, NetworkHealth, NetworkMessage, NetworkStats,
        PeerInfo,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libp2p::kad::{Record, record::Key as KadKey};
use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

// Example key and value types for demonstrations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExampleKey(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleValue {
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl NetabaseSchemaKey for ExampleKey {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ExampleKey(String::from_utf8(bytes)?))
    }
}

impl NetabaseSchema for ExampleValue {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn deserialize(data: Vec<u8>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::from_slice(&data)?)
    }
}

// =============================================================================
// Mock Database Implementation
// =============================================================================

/// Mock in-memory database implementation for testing and examples
pub struct MockDatabase<K: NetabaseSchemaKey, V: NetabaseSchema> {
    data: Arc<Mutex<HashMap<K, V>>>,
    config: Option<DatabaseConfig>,
    stats: DatabaseStats,
    event_sender: broadcast::Sender<DatabaseEvent<K, V>>,
}

impl<K: NetabaseSchemaKey, V: NetabaseSchema> MockDatabase<K, V> {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            config: None,
            stats: DatabaseStats {
                total_entries: 0,
                total_size: 0,
                last_compaction: None,
                cache_hit_rate: 1.0,
                average_entry_size: 0,
                total_kad_records: 0,
            },
            event_sender,
        }
    }
}

#[async_trait]
impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> NetabaseDatabase<K, V>
    for MockDatabase<K, V>
{
    async fn initialize(&mut self, config: DatabaseConfig) -> DatabaseResult<()> {
        self.config = Some(config);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.config.is_some()
    }

    async fn close(&mut self) -> DatabaseResult<()> {
        self.data.lock().unwrap().clear();
        Ok(())
    }

    async fn put(&mut self, key: K, value: V) -> DatabaseResult<()> {
        let mut data = self.data.lock().unwrap();
        let is_update = data.contains_key(&key);

        if let Some(old_value) = data.insert(key.clone(), value.clone()) {
            let _ = self.event_sender.send(DatabaseEvent::Updated {
                key,
                old_value,
                new_value: value,
            });
        } else {
            let _ = self
                .event_sender
                .send(DatabaseEvent::Inserted { key, value });
        }

        Ok(())
    }

    async fn put_batch(&mut self, entries: Vec<(K, V)>) -> DatabaseResult<()> {
        for (key, value) in entries {
            self.put(key, value).await?;
        }
        Ok(())
    }

    async fn get(&self, key: &K) -> DatabaseResult<Option<V>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(key).cloned())
    }

    async fn get_batch(&self, keys: &[K]) -> DatabaseResult<HashMap<K, V>> {
        let data = self.data.lock().unwrap();
        let mut result = HashMap::new();
        for key in keys {
            if let Some(value) = data.get(key) {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(result)
    }

    async fn contains_key(&self, key: &K) -> DatabaseResult<bool> {
        let data = self.data.lock().unwrap();
        Ok(data.contains_key(key))
    }

    async fn delete(&mut self, key: &K) -> DatabaseResult<bool> {
        let mut data = self.data.lock().unwrap();
        if let Some(value) = data.remove(key) {
            let _ = self.event_sender.send(DatabaseEvent::Deleted {
                key: key.clone(),
                value,
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_batch(&mut self, keys: &[K]) -> DatabaseResult<Vec<K>> {
        let mut deleted = Vec::new();
        for key in keys {
            if self.delete(key).await? {
                deleted.push(key.clone());
            }
        }
        Ok(deleted)
    }

    async fn keys(&self, _options: Option<QueryOptions>) -> DatabaseResult<Vec<K>> {
        let data = self.data.lock().unwrap();
        Ok(data.keys().cloned().collect())
    }

    async fn values(&self, _options: Option<QueryOptions>) -> DatabaseResult<Vec<V>> {
        let data = self.data.lock().unwrap();
        Ok(data.values().cloned().collect())
    }

    async fn entries(&self, _options: Option<QueryOptions>) -> DatabaseResult<Vec<(K, V)>> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    async fn len(&self) -> DatabaseResult<usize> {
        let data = self.data.lock().unwrap();
        Ok(data.len())
    }

    async fn is_empty(&self) -> DatabaseResult<bool> {
        let data = self.data.lock().unwrap();
        Ok(data.is_empty())
    }

    async fn clear(&mut self) -> DatabaseResult<()> {
        let mut data = self.data.lock().unwrap();
        data.clear();
        let _ = self.event_sender.send(DatabaseEvent::Cleared);
        Ok(())
    }

    async fn begin_transaction(&mut self) -> DatabaseResult<DatabaseTransaction<'_, K, V>> {
        // Simplified transaction implementation
        Ok(DatabaseTransaction {
            _phantom: std::marker::PhantomData,
        })
    }

    async fn commit_transaction(
        &mut self,
        _transaction: DatabaseTransaction<'_, K, V>,
    ) -> DatabaseResult<()> {
        // In a real implementation, this would commit pending operations
        Ok(())
    }

    async fn rollback_transaction(
        &mut self,
        _transaction: DatabaseTransaction<'_, K, V>,
    ) -> DatabaseResult<()> {
        // In a real implementation, this would discard pending operations
        Ok(())
    }

    async fn compact(&mut self) -> DatabaseResult<()> {
        // Mock compaction - in reality this would optimize storage
        self.stats.last_compaction = Some(Utc::now());
        Ok(())
    }

    async fn stats(&self) -> DatabaseResult<DatabaseStats> {
        let data = self.data.lock().unwrap();
        let total_entries = data.len();
        let total_size = data
            .values()
            .map(|v| v.serialize().unwrap_or_default().len())
            .sum::<usize>();

        Ok(DatabaseStats {
            total_entries,
            total_size,
            last_compaction: self.stats.last_compaction,
            cache_hit_rate: 0.95, // Mock value
            average_entry_size: if total_entries > 0 {
                total_size / total_entries
            } else {
                0
            },
            total_kad_records: total_entries,
            records_pending_republish: 0, // Mock value
            records_expiring_soon: 0,     // Mock value
        })
    }

    async fn put_record(&mut self, record: libp2p::kad::Record) -> DatabaseResult<()> {
        let value: V = self.from_record(record);
        let key = value.key();
        self.put(key, value).await
    }

    async fn get_record(
        &self,
        key: &libp2p::kad::record::Key,
    ) -> DatabaseResult<Option<libp2p::kad::Record>> {
        let schema_key = self.kad_key_to_schema_key(key.clone());
        if let Some(value) = self.get(&schema_key).await? {
            let record = self.to_record(value);
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    fn to_record(&self, value: V) -> libp2p::kad::Record {
        value.into()
    }

    fn from_record(&self, record: libp2p::kad::Record) -> V {
        record.into()
    }

    async fn get_republish_records(&self) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        // Mock implementation - return first 10 records as needing republish
        let data = self.data.lock().unwrap();
        let mut records = Vec::new();

        for (key, value) in data.iter().take(10) {
            let record = self.to_record(value.clone());
            records.push(record);
        }

        Ok(records)
    }

    async fn mark_for_republish(&mut self, _keys: &[K]) -> DatabaseResult<()> {
        // Mock implementation - in reality, this would mark records for republishing
        Ok(())
    }

    async fn get_expiring_records(
        &self,
        _within: std::time::Duration,
    ) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        // Mock implementation - return empty list
        Ok(Vec::new())
    }

    async fn update_record_expiry(
        &mut self,
        _key: &K,
        _expires: Option<std::time::Instant>,
    ) -> DatabaseResult<()> {
        // Mock implementation - in reality, this would update expiration tracking
        Ok(())
    }

    async fn backup<P: AsRef<str> + Send>(&self, _backup_path: P) -> DatabaseResult<()> {
        // Mock backup implementation
        Ok(())
    }

    async fn restore<P: AsRef<str> + Send>(&mut self, _backup_path: P) -> DatabaseResult<()> {
        // Mock restore implementation
        Ok(())
    }
}

#[async_trait]
impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> NetabaseDatabaseExt<K, V>
    for MockDatabase<K, V>
{
    async fn update(&mut self, key: &K, value: V) -> DatabaseResult<bool> {
        let mut data = self.data.lock().unwrap();
        if data.contains_key(key) {
            data.insert(key.clone(), value);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn insert(&mut self, key: K, value: V) -> DatabaseResult<bool> {
        let mut data = self.data.lock().unwrap();
        if data.contains_key(&key) {
            Ok(false)
        } else {
            data.insert(key, value);
            Ok(true)
        }
    }

    async fn upsert(&mut self, key: K, value: V) -> DatabaseResult<()> {
        self.put(key, value).await
    }

    async fn take(&mut self, key: &K) -> DatabaseResult<Option<V>> {
        let mut data = self.data.lock().unwrap();
        Ok(data.remove(key))
    }

    async fn update_with<F>(&mut self, key: &K, updater: F) -> DatabaseResult<bool>
    where
        F: FnOnce(V) -> V + Send + Sync,
    {
        let mut data = self.data.lock().unwrap();
        if let Some(old_value) = data.remove(key) {
            let new_value = updater(old_value);
            data.insert(key.clone(), new_value);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn scan_prefix(
        &self,
        prefix: &str,
        _options: Option<QueryOptions>,
    ) -> DatabaseResult<Vec<K>> {
        let data = self.data.lock().unwrap();
        let prefix_bytes = prefix.as_bytes();
        let mut result = Vec::new();

        for key in data.keys() {
            let key_bytes = key.as_bytes();
            if key_bytes.starts_with(prefix_bytes) {
                result.push(key.clone());
            }
        }

        Ok(result)
    }

    async fn scan_range(
        &self,
        _start: &K,
        _end: &K,
        _options: Option<QueryOptions>,
    ) -> DatabaseResult<Vec<K>> {
        // Mock implementation - just return all keys
        self.keys(None).await
    }

    async fn subscribe_key(
        &self,
        _key: &K,
    ) -> DatabaseResult<tokio::sync::broadcast::Receiver<DatabaseEvent<K, V>>> {
        Ok(self.event_sender.subscribe())
    }

    async fn subscribe_all(
        &self,
    ) -> DatabaseResult<tokio::sync::broadcast::Receiver<DatabaseEvent<K, V>>> {
        Ok(self.event_sender.subscribe())
    }

    async fn sync_to_network(&mut self) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        self.records_iter().await
    }

    async fn sync_from_network(&mut self, records: Vec<libp2p::kad::Record>) -> DatabaseResult<()> {
        for record in records {
            self.put_record(record).await?;
        }
        Ok(())
    }

    async fn get_records_by_prefix(
        &self,
        prefix: &[u8],
    ) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        let data = self.data.lock().unwrap();
        let mut records = Vec::new();

        for (key, value) in data.iter() {
            let kad_key = self.schema_key_to_kad_key(key.clone());
            if kad_key.as_ref().starts_with(prefix) {
                let record = self.to_record(value.clone());
                records.push(record);
            }
        }

        Ok(records)
    }

    async fn put_records_batch(&mut self, records: Vec<libp2p::kad::Record>) -> DatabaseResult<()> {
        for record in records {
            self.put_record(record).await?;
        }
        Ok(())
    }

    async fn is_local_newer(&self, network_record: &libp2p::kad::Record) -> DatabaseResult<bool> {
        // Mock implementation - always consider local as newer
        Ok(true)
    }

    async fn resolve_record_conflict(
        &mut self,
        local_record: libp2p::kad::Record,
        network_record: libp2p::kad::Record,
    ) -> DatabaseResult<libp2p::kad::Record> {
        // Mock implementation - always prefer local record
        Ok(local_record)
    }

    async fn records_iter(&self) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        let data = self.data.lock().unwrap();
        let mut records = Vec::new();

        for (key, value) in data.iter() {
            let record = self.to_record(value.clone());
            records.push(record);
        }

        Ok(records)
    }

    async fn get_records_by_publisher(
        &self,
        _publisher: Option<libp2p::PeerId>,
    ) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        // Mock implementation - just return all records
        self.records_iter().await
    }

    async fn remove_expired_records(&mut self) -> DatabaseResult<usize> {
        // Mock implementation - no expiration tracking in simple HashMap
        Ok(0)
    }

    async fn get_closest_records(
        &self,
        _key: &libp2p::kad::record::Key,
        limit: usize,
    ) -> DatabaseResult<Vec<libp2p::kad::Record>> {
        let records = self.records_iter().await?;
        Ok(records.into_iter().take(limit).collect())
    }

    async fn is_at_capacity(&self) -> DatabaseResult<bool> {
        Ok(false)
    }

    async fn capacity_info(&self) -> DatabaseResult<crate::traits::database::StoreCapacityInfo> {
        let data = self.data.lock().unwrap();
        let current_records = data.len();
        let current_size_bytes = data
            .values()
            .map(|v| v.serialize().unwrap_or_default().len())
            .sum::<usize>();

        Ok(crate::traits::database::StoreCapacityInfo {
            current_records,
            max_records: None,
            current_size_bytes,
            max_size_bytes: None,
            is_full: false,
            utilization_percentage: 0.0,
        })
    }

    async fn maintain_store(
        &mut self,
    ) -> DatabaseResult<crate::traits::database::StoreMaintenanceResult> {
        let start_time = std::time::Instant::now();
        let maintenance_duration = start_time.elapsed();

        let result = crate::traits::database::StoreMaintenanceResult {
            records_removed: 0,
            bytes_freed: 0,
            expired_records_cleaned: 0,
            maintenance_duration,
        };

        Ok(result)
    }
}

// =============================================================================
// Mock Network Implementation
// =============================================================================

/// Mock network implementation for testing and examples
pub struct MockNetwork<K: NetabaseSchemaKey, V: NetabaseSchema> {
    local_peer_id: PeerId,
    connected_peers: Arc<Mutex<Vec<PeerInfo>>>,
    config: Option<NetworkConfig>,
    is_running: bool,
    event_sender: broadcast::Sender<NetworkEvent<K, V>>,
    listening_addresses: Vec<Multiaddr>,
    dht_mode: KademliaDhtMode,
    dht_mode_stats: DhtModeStats,
}

impl<K: NetabaseSchemaKey, V: NetabaseSchema> MockNetwork<K, V> {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            local_peer_id: PeerId::random(),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
            config: None,
            is_running: false,
            event_sender,
            listening_addresses: vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()],
            dht_mode: KademliaDhtMode::default(),
            dht_mode_stats: DhtModeStats {
                current_mode: KademliaDhtMode::default(),
                mode_switches_count: 0,
                time_in_server_mode: Duration::from_secs(0),
                time_in_client_mode: Duration::from_secs(0),
                records_stored: 0,
                queries_answered: 0,
                auto_mode_triggers: vec![],
            },
        }
    }
}

#[async_trait]
impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> NetabaseNetwork<K, V>
    for MockNetwork<K, V>
{
    async fn initialize(&mut self, config: NetworkConfig) -> NetworkResult<()> {
        self.config = Some(config);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.config.is_some()
    }

    async fn start(&mut self) -> NetworkResult<()> {
        if self.is_running {
            return Err(NetworkError::NetworkAlreadyStarted);
        }
        self.is_running = true;
        Ok(())
    }

    async fn stop(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NetworkNotStarted);
        }
        self.is_running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn local_peer_id(&self) -> NetworkResult<PeerId> {
        Ok(self.local_peer_id)
    }

    fn listening_addresses(&self) -> NetworkResult<Vec<Multiaddr>> {
        Ok(self.listening_addresses.clone())
    }

    async fn add_listening_address(&mut self, address: Multiaddr) -> NetworkResult<()> {
        self.listening_addresses.push(address.clone());
        let _ = self
            .event_sender
            .send(NetworkEvent::NewListenAddr { address });
        Ok(())
    }

    async fn remove_listening_address(&mut self, address: &Multiaddr) -> NetworkResult<bool> {
        let initial_len = self.listening_addresses.len();
        self.listening_addresses.retain(|addr| addr != address);
        let removed = self.listening_addresses.len() < initial_len;

        if removed {
            let _ = self.event_sender.send(NetworkEvent::ExpiredListenAddr {
                address: address.clone(),
            });
        }

        Ok(removed)
    }

    async fn connect_peer(&mut self, peer_id: PeerId, address: Multiaddr) -> NetworkResult<()> {
        let peer_info = PeerInfo {
            peer_id,
            addresses: vec![address.clone()],
            connected_address: Some(address),
            connection_established: Utc::now(),
            user_agent: Some("MockPeer/1.0.0".to_string()),
            protocol_version: Some("/mock/1.0.0".to_string()),
            supported_protocols: vec!["/mock/1.0.0".to_string()],
            is_connected: true,
            connection_count: 1,
        };

        {
            let mut peers = self.connected_peers.lock().unwrap();
            peers.push(peer_info.clone());
        }

        let _ = self
            .event_sender
            .send(NetworkEvent::PeerConnected { peer_info });
        Ok(())
    }

    async fn disconnect_peer(&mut self, peer_id: &PeerId) -> NetworkResult<()> {
        {
            let mut peers = self.connected_peers.lock().unwrap();
            peers.retain(|peer| peer.peer_id != *peer_id);
        }

        let _ = self.event_sender.send(NetworkEvent::PeerDisconnected {
            peer_id: *peer_id,
            reason: Some("Manual disconnect".to_string()),
        });
        Ok(())
    }

    async fn connected_peers(&self) -> NetworkResult<Vec<PeerInfo>> {
        let peers = self.connected_peers.lock().unwrap();
        Ok(peers.clone())
    }

    async fn peer_info(&self, peer_id: &PeerId) -> NetworkResult<Option<PeerInfo>> {
        let peers = self.connected_peers.lock().unwrap();
        Ok(peers.iter().find(|peer| peer.peer_id == *peer_id).cloned())
    }

    async fn send_message(
        &mut self,
        peer_id: &PeerId,
        message: NetworkMessage<K, V>,
    ) -> NetworkResult<()> {
        // Mock message sending
        let _ = self.event_sender.send(NetworkEvent::MessageReceived {
            peer_id: *peer_id,
            message,
        });
        Ok(())
    }

    async fn broadcast_message(
        &mut self,
        _message: NetworkMessage<K, V>,
        options: BroadcastOptions,
    ) -> NetworkResult<()> {
        let peer_count = self.connected_peers.lock().unwrap().len();
        let _ = self.event_sender.send(NetworkEvent::MessageBroadcasted {
            topic: options.topic.unwrap_or_else(|| "default".to_string()),
            message_size: 1024, // Mock size
            peer_count,
        });
        Ok(())
    }

    async fn subscribe_topic(&mut self, _topic: &str) -> NetworkResult<()> {
        Ok(())
    }

    async fn unsubscribe_topic(&mut self, _topic: &str) -> NetworkResult<()> {
        Ok(())
    }

    fn subscribed_topics(&self) -> NetworkResult<Vec<String>> {
        Ok(vec!["default".to_string()])
    }

    async fn publish_topic(&mut self, _topic: &str, _data: Vec<u8>) -> NetworkResult<()> {
        Ok(())
    }

    async fn dht_put(&mut self, _key: String, _value: Vec<u8>) -> NetworkResult<()> {
        Ok(())
    }

    async fn dht_get(&mut self, _key: &str) -> NetworkResult<Option<Vec<u8>>> {
        Ok(Some(b"mock_value".to_vec()))
    }

    async fn dht_add_address(
        &mut self,
        _peer_id: PeerId,
        _address: Multiaddr,
    ) -> NetworkResult<()> {
        Ok(())
    }

    async fn dht_get_addresses(&mut self, _peer_id: &PeerId) -> NetworkResult<Vec<Multiaddr>> {
        Ok(vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()])
    }

    async fn bootstrap(&mut self) -> NetworkResult<()> {
        let peer_count = self.connected_peers.lock().unwrap().len();
        let _ = self.event_sender.send(NetworkEvent::BootstrapCompleted {
            connected_peers: peer_count,
        });
        Ok(())
    }

    async fn stats(&self) -> NetworkResult<NetworkStats> {
        Ok(NetworkStats {
            local_peer_id: self.local_peer_id,
            listening_addresses: self.listening_addresses.clone(),
            connected_peers: self.connected_peers.lock().unwrap().len(),
            pending_connections: 0,
            total_connections_established: 10,
            total_connections_closed: 5,
            bytes_sent: 1024,
            bytes_received: 2048,
            messages_sent: 50,
            messages_received: 75,
            uptime: Duration::from_secs(3600),
            dht_routing_table_size: 100,
            gossipsub_topics: vec!["default".to_string()],
        })
    }

    fn event_receiver(&self) -> NetworkResult<broadcast::Receiver<NetworkEvent<K, V>>> {
        Ok(self.event_sender.subscribe())
    }

    async fn set_dht_mode(&mut self, mode: KademliaDhtMode) -> NetworkResult<()> {
        let old_mode = self.dht_mode.clone();
        self.dht_mode = mode.clone();

        // Update stats
        self.dht_mode_stats.current_mode = mode.clone();
        self.dht_mode_stats.mode_switches_count += 1;

        // Emit DHT status change event
        let _ = self.event_sender.send(NetworkEvent::NetworkError {
            error: NetworkError::ProtocolError {
                message: format!("DHT mode changed from {:?} to {:?}", old_mode, mode),
            },
        });

        Ok(())
    }

    fn get_dht_mode(&self) -> NetworkResult<KademliaDhtMode> {
        Ok(self.dht_mode.clone())
    }

    fn is_dht_server(&self) -> NetworkResult<bool> {
        Ok(matches!(self.dht_mode, KademliaDhtMode::Server))
    }

    fn is_dht_client(&self) -> NetworkResult<bool> {
        Ok(matches!(self.dht_mode, KademliaDhtMode::Client))
    }
}

#[async_trait]
impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> NetabaseNetworkExt<K, V>
    for MockNetwork<K, V>
{
    async fn discover_mdns_peers(&mut self) -> NetworkResult<Vec<PeerInfo>> {
        // Mock implementation - return empty list
        Ok(vec![])
    }

    async fn set_custom_protocols(&mut self, _protocols: Vec<String>) -> NetworkResult<()> {
        Ok(())
    }

    async fn dht_get_closest_peers(&mut self, _key: &str) -> NetworkResult<Vec<PeerId>> {
        let peers = self.connected_peers.lock().unwrap();
        Ok(peers.iter().take(3).map(|p| p.peer_id).collect())
    }

    async fn dht_get_providers(&mut self, _key: &str) -> NetworkResult<Vec<PeerId>> {
        Ok(vec![])
    }

    async fn dht_start_providing(&mut self, _key: &str) -> NetworkResult<()> {
        if matches!(self.dht_mode, KademliaDhtMode::Server) {
            self.dht_mode_stats.records_stored += 1;
            Ok(())
        } else {
            Err(NetworkError::ProtocolError {
                message: "DHT is in client mode, cannot provide records".to_string(),
            })
        }
    }

    async fn dht_stop_providing(&mut self, _key: &str) -> NetworkResult<()> {
        Ok(())
    }

    async fn ban_peer(&mut self, _peer_id: &PeerId, _duration: Duration) -> NetworkResult<()> {
        Ok(())
    }

    async fn unban_peer(&mut self, _peer_id: &PeerId) -> NetworkResult<()> {
        Ok(())
    }

    fn banned_peers(&self) -> NetworkResult<Vec<PeerId>> {
        Ok(vec![])
    }

    async fn set_connection_limits(
        &mut self,
        _max_connections: Option<u32>,
        _max_pending: Option<u32>,
    ) -> NetworkResult<()> {
        Ok(())
    }

    fn connection_limits(&self) -> NetworkResult<(Option<u32>, Option<u32>)> {
        Ok((Some(4), Some(256)))
    }

    async fn configure_protocols(&mut self, config: ProtocolConfig) -> NetworkResult<()> {
        if let Some(dht_mode) = config.kademlia_dht_mode {
            self.set_dht_mode(dht_mode).await?;
        }
        Ok(())
    }

    async fn toggle_dht_mode_auto(&mut self) -> NetworkResult<KademliaDhtMode> {
        let new_mode = match self.dht_mode {
            KademliaDhtMode::Server => {
                // Mock logic: switch to client if we have few connections
                let peer_count = self.connected_peers.lock().unwrap().len();
                if peer_count < 3 {
                    self.dht_mode_stats
                        .auto_mode_triggers
                        .push("Low peer count".to_string());
                    KademliaDhtMode::Client
                } else {
                    KademliaDhtMode::Server
                }
            }
            KademliaDhtMode::Client => {
                // Mock logic: switch to server if we have good connections
                let peer_count = self.connected_peers.lock().unwrap().len();
                if peer_count > 5 {
                    self.dht_mode_stats
                        .auto_mode_triggers
                        .push("Good peer count".to_string());
                    KademliaDhtMode::Server
                } else {
                    KademliaDhtMode::Client
                }
            }
            KademliaDhtMode::Auto => KademliaDhtMode::Auto,
        };

        self.set_dht_mode(new_mode.clone()).await?;
        Ok(new_mode)
    }

    async fn force_dht_server_mode(&mut self) -> NetworkResult<()> {
        self.set_dht_mode(KademliaDhtMode::Server).await
    }

    async fn force_dht_client_mode(&mut self) -> NetworkResult<()> {
        self.set_dht_mode(KademliaDhtMode::Client).await
    }

    async fn get_dht_mode_stats(&self) -> NetworkResult<DhtModeStats> {
        Ok(self.dht_mode_stats.clone())
    }

    async fn health_check(&self) -> NetworkResult<NetworkHealth> {
        Ok(NetworkHealth {
            is_healthy: true,
            connected_peer_count: self.connected_peers.lock().unwrap().len(),
            min_required_peers: 3,
            bootstrap_status: BootstrapStatus::Completed,
            dht_status: match self.dht_mode {
                KademliaDhtMode::Server => DhtStatus::ServerMode,
                KademliaDhtMode::Client => DhtStatus::ClientMode,
                KademliaDhtMode::Auto => DhtStatus::Ready,
            },
            dht_mode: self.dht_mode.clone(),
            issues: vec![],
        })
    }
}

// =============================================================================
// Mock Configuration Implementation
// =============================================================================

/// Mock configuration implementation for testing and examples
pub struct MockConfiguration {
    data: Arc<Mutex<HashMap<String, String>>>,
    metadata: ConfigurationMetadata,
    event_sender: broadcast::Sender<ConfigurationEvent>,
}

impl MockConfiguration {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            metadata: ConfigurationMetadata {
                loaded_at: Utc::now(),
                source: ConfigurationSource::Default,
                version: "1.0.0".to_string(),
                checksum: "mock_checksum".to_string(),
                is_valid: true,
                validation_errors: vec![],
            },
            event_sender,
        }
    }
}

#[async_trait]
impl NetabaseConfiguration for MockConfiguration {
    async fn load(&mut self, _options: ConfigurationOptions) -> ConfigurationResult<()> {
        // Mock loading some default configuration
        let mut data = self.data.lock().unwrap();
        data.insert("database.path".to_string(), "./mock_data".to_string());
        data.insert("network.port".to_string(), "4001".to_string());
        data.insert(
            "network.user_agent".to_string(),
            "MockNet/1.0.0".to_string(),
        );
        Ok(())
    }

    async fn reload(&mut self) -> ConfigurationResult<()> {
        // Mock reload
        let _ = self.event_sender.send(ConfigurationEvent::Loaded {
            source: ConfigurationSource::Default,
        });
        Ok(())
    }

    async fn save<P: AsRef<std::path::Path> + Send>(
        &self,
        _path: P,
        _format: FileFormat,
    ) -> ConfigurationResult<()> {
        Ok(())
    }

    async fn validate(
        &self,
    ) -> ConfigurationResult<Vec<crate::traits::configuration::ConfigurationError>> {
        Ok(vec![])
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn metadata(&self) -> ConfigurationMetadata {
        self.metadata.clone()
    }

    fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> ConfigurationResult<Option<T>> {
        let data = self.data.lock().unwrap();
        if let Some(value_str) = data.get(key) {
            match serde_json::from_str(value_str) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> ConfigurationResult<()> {
        let value_str = serde_json::to_string(&value).map_err(|e| {
            crate::traits::configuration::ConfigurationError::SerializationError {
                source: Box::new(e),
            }
        })?;

        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), value_str);
        Ok(())
    }

    fn contains_key(&self, key: &str) -> bool {
        let data = self.data.lock().unwrap();
        data.contains_key(key)
    }

    fn keys(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.keys().cloned().collect()
    }

    fn to_map(&self) -> HashMap<String, String> {
        let data = self.data.lock().unwrap();
        data.clone()
    }

    fn clear(&mut self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }

    fn merge(
        &mut self,
        _other: &dyn NetabaseConfiguration,
        _strategy: MergeStrategy,
    ) -> ConfigurationResult<()> {
        Ok(())
    }

    fn snapshot(&self) -> ConfigurationResult<Box<dyn NetabaseConfiguration>> {
        let mut new_config = MockConfiguration::new();
        let data = self.data.lock().unwrap();
        let mut new_data = new_config.data.lock().unwrap();
        *new_data = data.clone();
        Ok(Box::new(new_config))
    }

    fn restore(&mut self, _snapshot: Box<dyn NetabaseConfiguration>) -> ConfigurationResult<()> {
        Ok(())
    }

    async fn subscribe_changes(
        &self,
    ) -> ConfigurationResult<broadcast::Receiver<ConfigurationEvent>> {
        Ok(self.event_sender.subscribe())
    }

    async fn start_watching(&mut self) -> ConfigurationResult<()> {
        Ok(())
    }

    async fn stop_watching(&mut self) -> ConfigurationResult<()> {
        Ok(())
    }

    fn export(&self, _format: FileFormat) -> ConfigurationResult<String> {
        let data = self.data.lock().unwrap();
        Ok(serde_json::to_string_pretty(&*data).unwrap())
    }

    fn import(&mut self, data_str: &str, _format: FileFormat) -> ConfigurationResult<()> {
        let imported_data: HashMap<String, String> =
            serde_json::from_str(data_str).map_err(|e| {
                crate::traits::configuration::ConfigurationError::SerializationError {
                    source: Box::new(e),
                }
            })?;

        let mut data = self.data.lock().unwrap();
        *data = imported_data;
        Ok(())
    }
}

// =============================================================================
// Complete Netabase Implementation
// =============================================================================

/// Complete Netabase implementation that composes all subsystems
pub struct ExampleNetabase<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> {
    database: Box<dyn NetabaseDatabase<K, V>>,
    network: Box<dyn NetabaseNetwork<K, V>>,
    configuration: Box<dyn NetabaseConfiguration>,
    state: SystemState,
    event_sender: broadcast::Sender<NetabaseEvent<K, V>>,
    config: Option<NetabaseConfig>,
    start_time: Option<DateTime<Utc>>,
}

impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> ExampleNetabase<K, V> {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self {
            database: Box::new(MockDatabase::new()),
            network: Box::new(MockNetwork::new()),
            configuration: Box::new(MockConfiguration::new()),
            state: SystemState::Uninitialized,
            event_sender,
            config: None,
            start_time: None,
        }
    }

    pub fn with_database(mut self, database: Box<dyn NetabaseDatabase<K, V>>) -> Self {
        self.database = database;
        self
    }

    pub fn with_network(mut self, network: Box<dyn NetabaseNetwork<K, V>>) -> Self {
        self.network = network;
        self
    }

    pub fn with_configuration(mut self, configuration: Box<dyn NetabaseConfiguration>) -> Self {
        self.configuration = configuration;
        self
    }

    fn transition_state(&mut self, new_state: SystemState) {
        let old_state = self.state.clone();
        self.state = new_state.clone();

        let _ = self.event_sender.send(NetabaseEvent::StateChanged {
            old_state,
            new_state,
        });
    }
}

#[async_trait]
impl<K: NetabaseSchemaKey + 'static, V: NetabaseSchema + 'static> NetabaseCore<K, V>
    for ExampleNetabase<K, V>
{
    async fn initialize(&mut self, config: NetabaseConfig) -> NetabaseResult<()> {
        if self.state != SystemState::Uninitialized {
            return Err(NetabaseError::AlreadyInitialized);
        }

        self.transition_state(SystemState::Initializing);

        // Initialize all subsystems
        self.database
            .initialize(config.database_config.clone())
            .await?;
        self.network
            .initialize(config.network_config.clone())
            .await?;
        self.configuration
            .load(config.configuration_options.clone())
            .await?;

        self.config = Some(config);
        self.transition_state(SystemState::Initialized);
        Ok(())
    }

    async fn start(&mut self) -> NetabaseResult<()> {
        if self.state != SystemState::Initialized {
            return Err(NetabaseError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Running".to_string(),
                reason: "System must be initialized first".to_string(),
            });
        }

        self.transition_state(SystemState::Starting);
        self.network.start().await?;
        self.start_time = Some(Utc::now());
        self.transition_state(SystemState::Running);
        Ok(())
    }

    async fn stop(&mut self) -> NetabaseResult<()> {
        if self.state != SystemState::Running {
            return Err(NetabaseError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Stopped".to_string(),
                reason: "System is not running".to_string(),
            });
        }

        self.transition_state(SystemState::Stopping);
        self.network.stop().await?;
        self.transition_state(SystemState::Stopped);
        Ok(())
    }

    async fn shutdown(mut self) -> NetabaseResult<()> {
        if self.state == SystemState::Running {
            self.stop().await?;
        }
        self.database.close().await?;
        self.transition_state(SystemState::Shutdown);
        Ok(())
    }

    fn state(&self) -> SystemState {
        self.state.clone()
    }

    fn is_initialized(&self) -> bool {
        matches!(
            self.state,
            SystemState::Initialized | SystemState::Running | SystemState::Stopped
        )
    }

    fn is_running(&self) -> bool {
        self.state == SystemState::Running
    }

    async fn put(&mut self, key: K, value: V) -> NetabaseResult<()> {
        if !self.is_running() {
            return Err(NetabaseError::NotRunning);
        }

        let start_time = std::time::Instant::now();

        // Store locally first
        self.database.put(key.clone(), value.clone()).await?;

        // Then broadcast to network if needed
        if self.network.is_running() {
            let message = NetworkMessage::StoreRequest {
                key: key.clone(),
                value: value.clone(),
            };
            let options = BroadcastOptions::default();
            self.network.broadcast_message(message, options).await?;
        }

        let duration = start_time.elapsed();
        let _ = self.event_sender.send(NetabaseEvent::DataOperation {
            operation: crate::traits::core::DataOperationType::Put { key, value },
            success: true,
            duration,
        });

        Ok(())
    }

    async fn put_with_sync(
        &mut self,
        key: K,
        value: V,
        sync_immediately: bool,
    ) -> NetabaseResult<()> {
        self.database.put(key.clone(), value.clone()).await?;

        if sync_immediately && self.network.is_running() {
            let message = NetworkMessage::StoreRequest { key, value };
            let options = BroadcastOptions::default();
            self.network.broadcast_message(message, options).await?;
        }

        Ok(())
    }

    async fn get(&self, key: &K) -> NetabaseResult<Option<V>> {
        self.database.get(key).await
    }
}
