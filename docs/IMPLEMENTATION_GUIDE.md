# Netabase Implementation Guide

This document provides technical approaches and implementation strategies for completing the TODO items in the Netabase crate.

## Critical Implementation Challenges

### 1. Complete Macro System Implementation (Highest Priority)

**Current Problem**: Macro system has placeholder implementations and incomplete code generation.

**Current State**: Basic procedural macro infrastructure exists but generates incomplete or incorrect code.

#### Implementation Strategy

**Step 1: Fix Serialization Macro Generation**

```rust
// In netabase_macros/src/serialization.rs
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

pub fn derive_netabase_serialize(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let serialize_impl = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_serializations = fields.named.iter().map(|field| {
                        let field_name = &field.ident;
                        let field_type = &field.ty;
                        
                        // Check for netabase attributes
                        let is_key = field.attrs.iter().any(|attr| {
                            attr.path().is_ident("netabase") && 
                            attr.tokens.to_string().contains("key")
                        });
                        
                        if is_key {
                            quote! {
                                writer.write_all(b"__KEY__")?;
                                bincode::serialize_into(&mut writer, &self.#field_name)?;
                            }
                        } else {
                            quote! {
                                bincode::serialize_into(&mut writer, &self.#field_name)?;
                            }
                        }
                    });

                    quote! {
                        fn serialize(&self) -> anyhow::Result<Vec<u8>> {
                            let mut buffer = Vec::new();
                            {
                                let mut writer = std::io::Cursor::new(&mut buffer);
                                #(#field_serializations)*
                            }
                            Ok(buffer)
                        }
                    }
                }
                _ => return syn::Error::new_spanned(name, "Only named fields are supported").to_compile_error(),
            }
        }
        _ => return syn::Error::new_spanned(name, "Only structs are supported").to_compile_error(),
    };

    quote! {
        impl #impl_generics crate::traits::NetabaseSerialize for #name #ty_generics #where_clause {
            #serialize_impl
        }
    }
}
```

**Step 2: Implement Fallible Conversion Macros**

```rust
// In netabase_macros/src/conversion.rs
pub fn derive_dht_record_conversion(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let key_field = find_key_field(&input.data)
        .expect("NetabaseSchema requires a field marked with #[netabase(key)]");

    quote! {
        impl #impl_generics TryFrom<libp2p::kad::Record> for #name #ty_generics #where_clause {
            type Error = NetabaseConversionError;

            fn try_from(record: libp2p::kad::Record) -> Result<Self, Self::Error> {
                let value: #name = bincode::deserialize(&record.value)
                    .map_err(|e| NetabaseConversionError::DeserializationFailed {
                        reason: e.to_string(),
                        record_key: String::from_utf8_lossy(&record.key).to_string(),
                    })?;

                // Validate key consistency
                let expected_key = value.#key_field.as_bytes();
                if record.key != expected_key {
                    return Err(NetabaseConversionError::KeyMismatch {
                        expected: String::from_utf8_lossy(expected_key).to_string(),
                        actual: String::from_utf8_lossy(&record.key).to_string(),
                    });
                }

                Ok(value)
            }
        }

        impl #impl_generics TryInto<libp2p::kad::Record> for #name #ty_generics #where_clause {
            type Error = NetabaseConversionError;

            fn try_into(self) -> Result<libp2p::kad::Record, Self::Error> {
                let key = self.#key_field.as_bytes().to_vec();
                let value = bincode::serialize(&self)
                    .map_err(|e| NetabaseConversionError::SerializationFailed {
                        reason: e.to_string(),
                        data_type: stringify!(#name).to_string(),
                    })?;

                Ok(libp2p::kad::Record {
                    key: libp2p::kad::record::Key::new(&key),
                    value,
                    publisher: None,
                    expires: None,
                })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetabaseConversionError {
    #[error("Failed to deserialize record: {reason} (key: {record_key})")]
    DeserializationFailed { reason: String, record_key: String },
    
    #[error("Failed to serialize data: {reason} (type: {data_type})")]
    SerializationFailed { reason: String, data_type: String },
    
    #[error("Key mismatch: expected '{expected}', got '{actual}'")]
    KeyMismatch { expected: String, actual: String },
    
    #[error("Invalid schema: {reason}")]
    InvalidSchema { reason: String },
}

fn find_key_field(data: &Data) -> Option<&syn::Ident> {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    fields.named.iter().find_map(|field| {
                        let has_key_attr = field.attrs.iter().any(|attr| {
                            attr.path().is_ident("netabase") && 
                            attr.tokens.to_string().contains("key")
                        });
                        
                        if has_key_attr {
                            field.ident.as_ref()
                        } else {
                            None
                        }
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}
```

**Step 3: Clean Up Macro Code Generation**

```rust
// In netabase_macros/src/lib.rs
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod serialization;
mod conversion;
mod validation;

#[proc_macro_derive(NetabaseSchema, attributes(netabase))]
pub fn derive_netabase_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Validate input structure
    if let Err(error) = validation::validate_schema_input(&input) {
        return error.to_compile_error().into();
    }
    
    let serialize_impl = serialization::derive_netabase_serialize(input.clone());
    let conversion_impl = conversion::derive_dht_record_conversion(input.clone());
    
    let expanded = quote::quote! {
        #serialize_impl
        #conversion_impl
    };
    
    TokenStream::from(expanded)
}

#[proc_macro_derive(NetabaseSerialize)]
pub fn derive_netabase_serialize_only(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = serialization::derive_netabase_serialize(input);
    TokenStream::from(expanded)
}
```

### 2. Implement Core DHT Functionality

**Current Problem**: Basic CRUD operations exist but DHT integration is incomplete.

#### Implementation Strategy

**Complete Kademlia Integration**

```rust
// In src/network/dht.rs
use libp2p::{
    kad::{
        record::{Key, Record},
        AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
        GetRecordOk, GetProvidersOk,
    },
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId,
};

pub struct DhtBehavior {
    kademlia: Kademlia<MemoryStore>,
    pending_queries: HashMap<QueryId, PendingQuery>,
}

#[derive(Debug)]
pub enum PendingQuery {
    GetRecord { 
        key: String, 
        sender: oneshot::Sender<Result<Vec<u8>, DhtError>> 
    },
    PutRecord { 
        key: String, 
        sender: oneshot::Sender<Result<(), DhtError>> 
    },
    GetProviders { 
        key: String, 
        sender: oneshot::Sender<Result<Vec<PeerId>, DhtError>> 
    },
}

impl DhtBehavior {
    pub fn new(local_peer_id: PeerId) -> Self {
        let mut config = KademliaConfig::default();
        config.set_query_timeout(Duration::from_secs(60));
        config.set_replication_factor(NonZeroUsize::new(20).unwrap());
        config.set_publication_interval(Some(Duration::from_secs(86400))); // 24 hours
        
        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::with_config(local_peer_id, store, config);
        
        Self {
            kademlia,
            pending_queries: HashMap::new(),
        }
    }
    
    pub fn put_record(&mut self, key: String, value: Vec<u8>) -> oneshot::Receiver<Result<(), DhtError>> {
        let (sender, receiver) = oneshot::channel();
        
        let record = Record {
            key: Key::new(&key),
            value,
            publisher: None,
            expires: None,
        };
        
        let query_id = self.kademlia.put_record(record, Quorum::Majority);
        self.pending_queries.insert(
            query_id,
            PendingQuery::PutRecord { key, sender }
        );
        
        receiver
    }
    
    pub fn get_record(&mut self, key: String) -> oneshot::Receiver<Result<Vec<u8>, DhtError>> {
        let (sender, receiver) = oneshot::channel();
        
        let query_id = self.kademlia.get_record(Key::new(&key));
        self.pending_queries.insert(
            query_id,
            PendingQuery::GetRecord { key, sender }
        );
        
        receiver
    }
    
    pub fn bootstrap(&mut self) -> Result<QueryId, DhtError> {
        self.kademlia.bootstrap()
            .map_err(|e| DhtError::BootstrapFailed(e.to_string()))
    }
    
    pub fn add_address(&mut self, peer: &PeerId, address: Multiaddr) {
        self.kademlia.add_address(peer, address);
    }
}

impl NetworkBehaviour for DhtBehavior {
    type ConnectionHandler = <Kademlia<MemoryStore> as NetworkBehaviour>::ConnectionHandler;
    type ToSwarm = DhtEvent;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.kademlia.handle_established_inbound_connection(
            connection_id, peer, local_addr, remote_addr
        )
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.kademlia.handle_established_outbound_connection(
            connection_id, peer, addr, role_override
        )
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        self.kademlia.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        self.kademlia.on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        match self.kademlia.poll(cx, params) {
            Poll::Ready(ToSwarm::GenerateEvent(event)) => {
                if let Some(dht_event) = self.handle_kademlia_event(event) {
                    Poll::Ready(ToSwarm::GenerateEvent(dht_event))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(other) => Poll::Ready(other.map_out(|_| unreachable!())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl DhtBehavior {
    fn handle_kademlia_event(&mut self, event: KademliaEvent) -> Option<DhtEvent> {
        match event {
            KademliaEvent::OutboundQueryProgressed { id, result, .. } => {
                self.handle_query_result(id, result)
            }
            KademliaEvent::RoutingUpdated { peer, .. } => {
                Some(DhtEvent::PeerAdded(peer))
            }
            KademliaEvent::InboundRequest { request } => {
                // Handle inbound requests (get, put, etc.)
                None
            }
            _ => None,
        }
    }
    
    fn handle_query_result(&mut self, query_id: QueryId, result: QueryResult) -> Option<DhtEvent> {
        if let Some(pending_query) = self.pending_queries.remove(&query_id) {
            match (pending_query, result) {
                (PendingQuery::PutRecord { sender, .. }, QueryResult::PutRecord(Ok(_))) => {
                    let _ = sender.send(Ok(()));
                }
                (PendingQuery::PutRecord { sender, .. }, QueryResult::PutRecord(Err(e))) => {
                    let _ = sender.send(Err(DhtError::PutFailed(e.to_string())));
                }
                (PendingQuery::GetRecord { sender, .. }, QueryResult::GetRecord(Ok(record))) => {
                    if let Some(record) = record.records.into_iter().next() {
                        let _ = sender.send(Ok(record.record.value));
                    } else {
                        let _ = sender.send(Err(DhtError::RecordNotFound));
                    }
                }
                (PendingQuery::GetRecord { sender, .. }, QueryResult::GetRecord(Err(e))) => {
                    let _ = sender.send(Err(DhtError::GetFailed(e.to_string())));
                }
                _ => {}
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum DhtEvent {
    PeerAdded(PeerId),
    RecordStored(String),
    RecordRetrieved(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DhtError {
    #[error("Bootstrap failed: {0}")]
    BootstrapFailed(String),
    
    #[error("Put operation failed: {0}")]
    PutFailed(String),
    
    #[error("Get operation failed: {0}")]
    GetFailed(String),
    
    #[error("Record not found")]
    RecordNotFound,
    
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

**Enhanced Swarm Management**

```rust
// In src/network/swarm_manager.rs
use libp2p::{
    swarm::{Swarm, SwarmEvent},
    identify, mdns, noise, quic, tcp, yamux, Multiaddr, PeerId,
};

pub struct SwarmManager {
    swarm: Swarm<NetabaseBehavior>,
    command_receiver: mpsc::UnboundedReceiver<SwarmCommand>,
    event_sender: broadcast::Sender<NetworkEvent>,
}

#[derive(NetworkBehaviour)]
pub struct NetabaseBehavior {
    dht: DhtBehavior,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug, Clone)]
pub enum SwarmCommand {
    StartListening { address: Multiaddr },
    Dial { peer_id: PeerId, address: Multiaddr },
    PutRecord { key: String, value: Vec<u8>, response: oneshot::Sender<Result<(), DhtError>> },
    GetRecord { key: String, response: oneshot::Sender<Result<Vec<u8>, DhtError>> },
    Bootstrap,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    ListeningOn { address: Multiaddr },
    ConnectionEstablished { peer_id: PeerId },
    ConnectionClosed { peer_id: PeerId },
    RecordStored { key: String },
    RecordRetrieved { key: String },
    BootstrapCompleted,
    Error { message: String },
}

impl SwarmManager {
    pub async fn new(
        keypair: identity::Keypair,
    ) -> Result<(Self, mpsc::UnboundedSender<SwarmCommand>, broadcast::Receiver<NetworkEvent>), NetabaseError> {
        let local_peer_id = PeerId::from(keypair.public());
        
        // Build transport
        let transport = {
            let tcp = tcp::tokio::Transport::default()
                .upgrade(upgrade::Version::V1Lazy)
                .authenticate(noise::Config::new(&keypair)?)
                .multiplex(yamux::Config::default())
                .boxed();
                
            let quic = quic::tokio::Transport::new(quic::Config::new(&keypair));
            
            tcp.or_transport(quic).boxed()
        };
        
        // Build behavior
        let behavior = NetabaseBehavior {
            dht: DhtBehavior::new(local_peer_id),
            identify: identify::Behaviour::new(identify::Config::new(
                "/netabase/1.0.0".into(),
                keypair.public(),
            )),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
        };
        
        let mut swarm = Swarm::with_tokio_executor(transport, behavior, local_peer_id);
        swarm.behaviour_mut().dht.bootstrap()?;
        
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = broadcast::channel(100);
        
        let manager = Self {
            swarm,
            command_receiver,
            event_sender,
        };
        
        Ok((manager, command_sender, event_receiver))
    }
    
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    if let Some(network_event) = self.handle_swarm_event(event).await {
                        let _ = self.event_sender.send(network_event);
                    }
                }
                
                // Handle commands
                Some(command) = self.command_receiver.recv() => {
                    if self.handle_command(command).await.is_err() {
                        break;
                    }
                }
                
                // Handle shutdown
                else => break,
            }
        }
    }
    
    async fn handle_swarm_event(&mut self, event: SwarmEvent<NetabaseBehaviorEvent>) -> Option<NetworkEvent> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Listening on {}", address);
                Some(NetworkEvent::ListeningOn { address })
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                log::info!("Connected to {}", peer_id);
                Some(NetworkEvent::ConnectionEstablished { peer_id })
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                log::info!("Disconnected from {}", peer_id);
                Some(NetworkEvent::ConnectionClosed { peer_id })
            }
            SwarmEvent::Behaviour(NetabaseBehaviorEvent::Dht(dht_event)) => {
                self.handle_dht_event(dht_event).await
            }
            SwarmEvent::Behaviour(NetabaseBehaviorEvent::Mdns(mdns_event)) => {
                self.handle_mdns_event(mdns_event).await
            }
            _ => None,
        }
    }
    
    async fn handle_command(&mut self, command: SwarmCommand) -> Result<(), ()> {
        match command {
            SwarmCommand::StartListening { address } => {
                if let Err(e) = self.swarm.listen_on(address) {
                    log::error!("Failed to start listening: {}", e);
                    let _ = self.event_sender.send(NetworkEvent::Error { 
                        message: e.to_string() 
                    });
                }
            }
            SwarmCommand::Dial { peer_id, address } => {
                self.swarm.behaviour_mut().dht.add_address(&peer_id, address.clone());
                if let Err(e) = self.swarm.dial(address) {
                    log::error!("Failed to dial: {}", e);
                }
            }
            SwarmCommand::PutRecord { key, value, response } => {
                let receiver = self.swarm.behaviour_mut().dht.put_record(key, value);
                tokio::spawn(async move {
                    let result = receiver.await.unwrap_or(Err(DhtError::NetworkError("Channel closed".to_string())));
                    let _ = response.send(result);
                });
            }
            SwarmCommand::GetRecord { key, response } => {
                let receiver = self.swarm.behaviour_mut().dht.get_record(key);
                tokio::spawn(async move {
                    let result = receiver.await.unwrap_or(Err(DhtError::NetworkError("Channel closed".to_string())));
                    let _ = response.send(result);
                });
            }
            SwarmCommand::Bootstrap => {
                if let Err(e) = self.swarm.behaviour_mut().dht.bootstrap() {
                    log::error!("Bootstrap failed: {}", e);
                }
            }
            SwarmCommand::Shutdown => {
                return Err(());
            }
        }
        Ok(())
    }
    
    async fn handle_dht_event(&mut self, event: DhtEvent) -> Option<NetworkEvent> {
        match event {
            DhtEvent::RecordStored(key) => Some(NetworkEvent::RecordStored { key }),
            DhtEvent::RecordRetrieved(key) => Some(NetworkEvent::RecordRetrieved { key }),
            DhtEvent::PeerAdded(_) => None,
        }
    }
    
    async fn handle_mdns_event(&mut self, event: mdns::Event) -> Option<NetworkEvent> {
        match event {
            mdns::Event::Discovered(list) => {
                for (peer_id, multiaddr) in list {
                    log::info!("Discovered peer: {} at {}", peer_id, multiaddr);
                    self.swarm.behaviour_mut().dht.add_address(&peer_id, multiaddr);
                }
                None
            }
            mdns::Event::Expired(list) => {
                for (peer_id, _) in list {
                    log::info!("Peer expired: {}", peer_id);
                }
                None
            }
        }
    }
}
```

### 3. Implement Query System

**Current Problem**: No query functionality exists beyond basic key-value operations.

#### Implementation Strategy

**Local Query Engine with Sled Integration**

```rust
// In src/database/query_engine.rs
use sled::{Db, Tree};
use serde::{Deserialize, Serialize};

pub struct QueryEngine {
    db: Db,
    indices: HashMap<String, Tree>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub collection: String,
    pub filter: Option<Filter>,
    pub sort: Option<SortCriteria>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    Equals { field: String, value: serde_json::Value },
    Contains { field: String, value: String },
    Range { field: String, min: Option<serde_json::Value>, max: Option<serde_json::Value> },
    And(Vec<Filter>),
    Or(Vec<Filter>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCriteria {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl QueryEngine {
    pub fn new(db_path: &Path) -> Result<Self, QueryError> {
        let db = sled::open(db_path)?;
        let indices = HashMap::new();
        
        Ok(Self { db, indices })
    }
    
    pub async fn execute_query<T>(&self, query: Query) -> Result<Vec<T>, QueryError>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let tree = self.db.open_tree(&query.collection)?;
        
        // Apply filters
        let filtered_keys = if let Some(filter) = &query.filter {
            self.apply_filter(&tree, filter).await?
        } else {
            tree.iter().keys().collect::<Result<Vec<_>, _>>()?
        };
        
        // Retrieve and deserialize records
        let mut records = Vec::new();
        for key in filtered_keys {
            if let Some(value_bytes) = tree.get(&key)? {
                let record: T = bincode::deserialize(&value_bytes)
                    .map_err(|e| QueryError::DeserializationFailed(e.to_string()))?;
                records.push(record);
            }
        }
        
        // Apply sorting
        if let Some(sort_criteria) = &query.sort {
            // This would need to be implemented based on the actual field types
            // For now, we'll sort by key
            records.sort_by(|a, b| {
                // Placeholder sorting logic
                std::cmp::Ordering::Equal
            });
        }
        
        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = if let Some(limit) = query.limit {
            start + limit
        } else {
            records.len()
        };
        
        Ok(records.into_iter().skip(start).take(end - start).collect())
    }
    
    async fn apply_filter(&self, tree: &Tree, filter: &Filter) -> Result<Vec<sled::IVec>, QueryError> {
        match filter {
            Filter::Equals { field, value } => {
                // This would need an index on the field
                self.query_by_index(tree, field, value).await
            }
            Filter::Contains { field, value } => {
                self.query_contains(tree, field, value).await
            }
            Filter::Range { field, min, max } => {
                self.query_range(tree, field, min.as_ref(), max.as_ref()).await
            }
            Filter::And(filters) => {
                let mut results = None;
                for filter in filters {
                    let filter_results = self.apply_filter(tree, filter).await?;
                    results = Some(match results {
                        None => filter_results,
                        Some(existing) => {
                            // Intersection
                            existing.into_iter()
                                .filter(|key| filter_results.contains(key))
                                .collect()
                        }
                    });
                }
                Ok(results.unwrap_or_default())
            }
            Filter::Or(filters) => {
                let mut all_results = Vec::new();
                for filter in filters {
                    let filter_results = self.apply_filter(tree, filter).await?;
                    all_results.extend(filter_results);
                }
                all_results.sort();
                all_results.dedup();
                Ok(all_results)
            }
        }
    }
    
    async fn query_by_index(
        &self, 
        tree: &Tree, 
        field: &str, 
        value: &serde_json::Value
    ) -> Result<Vec<sled::IVec>, QueryError> {
        // Implementation would use field-specific indices
        // For now, scan all records (inefficient)
        let mut matching_keys = Vec::new();
        
        for item in tree.iter() {
            let (key, value_bytes) = item?;
            // Parse and check field value
            // This is a simplified implementation
            matching_keys.push(key);
        }
        
        Ok(matching_keys)
    }
    
    async fn query_contains(
        &self,
        tree: &Tree,
        field: &str,
        search_value: &str,
    ) -> Result<Vec<sled::IVec>, QueryError> {
        // Placeholder implementation
        Ok(Vec::new())
    }
    
    async fn query_range(
        &self,
        tree: &Tree,
        field: &str,
        min: Option<&serde_json::Value>,
        max: Option<&serde_json::Value>,
    ) -> Result<Vec<sled::IVec>, QueryError> {
        // Placeholder implementation
        Ok(Vec::new())
    }
    
    pub fn create_index(&mut self, collection: &str, field: &str) -> Result<(), QueryError> {
        let index_name = format!("{}_{}_index", collection, field);
        let index_tree = self.db.open_tree(&index_name)?;
        self.indices.insert(index_name, index_tree);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sled::Error),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Index not found: {0}")]
    IndexNotFound(String),
}
```

**Distributed Query Processing**

```rust
// In src/database/distributed_query.rs
use libp2p::gossipsub::{Gossipsub, GossipsubEvent, MessageId, TopicHash};

pub struct DistributedQueryEngine {
    gossipsub: Gossipsub,
    local_engine: QueryEngine,
    pending_queries: HashMap<QueryId, PendingDistributedQuery>,
}

#[derive(Debug)]
struct PendingDistributedQuery {
    query: DistributedQuery,
    results: Vec<QueryResult>,
    response_count: usize,
    expected_responses: usize,
    sender: oneshot::Sender<Vec<QueryResult>>,
    timeout: Pin<Box<tokio::time::Sleep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedQuery {
    pub id: QueryId,
    pub query: Query,
    pub requester: PeerId,
    pub max_results: Option<usize>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub data: Vec<u8>, // Serialized result
    pub peer: PeerId,
    pub confidence: f32,
}

impl DistributedQueryEngine {
    pub fn new(local_engine: QueryEngine) -> Self {
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");
        
        let gossipsub = Gossipsub::new(
            gossipsub::MessageAuthenticity::Signed(keypair),
            gossipsub_config,
        ).expect("Correct configuration");
        
        Self {
            gossipsub,
            local_engine,
            pending_queries: HashMap::new(),
        }
    }
    
    pub async fn execute_distributed_query<T>(
        &mut self,
        query: Query,
        timeout: Duration,
    ) -> Result<Vec<T>, QueryError>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let query_id = QueryId::new();
        let distributed_query = DistributedQuery {
            id: query_id,
            query,
            requester: self.local_peer_id,
            max_results: Some(100),
            timeout,
        };
        
        // Execute locally first
        let local_results = self.local_engine
            .execute_query(distributed_query.query.clone())
            .await?;
        
        // Broadcast query to network
        let query_topic = TopicHash::from_raw("netabase/queries");
        let query_message = bincode::serialize(&distributed_query)?;
        
        self.gossipsub.publish(query_topic, query_message)?;
        
        // Set up result collection
        let (sender, receiver) = oneshot::channel();
        let pending_query = PendingDistributedQuery {
            query: distributed_query,
            results: Vec::new(),
            response_count: 0,
            expected_responses: 5, // Could be dynamic based on network size
            sender,
            timeout: Box::pin(tokio::time::sleep(timeout)),
        };
        
        self.pending_queries.insert(query_id, pending_query);
        
        // Await results
        let distributed_results = receiver.await.unwrap_or_default();
        
        // Combine local and distributed results
        let mut all_results = local_results;
        for result in distributed_results {
            if let Ok(deserialized) = bincode::deserialize(&result.data) {
                all_results.push(deserialized);
            }
        }
        
        // Deduplicate and rank results
        // Implementation would depend on specific data types
        
        Ok(all_results)
    }
    
    pub fn handle_query_request(&mut self, query: DistributedQuery) {
        // Execute query locally and send response
        let local_engine = self.local_engine.clone();
        let gossipsub = self.gossipsub.clone();
        
        tokio::spawn(async move {
            if let Ok(results) = local_engine.execute_query(query.query).await {
                let response = QueryResponse {
                    query_id: query.id,
                    results: results.into_iter()
                        .filter_map(|r| bincode::serialize(&r).ok())
                        .collect(),
                    peer: local_peer_id,
                };
                
                let response_topic = TopicHash::from_raw("netabase/query_responses");
                let response_message = bincode::serialize(&response).unwrap();
                
                let _ = gossipsub.publish(response_topic, response_message);
            }
        });
    }
}

type QueryId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub query_id: QueryId,
    pub results: Vec<Vec<u8>>,
    pub peer: PeerId,
}
```

### 4. Configuration System Enhancement

**Implementation Strategy**:

```rust
// In src/config/mod.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;
use derive_builder::Builder;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NetabaseConfig {
    #[builder(default = "KademliaConfig::default()")]
    pub kademlia: KademliaConfig,
    
    #[builder(default = "SwarmConfig::default()")]
    pub swarm: SwarmConfig,
    
    #[builder(default = "StorageConfig::default()")]
    pub storage: StorageConfig,
    
    #[builder(default = "SecurityConfig::default()")]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct KademliaConfig {
    #[builder(default = "20")]
    pub replication_factor: usize,
    
    #[builder(default = "Duration::from_secs(300)")]
    pub publication_interval: Duration,
    
    #[builder(default = "Duration::from_secs(60)")]
    pub query_timeout: Duration,
    
    #[builder(default = "Duration::from_secs(86400)")]
    pub record_ttl: Duration,
    
    #[builder(default = "1000")]
    pub max_records: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct SwarmConfig {
    #[builder(default = "vec![\"/ip4/0.0.0.0/tcp/0\".parse().unwrap()]")]
    pub listen_addresses: Vec<Multiaddr>,
    
    #[builder(default = "true")]
    pub enable_mdns: bool,
    
    #[builder(default = "100")]
    pub connection_limits: usize,
    
    #[builder(default = "Vec::new()")]
    pub bootstrap_peers: Vec<(PeerId, Multiaddr)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct StorageConfig {
    #[builder(default = "StorageBackend::Sled")]
    pub backend: StorageBackend,
    
    #[builder(default = "\"./netabase_data\".to_string()")]
    pub data_dir: String,
    
    #[builder(default = "1000")]
    pub cache_size: usize,
    
    #[builder(default = "true")]
    pub enable_persistence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    Sled,
    Memory,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct SecurityConfig {
    #[builder(default = "true")]
    pub enable_encryption: bool,
    
    #[builder(default = "1000")]
    pub rate_limit: u32,
    
    #[builder(default = "Duration::from_secs(300)")]
    pub ban_duration: Duration,
    
    #[builder(default = "Vec::new()")]
    pub allowed_peers: Vec<PeerId>,
    
    #[builder(default = "Vec::new()")]
    pub blocked_peers: Vec<PeerId>,
}

impl Default for NetabaseConfig {
    fn default() -> Self {
        Self {
            kademlia: KademliaConfig::default(),
            swarm: SwarmConfig::default(),
            storage: StorageConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl NetabaseConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = envy::from_env::<Self>()?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.kademlia.replication_factor == 0 {
            return Err(ConfigError::InvalidValue("replication_factor must be > 0".to_string()));
        }
        
        if self.swarm.connection_limits == 0 {
            return Err(ConfigError::InvalidValue("connection_limits must be > 0".to_string()));
        }
        
        // Additional validation rules...
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
    
    #[error("Environment variable error: {0}")]
    EnvError(#[from] envy::Error),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}
```

This implementation guide provides concrete technical approaches for the major TODO items in the netabase crate. Focus on completing the macro system first as it's fundamental to the user experience, then implement the core DHT functionality to enable actual distributed storage operations.