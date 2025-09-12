use crate::{
    netabase_trait::{
        self, NetabaseRegistery, NetabaseRegistryKey, NetabaseSchema, NetabaseSchemaKey,
    },
    network::event_messages::command_messages::{
        configuration_commands::ConfigurationCommand, database_commands::DatabaseCommand,
        network_commands::NetworkCommand, system_commands::SystemCommand,
    },
    traits::network::{DhtModeStats, KademliaDhtMode},
};
use libp2p::{Multiaddr, PeerId, kad};

use tokio::sync::oneshot;

pub enum NetabaseCommand<R: NetabaseRegistery> {
    System(SystemCommand),
    Database(DatabaseCommand<R>),
    Network(NetworkCommand<R>),
    Configuration(ConfigurationCommand),
    Close,
}

pub enum CommandResponse<R: NetabaseRegistery> {
    Database(DatabaseResponse<R>),
    Network(NetworkResponse<R>),
    Configuration(ConfigurationResponse),
    System(SystemResponse),
    Success,
    Error(String),
}

pub enum DatabaseResponse<R: NetabaseRegistery> {
    GetResult(Option<R>),
    BatchGetResult(std::collections::HashMap<R::KeyRegistry, R>),

    DeleteResult(bool),
    BatchDeleteResult(Vec<R::KeyRegistry>),
    KeysResult(Vec<R::KeyRegistry>),
    ValuesResult(Vec<R>),
    EntriesResult(Vec<(R::KeyRegistry, R)>),
    LenResult(usize),
    IsEmptyResult(bool),
    Stats(crate::traits::database::DatabaseStats),
    TransactionId(String),
    IntegrityReport(Vec<R::KeyRegistry>), // Keys that failed integrity check
    SyncStatus(bool),                     // Whether sync completed successfully
}

pub enum NetworkResponse<R: NetabaseRegistery> {
    _Phantom(std::marker::PhantomData<(R::KeyRegistry, R)>),
    PeerInfo(Vec<crate::traits::network::PeerInfo>),
    Stats(crate::traits::network::NetworkStats),
    LocalPeerId(PeerId),
    ListeningAddresses(Vec<Multiaddr>),

    DhtAddresses(Vec<Multiaddr>),
    SubscribedTopics(Vec<String>),
    DhtMode(KademliaDhtMode),
    IsDhtServer(bool),
    IsDhtClient(bool),
    DhtModeStats(DhtModeStats),
    DhtPutRecord(Result<kad::PutRecordOk, kad::PutRecordError>),
    DhtGetRecord(Result<kad::GetRecordOk, kad::GetRecordError>),
    DhtGetClosestPeers(Result<kad::GetClosestPeersOk, kad::GetClosestPeersError>),
    DhtGetProviders(Result<kad::GetProvidersOk, kad::GetProvidersError>),
    DhtStartProviding(Result<kad::AddProviderOk, kad::AddProviderError>),
    DhtBootstrap(Result<kad::BootstrapOk, kad::BootstrapError>),
    DhtRepublishRecord(Result<kad::PutRecordOk, kad::PutRecordError>),
    DhtRepublishProvider(Result<kad::AddProviderOk, kad::AddProviderError>),
}

pub enum ConfigurationResponse {
    Setting(String),
    AllSettings(std::collections::HashMap<String, String>),
    SectionSettings(std::collections::HashMap<String, String>),
    SettingExists(bool),
    IsValid(bool),
    ValidationErrors(Vec<String>),
    Profiles(Vec<String>),
}

/// System operation responses
pub enum SystemResponse {
    Health(crate::traits::core::SystemHealth),
    Stats(crate::traits::core::SystemStats),
    State(crate::traits::core::SystemState),
    PerformanceMetrics(crate::traits::core::PerformanceMetrics),
}

/// Command with response channel for operations that need to return results
pub struct CommandWithResponse<R: NetabaseRegistery> {
    pub command: NetabaseCommand<R>,
    pub response_sender: oneshot::Sender<CommandResponse<R>>,
}

pub mod database_commands {
    use crate::netabase_trait::{
        NetabaseRegistery, NetabaseRegistryKey, NetabaseSchema, NetabaseSchemaKey,
    };
    use crate::traits::database::{DatabaseConfig, QueryOptions};

    pub enum DatabaseCommand<R: NetabaseRegistery>
    where
        R::KeyRegistry: NetabaseRegistryKey,
    {
        // Core CRUD operations o user data
        Put {
            value: R,
        },
        Get {
            key: R::KeyRegistry,
        },
        Delete {
            key: R::KeyRegistry,
        },
        // Batch operations for efficiency
        PutBatch {
            entries: Vec<R>,
        },
        GetBatch {
            keys: Vec<R::KeyRegistry>,
        },
        DeleteBatch {
            keys: Vec<R::KeyRegistry>,
        },

        // Advanced data operations
        Update {
            value: R,
        },
        Upsert {
            value: R,
        },

        // Querying and scanning user data
        ScanPrefix {
            prefix: String,
            options: Option<QueryOptions>,
        },
        ScanRange {
            start: R::KeyRegistry,
            end: R::KeyRegistry,
            options: Option<QueryOptions>,
        },
        Keys {
            options: Option<QueryOptions>,
        },
        Values {
            options: Option<QueryOptions>,
        },
        Entries {
            options: Option<QueryOptions>,
        },
        Len,
        IsEmpty,

        // Transaction operations for data consistency
        BeginTransaction,
        CommitTransaction {
            transaction_id: String,
        },
        RollbackTransaction {
            transaction_id: String,
        },

        // Database maintenance operations
        Compact,
        Stats,

        // Database lifecycle
        Initialize {
            config: DatabaseConfig,
        },
        Close,

        // Data replication and sync
        SyncData {
            peer_id: Option<libp2p::PeerId>,
        },
        ReplicateKey {
            key: R::KeyRegistry,
            target_peers: Vec<libp2p::PeerId>,
        },

        // Data integrity
        VerifyIntegrity,
        RepairCorruption {
            keys: Vec<R::KeyRegistry>,
        },

        // Change monitoring
        Subscribe {
            key: R::KeyRegistry,
        },
        Unsubscribe {
            key: R::KeyRegistry,
        },
    }
}

pub mod network_commands {
    use crate::NetabaseRegistery;
    use crate::NetabaseRegistryKey;
    use crate::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
    use crate::traits::network::{BroadcastOptions, NetworkConfig, NetworkMessage, ProtocolConfig};
    use libp2p::{Multiaddr, PeerId};
    use std::time::Duration;

    pub enum NetworkCommand<R: NetabaseRegistery> {
        // Lifecycle
        Initialize {
            config: NetworkConfig,
        },
        Start,
        Stop,

        // Connection management
        ConnectPeer {
            peer_id: PeerId,
            address: Multiaddr,
        },
        DisconnectPeer {
            peer_id: PeerId,
        },
        AddListeningAddress {
            address: Multiaddr,
        },
        RemoveListeningAddress {
            address: Multiaddr,
        },

        // Messaging
        SendMessage {
            peer_id: PeerId,
            message: NetworkMessage<R>,
        },
        BroadcastMessage {
            message: NetworkMessage<R>,
            options: BroadcastOptions,
        },

        // Gossipsub operations
        SubscribeTopic {
            topic: String,
        },
        UnsubscribeTopic {
            topic: String,
        },
        PublishTopic {
            topic: String,
            data: Vec<u8>,
        },
        GetSubscribedTopics,

        // DHT operations
        DhtPut {
            key: String,
            value: Vec<u8>,
        },
        DhtGet {
            key: String,
        },
        DhtAddAddress {
            peer_id: PeerId,
            address: Multiaddr,
        },
        DhtGetAddresses {
            peer_id: PeerId,
        },
        DhtGetClosestPeers {
            key: String,
        },
        DhtGetProviders {
            key: String,
        },
        DhtStartProviding {
            key: String,
        },
        DhtStopProviding {
            key: String,
        },

        // Bootstrap and discovery
        Bootstrap,
        DiscoverMdnsPeers,

        // Information queries
        GetLocalPeerId,
        GetListeningAddresses,
        GetConnectedPeers,
        GetPeerInfo {
            peer_id: PeerId,
        },
        GetStats,
        HealthCheck,

        // Peer management
        BanPeer {
            peer_id: PeerId,
            duration: Duration,
        },
        UnbanPeer {
            peer_id: PeerId,
        },
        GetBannedPeers,

        // Configuration
        SetConnectionLimits {
            max_connections: Option<u32>,
            max_pending: Option<u32>,
        },
        GetConnectionLimits,
        ConfigureProtocols {
            config: ProtocolConfig,
        },
        SetCustomProtocols {
            protocols: Vec<String>,
        },

        // DHT mode operations
        SetDhtMode {
            mode: crate::traits::network::KademliaDhtMode,
        },
        GetDhtMode,
        IsDhtServer,
        IsDhtClient,
        ToggleDhtModeAuto,
        ForceDhtServerMode,
        ForceDhtClientMode,
        GetDhtModeStats,
    }
}

pub mod system_commands {
    use crate::traits::core::{ExportFormat, NetabaseConfig};
    use std::time::Duration;

    pub enum SystemCommand {
        // Lifecycle
        Initialize {
            config: NetabaseConfig,
        },
        Start,
        Stop,
        Shutdown,

        // State queries
        GetState,
        IsInitialized,
        IsRunning,

        // Health and monitoring
        HealthCheck,
        GetStats,
        GetPerformanceMetrics,
        StartMonitoring,
        StopMonitoring,

        // Data management
        Backup {
            backup_path: String,
        },
        Restore {
            backup_path: String,
        },
        Export {
            format: ExportFormat,
        },
        Import {
            data: Vec<u8>,
            format: ExportFormat,
        },

        // Optimization
        Optimize,
        CreateSnapshot,

        // Event handling
        RegisterEventHandler {
            handler_id: String,
        },
        UnregisterEventHandler {
            handler_id: String,
        },

        // Synchronization
        SyncAll,
        SyncKey {
            key: String,
        },

        // Utilities
        WaitForCondition {
            condition: String,
            timeout: Duration,
        },
    }
}

pub mod configuration_commands {
    use crate::traits::configuration::{ConfigurationOptions, FileFormat, MergeStrategy};
    use std::collections::HashMap;

    pub enum ConfigurationCommand {
        // Configuration file operations
        LoadFromFile {
            path: String,
            format: FileFormat,
        },
        SaveToFile {
            path: String,
            format: FileFormat,
        },
        ReloadFromFile,

        // Configuration loading with options
        Load {
            options: ConfigurationOptions,
        },

        // Individual setting management
        GetSetting {
            key: String,
        },
        SetSetting {
            key: String,
            value: String,
        },
        RemoveSetting {
            key: String,
        },
        HasSetting {
            key: String,
        },

        // Bulk setting operations
        GetAllSettings,
        UpdateSettings {
            settings: HashMap<String, String>,
        },
        ClearAllSettings,

        // Configuration sections
        GetSection {
            section: String,
        },
        SetSection {
            section: String,
            values: HashMap<String, String>,
        },
        RemoveSection {
            section: String,
        },

        // Environment and runtime configuration
        LoadEnvironmentOverrides,
        ApplyDefaults,
        SetDefault {
            key: String,
            value: String,
        },

        // Configuration validation
        Validate,
        ValidateSection {
            section: String,
        },

        // Configuration merging
        MergeConfiguration {
            other_config: HashMap<String, String>,
            strategy: MergeStrategy,
        },

        // File watching for configuration changes
        StartFileWatcher {
            paths: Vec<String>,
        },
        StopFileWatcher,

        // Configuration profiles/presets
        LoadProfile {
            profile_name: String,
        },
        SaveProfile {
            profile_name: String,
        },
        ListProfiles,

        // Configuration backup/restore (system settings only)
        BackupConfiguration {
            backup_path: String,
        },
        RestoreConfiguration {
            backup_path: String,
        },
    }
}
