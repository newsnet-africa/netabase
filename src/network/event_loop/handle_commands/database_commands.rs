use crate::{
    netabase_trait::{NetabaseSchema, NetabaseSchemaKey},
    network::behaviour::NetabaseBehaviour,
    network::event_messages::command_messages::{
        CommandResponse, DatabaseResponse, database_commands::DatabaseCommand,
    },
    traits::database::{DatabaseConfig, QueryOptions},
};
use bincode;
use libp2p::{
    Swarm,
    kad::{self, QueryId},
};
use std::collections::HashMap;
use tokio::sync::oneshot;

/// Context for tracking database operations that use DHT
#[derive(Debug, Clone)]
pub enum DatabaseOperationContext {
    Put,
    Get,
    Delete,
    Contains,
}

/// Handle all database-level commands for distributed data storage
pub fn handle_database_command<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    command: DatabaseCommand<K, V>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    match command {
        // Core CRUD operations on user data
        DatabaseCommand::Put { key, value } => {
            handle_put(
                key,
                value,
                response_sender,
                query_queue,
                database_context,
                swarm,
            );
        }
        DatabaseCommand::Get { key } => {
            handle_get(key, response_sender, query_queue, database_context, swarm);
        }
        DatabaseCommand::Delete { key } => {
            handle_delete(key, response_sender, query_queue, database_context, swarm);
        }
        DatabaseCommand::Contains { key } => {
            handle_contains(key, response_sender, query_queue, database_context, swarm);
        }

        // Batch operations for efficiency
        DatabaseCommand::PutBatch { entries } => {
            handle_put_batch(entries, response_sender);
        }
        DatabaseCommand::GetBatch { keys } => {
            handle_get_batch(keys, response_sender);
        }
        DatabaseCommand::DeleteBatch { keys } => {
            handle_delete_batch(keys, response_sender);
        }

        // Advanced data operations
        DatabaseCommand::Update { key, value } => {
            handle_update(key, value, response_sender);
        }
        DatabaseCommand::Upsert { key, value } => {
            handle_upsert(key, value, response_sender);
        }

        // Querying and scanning user data
        DatabaseCommand::ScanPrefix { prefix, options } => {
            handle_scan_prefix(prefix, options, response_sender);
        }
        DatabaseCommand::ScanRange {
            start,
            end,
            options,
        } => {
            handle_scan_range(start, end, options, response_sender);
        }
        DatabaseCommand::Keys { options } => {
            handle_keys(options, response_sender);
        }
        DatabaseCommand::Values { options } => {
            handle_values(options, response_sender);
        }
        DatabaseCommand::Entries { options } => {
            handle_entries(options, response_sender);
        }
        DatabaseCommand::Len => {
            handle_len(response_sender);
        }
        DatabaseCommand::IsEmpty => {
            handle_is_empty(response_sender);
        }

        // Transaction operations for data consistency
        DatabaseCommand::BeginTransaction => {
            handle_begin_transaction(response_sender);
        }
        DatabaseCommand::CommitTransaction { transaction_id } => {
            handle_commit_transaction(transaction_id, response_sender);
        }
        DatabaseCommand::RollbackTransaction { transaction_id } => {
            handle_rollback_transaction(transaction_id, response_sender);
        }

        // Database maintenance operations
        DatabaseCommand::Compact => {
            handle_compact(response_sender);
        }
        DatabaseCommand::Stats => {
            handle_stats(response_sender);
        }

        // Database lifecycle
        DatabaseCommand::Initialize { config } => {
            handle_initialize(config, response_sender);
        }
        DatabaseCommand::Close => {
            handle_close(response_sender);
        }

        // Data replication and sync
        DatabaseCommand::SyncData { peer_id } => {
            handle_sync_data(peer_id, response_sender);
        }
        DatabaseCommand::ReplicateKey { key, target_peers } => {
            handle_replicate_key(key, target_peers, response_sender);
        }

        // Data integrity
        DatabaseCommand::VerifyIntegrity => {
            handle_verify_integrity(response_sender);
        }
        DatabaseCommand::RepairCorruption { keys } => {
            handle_repair_corruption(keys, response_sender);
        }

        // Change monitoring
        DatabaseCommand::Subscribe { key } => {
            handle_subscribe(key, response_sender);
        }
        DatabaseCommand::Unsubscribe { key } => {
            handle_unsubscribe(key, response_sender);
        }
    }
}

// Core CRUD operations

fn handle_put<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    key: K,
    value: V,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    log::info!("Database put operation for key: {:?}", key);

    // Use NetabaseSchema Into<Record> trait to convert value to DHT record
    let record: kad::Record = value.into();

    if let Some(sender) = response_sender {
        // Store the record in DHT with quorum
        match swarm
            .behaviour_mut()
            .kad
            .put_record(record, kad::Quorum::One)
        {
            Ok(query_id) => {
                log::debug!("Started DHT put operation with query ID: {:?}", query_id);
                query_queue.insert(query_id, sender);
                database_context.insert(query_id, DatabaseOperationContext::Put);
            }
            Err(e) => {
                log::error!("Failed to start DHT put operation: {}", e);
                let _ = sender.send(CommandResponse::Error(format!(
                    "Failed to start DHT put: {}",
                    e
                )));
            }
        }
    } else {
        // Fire and forget
        let _ = swarm
            .behaviour_mut()
            .kad
            .put_record(record, kad::Quorum::One);
    }
}

fn handle_get<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    key: K,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    log::info!("Database get operation for key: {:?}", key);

    // Use NetabaseSchemaKey Into<RecordKey> trait to convert key
    let record_key: kad::RecordKey = key.into();

    if let Some(sender) = response_sender {
        // Query the DHT for the record
        let query_id = swarm.behaviour_mut().kad.get_record(record_key);

        log::debug!("Started DHT get operation with query ID: {:?}", query_id);
        query_queue.insert(query_id, sender);
        database_context.insert(query_id, DatabaseOperationContext::Get);
    } else {
        // Fire and forget
        swarm.behaviour_mut().kad.get_record(record_key);
    }
}

fn handle_delete<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    key: K,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    log::info!("Database delete operation for key: {:?}", key);

    // Use NetabaseSchemaKey Into<RecordKey> trait to convert key
    let record_key: kad::RecordKey = key.into();

    // For delete operation, we put an empty record to indicate deletion
    // In a more sophisticated implementation, you might use tombstones
    let record = kad::Record {
        key: record_key,
        value: vec![], // Empty value indicates deletion
        publisher: None,
        expires: None,
    };

    if let Some(sender) = response_sender {
        match swarm
            .behaviour_mut()
            .kad
            .put_record(record, kad::Quorum::One)
        {
            Ok(query_id) => {
                log::debug!("Started DHT delete operation with query ID: {:?}", query_id);
                query_queue.insert(query_id, sender);
                database_context.insert(query_id, DatabaseOperationContext::Delete);
            }
            Err(e) => {
                log::error!("Failed to start DHT delete operation: {}", e);
                let _ = sender.send(CommandResponse::Error(format!(
                    "Failed to start DHT delete: {}",
                    e
                )));
            }
        }
    } else {
        let _ = swarm
            .behaviour_mut()
            .kad
            .put_record(record, kad::Quorum::One);
    }
}

fn handle_contains<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    key: K,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    log::info!("Database contains check for key: {:?}", key);

    // Contains check is implemented as a get operation - if we find the record, it exists
    let record_key: kad::RecordKey = key.into();

    if let Some(sender) = response_sender {
        let query_id = swarm.behaviour_mut().kad.get_record(record_key);

        log::debug!("Started DHT contains check with query ID: {:?}", query_id);
        // The response will be handled in the event handler and converted to a boolean result
        query_queue.insert(query_id, sender);
        database_context.insert(query_id, DatabaseOperationContext::Contains);
    } else {
        swarm.behaviour_mut().kad.get_record(record_key);
    }
}

// Helper functions for converting DHT responses to database responses

/// Convert a DHT get response to a database get response
pub fn convert_dht_get_to_database_response<
    K: NetabaseSchemaKey + std::fmt::Debug,
    V: NetabaseSchema,
>(
    dht_result: Result<kad::GetRecordOk, kad::GetRecordError>,
) -> CommandResponse<K, V> {
    match dht_result {
        Ok(kad::GetRecordOk::FoundRecord(peer_record)) => {
            // Try to deserialize the value back to the original type
            if peer_record.record.value.is_empty() {
                // Empty value indicates a deleted record
                CommandResponse::Database(DatabaseResponse::GetResult(None))
            } else {
                match bincode::decode_from_slice::<V, bincode::config::Configuration>(
                    &peer_record.record.value,
                    bincode::config::standard(),
                ) {
                    Ok(value) => {
                        CommandResponse::Database(DatabaseResponse::GetResult(Some(value.0)))
                    }
                    Err(e) => CommandResponse::Error(format!("Failed to decode value: {:?}", e)),
                }
            }
        }
        Ok(kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. }) => {
            // No record found
            CommandResponse::Database(DatabaseResponse::GetResult(None))
        }
        Err(kad::GetRecordError::NotFound { .. }) => {
            // Record not found
            CommandResponse::Database(DatabaseResponse::GetResult(None))
        }
        Err(kad::GetRecordError::QuorumFailed { .. }) => {
            CommandResponse::Error("Failed to reach quorum for get operation".to_string())
        }
        Err(kad::GetRecordError::Timeout { .. }) => {
            CommandResponse::Error("Get operation timed out".to_string())
        }
    }
}

/// Convert a DHT get response to a database contains response
pub fn convert_dht_get_to_contains_response<
    K: NetabaseSchemaKey + std::fmt::Debug,
    V: NetabaseSchema,
>(
    dht_result: Result<kad::GetRecordOk, kad::GetRecordError>,
) -> CommandResponse<K, V> {
    match dht_result {
        Ok(kad::GetRecordOk::FoundRecord(peer_record)) => {
            // Record exists if value is not empty (empty means deleted)
            let exists = !peer_record.record.value.is_empty();
            CommandResponse::Database(DatabaseResponse::ExistsResult(exists))
        }
        Ok(kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. }) => {
            // No record found
            CommandResponse::Database(DatabaseResponse::ExistsResult(false))
        }
        Err(kad::GetRecordError::NotFound { .. }) => {
            // Record not found
            CommandResponse::Database(DatabaseResponse::ExistsResult(false))
        }
        Err(kad::GetRecordError::QuorumFailed { .. }) => {
            CommandResponse::Error("Failed to reach quorum for contains operation".to_string())
        }
        Err(kad::GetRecordError::Timeout { .. }) => {
            CommandResponse::Error("Contains operation timed out".to_string())
        }
    }
}

/// Convert a DHT put response to a database response
pub fn convert_dht_put_to_database_response<
    K: NetabaseSchemaKey + std::fmt::Debug,
    V: NetabaseSchema,
>(
    dht_result: Result<kad::PutRecordOk, kad::PutRecordError>,
) -> CommandResponse<K, V> {
    match dht_result {
        Ok(_) => CommandResponse::Success,
        Err(kad::PutRecordError::QuorumFailed { .. }) => {
            CommandResponse::Error("Failed to reach quorum for put operation".to_string())
        }
        Err(kad::PutRecordError::Timeout { .. }) => {
            CommandResponse::Error("Put operation timed out".to_string())
        }
    }
}

/// Convert a DHT put response to a database delete response
pub fn convert_dht_put_to_delete_response<
    K: NetabaseSchemaKey + std::fmt::Debug,
    V: NetabaseSchema,
>(
    dht_result: Result<kad::PutRecordOk, kad::PutRecordError>,
) -> CommandResponse<K, V> {
    match dht_result {
        Ok(_) => CommandResponse::Database(DatabaseResponse::DeleteResult(true)),
        Err(kad::PutRecordError::QuorumFailed { .. }) => {
            CommandResponse::Error("Failed to reach quorum for delete operation".to_string())
        }
        Err(kad::PutRecordError::Timeout { .. }) => {
            CommandResponse::Error("Delete operation timed out".to_string())
        }
    }
}

// Batch operations

fn handle_put_batch<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    entries: Vec<(K, V)>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement efficient batch put operation
    log::info!(
        "Database batch put operation with {} entries",
        entries.len()
    );

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_get_batch<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    keys: Vec<K>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement efficient batch get operation
    log::info!("Database batch get operation for {} keys", keys.len());

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_delete_batch<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    keys: Vec<K>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement efficient batch delete operation
    log::info!("Database batch delete operation for {} keys", keys.len());

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Advanced operations

fn handle_update<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _key: K,
    _value: V,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement update operation (must exist)
    log::info!("Database update operation");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_upsert<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _key: K,
    _value: V,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement upsert operation (insert or update)
    log::info!("Database upsert operation");

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Querying operations

fn handle_scan_prefix<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    prefix: String,
    _options: Option<QueryOptions>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement prefix scanning across distributed data
    log::info!("Database scan prefix operation for: {}", prefix);

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_scan_range<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _start: K,
    _end: K,
    _options: Option<QueryOptions>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement range scanning across distributed data
    log::info!("Database scan range operation");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_keys<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _options: Option<QueryOptions>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed key enumeration
    log::info!("Database keys enumeration");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_values<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _options: Option<QueryOptions>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed value enumeration
    log::info!("Database values enumeration");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_entries<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _options: Option<QueryOptions>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed entry enumeration
    log::info!("Database entries enumeration");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_len<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed count operation
    log::info!("Database length operation");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_is_empty<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed empty check
    log::info!("Database is empty check");

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Transaction operations

fn handle_begin_transaction<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed transaction begin
    log::info!("Database begin transaction");

    if let Some(sender) = response_sender {
        let transaction_id = format!("tx_{}", uuid::Uuid::new_v4());
        todo!();
    }
}

fn handle_commit_transaction<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    transaction_id: String,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed transaction commit
    log::info!("Database commit transaction: {}", transaction_id);

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_rollback_transaction<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    transaction_id: String,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement distributed transaction rollback
    log::info!("Database rollback transaction: {}", transaction_id);

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Maintenance operations

fn handle_compact<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement database compaction
    log::info!("Database compaction operation");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_stats<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement database statistics collection
    log::info!("Database statistics operation");

    if let Some(sender) = response_sender {
        let stats = crate::traits::database::DatabaseStats {
            total_entries: 0,
            total_size: 0,
            last_compaction: None,
            cache_hit_rate: 0.0,
            average_entry_size: 0,
            total_kad_records: 0,
            records_pending_republish: 0,
            records_expiring_soon: 0,
        };
        todo!();
    }
}

// Lifecycle operations

fn handle_initialize<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _config: DatabaseConfig,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement database initialization
    log::info!("Database initialization");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_close<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement database shutdown
    log::info!("Database close operation");

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Replication and sync

fn handle_sync_data<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    peer_id: Option<libp2p::PeerId>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement data synchronization with peers
    match peer_id {
        Some(peer) => log::info!("Database sync with specific peer: {}", peer),
        None => log::info!("Database sync with all peers"),
    }

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_replicate_key<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _key: K,
    target_peers: Vec<libp2p::PeerId>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement key replication to specific peers
    log::info!("Database replicate key to {} peers", target_peers.len());

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Data integrity

fn handle_verify_integrity<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement data integrity verification
    log::info!("Database integrity verification");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_repair_corruption<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    keys: Vec<K>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement corruption repair for specific keys
    log::info!("Database repair corruption for {} keys", keys.len());

    if let Some(sender) = response_sender {
        todo!();
    }
}

// Change monitoring

fn handle_subscribe<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _key: K,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement change subscription for key
    log::info!("Database subscribe to key changes");

    if let Some(sender) = response_sender {
        todo!();
    }
}

fn handle_unsubscribe<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    _key: K,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement unsubscribe from key changes
    log::info!("Database unsubscribe from key changes");

    if let Some(sender) = response_sender {
        todo!();
    }
}

/// Process a DHT response based on the database operation context
pub fn process_database_dht_response<K: NetabaseSchemaKey + std::fmt::Debug, V: NetabaseSchema>(
    query_id: QueryId,
    result: &kad::QueryResult,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
) -> Option<CommandResponse<K, V>> {
    if let Some(operation_context) = database_context.remove(&query_id) {
        match result {
            kad::QueryResult::PutRecord(put_result) => match operation_context {
                DatabaseOperationContext::Put | DatabaseOperationContext::Delete => {
                    Some(convert_dht_put_to_database_response(put_result.clone()))
                }
                _ => None,
            },
            kad::QueryResult::GetRecord(get_result) => match operation_context {
                DatabaseOperationContext::Get => {
                    Some(convert_dht_get_to_database_response(get_result.clone()))
                }
                DatabaseOperationContext::Contains => {
                    Some(convert_dht_get_to_contains_response(get_result.clone()))
                }
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    }
}
