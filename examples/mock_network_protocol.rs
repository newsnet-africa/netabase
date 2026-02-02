//! Full protocol mock network implementation using actual protocol state machines.
//!
//! This example demonstrates the complete Netabase protocol flow:
//! 1. Handshake establishment
//! 2. Capability exchange
//! 3. Query execution
//! 4. Data synchronization
//!
//! Uses channels to simulate network transport between two nodes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use serde::{Deserialize, Serialize};

use netabase::capabilities::{Capability, CapabilitySignature, Operation};
use netabase::network::protocol::{
    features, ProtocolMessage, HandshakeRequest, HandshakeResponse,
    SyncRequest, SyncResponse, DisconnectMessage, DisconnectReason,
};
use netabase::primitives::{
    ConflictRank, KeyRange, LamportClock, NDimensionalRange, NodeId, NodeIdRange, PathBuilder,
};
use netabase::protocol::{
    HandshakeStateMachine, QueryHandler, SyncHandler, SessionManager, PeerSession,
};
use netabase::query::messages::{QueryEntry, QueryError, QueryResponse, SecureQuery};

// =========================================================================
//  Mock Network Layer
// =========================================================================

/// Network envelope wrapping protocol messages with routing info.
#[derive(Debug, Clone)]
struct NetworkEnvelope<PK, SK, T> {
    from: NodeId,
    to: NodeId,
    message: ProtocolMessage<PK, SK, T>,
}

/// Mock transport using channels.
struct MockTransport<PK, SK, T> {
    node_id: NodeId,
    tx: mpsc::UnboundedSender<NetworkEnvelope<PK, SK, T>>,
    rx: Arc<AsyncMutex<mpsc::UnboundedReceiver<NetworkEnvelope<PK, SK, T>>>>,
}

impl<PK, SK, T> MockTransport<PK, SK, T> {
    fn new(node_id: NodeId) -> (Self, mpsc::UnboundedSender<NetworkEnvelope<PK, SK, T>>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let external_tx = tx.clone();
        
        (
            Self {
                node_id,
                tx,
                rx: Arc::new(AsyncMutex::new(rx)),
            },
            external_tx,
        )
    }
    
    fn send(&self, to: NodeId, message: ProtocolMessage<PK, SK, T>) {
        let envelope = NetworkEnvelope {
            from: self.node_id,
            to,
            message,
        };
        
        self.tx.send(envelope).ok();
    }
    
    async fn recv(&self) -> Option<NetworkEnvelope<PK, SK, T>> {
        self.rx.lock().await.recv().await
    }
}

// =========================================================================
//  Mock Node with Protocol State Machines
// =========================================================================

/// A complete node with all protocol state machines.
struct MockNode<PK, SK, T>
where
    PK: Clone + PartialEq + Send + Sync + 'static,
    SK: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    /// Node identity
    id: NodeId,
    
    /// Transport layer
    transport: MockTransport<PK, SK, T>,
    
    /// Lamport clock
    clock: Arc<Mutex<LamportClock>>,
    
    /// Session manager
    sessions: SessionManager<PK, SK>,
    
    /// Local data store (simplified)
    data: Arc<Mutex<HashMap<Vec<u8>, QueryEntry<T>>>>,
    
    /// Schema hash
    schema_hash: [u8; 32],
}

impl<PK, SK, T> MockNode<PK, SK, T>
where
    PK: Clone + PartialEq + Send + Sync + 'static,
    SK: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn new(
        id: NodeId,
        transport: MockTransport<PK, SK, T>,
        schema_hash: [u8; 32],
    ) -> Self {
        Self {
            id,
            transport,
            clock: Arc::new(Mutex::new(LamportClock::new(
                0,
                id.to_bytes()[0..8].try_into().unwrap(),
            ))),
            sessions: SessionManager::new(300), // 5 minute timeout
            data: Arc::new(Mutex::new(HashMap::new())),
            schema_hash,
        }
    }
    
    /// Initiate handshake with a peer.
    fn initiate_handshake(&self, peer_id: NodeId) -> HandshakeRequest {
        let mut handshake = HandshakeStateMachine::new(
            self.id,
            1,
            features::SYNC | features::SUBSCRIPTIONS,
            self.schema_hash,
        );
        
        let mut clock = self.clock.lock().unwrap();
        let request = handshake.initiate(&mut clock);
        drop(clock);
        
        // Create session
        let session = PeerSession::new(peer_id, 1, 0);
        self.sessions.upsert(session);
        
        self.transport.send(peer_id, ProtocolMessage::HandshakeRequest(request.clone()));
        
        request
    }
    
    /// Handle incoming handshake request.
    fn handle_handshake_request(&self, request: HandshakeRequest) -> HandshakeResponse {
        let mut handshake = HandshakeStateMachine::new(
            self.id,
            1,
            features::SYNC | features::SUBSCRIPTIONS,
            self.schema_hash,
        );
        
        let response = handshake.handle_request(request.clone());
        
        if response.accepted {
            // Create or update session
            let session = PeerSession::new(
                request.from,
                request.protocol_version,
                request.features,
            );
            self.sessions.upsert(session);
            
            println!(
                "[Node {:?}] Handshake accepted from {:?}",
                &self.id.to_bytes()[0..4],
                &request.from.to_bytes()[0..4]
            );
        } else {
            println!(
                "[Node {:?}] Handshake rejected from {:?}: {:?}",
                &self.id.to_bytes()[0..4],
                &request.from.to_bytes()[0..4],
                response.reason
            );
        }
        
        self.transport.send(
            request.from,
            ProtocolMessage::HandshakeResponse(response.clone()),
        );
        
        response
    }
    
    /// Handle incoming handshake response.
    fn handle_handshake_response(&self, response: HandshakeResponse) {
        if response.accepted {
            println!(
                "[Node {:?}] Handshake complete with {:?}",
                &self.id.to_bytes()[0..4],
                &response.from.to_bytes()[0..4]
            );
            
            self.sessions.update(&response.from, |session| {
                session.touch();
                session.peer_features = 0; // Would be in request
            });
        } else {
            println!(
                "[Node {:?}] Handshake rejected by {:?}: {:?}",
                &self.id.to_bytes()[0..4],
                &response.from.to_bytes()[0..4],
                response.reason
            );
        }
    }
    
    /// Grant capability to a peer.
    fn grant_capability(
        &self,
        peer_id: NodeId,
        operation: Operation,
        range: NDimensionalRange<PK, SK>,
    ) -> Capability<PK, SK> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let cap = Capability::new_root(
            self.id,
            peer_id,
            operation,
            range,
            now + 3600, // 1 hour expiry
        );
        
        self.sessions.update(&peer_id, |session| {
            session.grant_capability(cap.clone());
        });
        
        cap
    }
    
    /// Create a query.
    fn create_query(
        &self,
        capability: Capability<PK, SK>,
        range: NDimensionalRange<PK, SK>,
    ) -> SecureQuery<PK, SK> {
        let mut clock = self.clock.lock().unwrap();
        clock.tick();
        let nonce = clock.counter;
        drop(clock);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // TODO: Actually sign the query
        let signature = CapabilitySignature([0u8; 64]);
        
        SecureQuery {
            range,
            capability,
            nonce,
            timestamp,
            signature,
            continuation: None,
            limit: Some(100),
        }
    }
    
    /// Execute a query locally.
    fn execute_query_local(&self, query: &SecureQuery<PK, SK>) -> Result<QueryResponse<T>, QueryError> {
        // Validate query (simplified - using QueryHandler would be better)
        let data = self.data.lock().unwrap();
        let entries: Vec<QueryEntry<T>> = data.values().cloned().collect();
        drop(data);
        
        // TODO: Actually filter by range
        
        Ok(QueryResponse {
            entries,
            has_more: false,
            continuation: None,
            responder: self.id,
            signature: CapabilitySignature([0u8; 64]),
        })
    }
    
    /// Store data locally.
    fn store_entry(&self, key: Vec<u8>, entry: QueryEntry<T>) {
        self.data.lock().unwrap().insert(key, entry);
    }
    
    /// Get all data.
    fn get_all_data(&self) -> Vec<QueryEntry<T>> {
        self.data.lock().unwrap().values().cloned().collect()
    }
    
    /// Handle incoming message.
    async fn handle_message(&self, envelope: NetworkEnvelope<PK, SK, T>) {
        // Update session last message time
        self.sessions.update(&envelope.from, |session| {
            session.touch();
            session.clock.merge(&self.clock.lock().unwrap());
        });
        
        match envelope.message {
            ProtocolMessage::HandshakeRequest(req) => {
                self.handle_handshake_request(req);
            }
            ProtocolMessage::HandshakeResponse(resp) => {
                self.handle_handshake_response(resp);
            }
            ProtocolMessage::Query(query) => {
                println!(
                    "[Node {:?}] Received query from {:?}",
                    &self.id.to_bytes()[0..4],
                    &envelope.from.to_bytes()[0..4]
                );
                
                let result = self.execute_query_local(&query);
                self.transport.send(
                    envelope.from,
                    ProtocolMessage::QueryResponse(result),
                );
            }
            ProtocolMessage::QueryResponse(result) => {
                match result {
                    Ok(response) => {
                        println!(
                            "[Node {:?}] Received query response: {} entries",
                            &self.id.to_bytes()[0..4],
                            response.entries.len()
                        );
                    }
                    Err(e) => {
                        println!(
                            "[Node {:?}] Query error: {:?}",
                            &self.id.to_bytes()[0..4],
                            e
                        );
                    }
                }
            }
            ProtocolMessage::SyncRequest(req) => {
                println!(
                    "[Node {:?}] Received sync request from {:?}",
                    &self.id.to_bytes()[0..4],
                    &envelope.from.to_bytes()[0..4]
                );
                
                let mut handler = SyncHandler::new(self.id);
                let entries = self.get_all_data();
                let response = handler.handle_sync_request(&req, entries);
                
                self.transport.send(
                    envelope.from,
                    ProtocolMessage::SyncResponse(response),
                );
            }
            ProtocolMessage::SyncResponse(resp) => {
                println!(
                    "[Node {:?}] Received sync response: {} entries",
                    &self.id.to_bytes()[0..4],
                    resp.entries.len()
                );
                
                // Merge synced entries
                for entry in resp.entries {
                    // TODO: Proper key generation
                    let key = entry.data_hash.to_vec();
                    self.store_entry(key, entry);
                }
            }
            ProtocolMessage::Disconnect(msg) => {
                println!(
                    "[Node {:?}] Peer {:?} disconnected: {:?}",
                    &self.id.to_bytes()[0..4],
                    &envelope.from.to_bytes()[0..4],
                    msg.reason
                );
                self.sessions.remove(&envelope.from);
            }
            _ => {
                println!(
                    "[Node {:?}] Unhandled message type",
                    &self.id.to_bytes()[0..4]
                );
            }
        }
    }
    
    /// Run the node's message loop.
    async fn run(&self) {
        loop {
            if let Some(envelope) = self.transport.recv().await {
                self.handle_message(envelope).await;
            } else {
                break;
            }
        }
    }
}

// =========================================================================
//  Test Scenario
// =========================================================================

#[tokio::main]
async fn main() {
    println!("=== Netabase Protocol Mock Network ===\n");
    
    // Create two nodes
    let node1_id = NodeId::from_bytes([1u8; 32]);
    let node2_id = NodeId::from_bytes([2u8; 32]);
    
    let schema_hash = [0u8; 32];
    
    let (transport1, tx1) = MockTransport::new(node1_id);
    let (transport2, tx2) = MockTransport::new(node2_id);
    
    let node1 = Arc::new(MockNode::<String, u16, String>::new(
        node1_id,
        transport1,
        schema_hash,
    ));
    
    let node2 = Arc::new(MockNode::<String, u16, String>::new(
        node2_id,
        transport2,
        schema_hash,
    ));
    
    // Cross-wire the transports
    let node1_clone = Arc::clone(&node1);
    let node2_clone = Arc::clone(&node2);
    
    // Spawn message handlers
    let handle1 = tokio::spawn(async move {
        node1_clone.run().await;
    });
    
    let handle2 = tokio::spawn(async move {
        node2_clone.run().await;
    });
    
    // Give handlers time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // PHASE 1: Handshake
    println!("PHASE 1: Handshake");
    println!("-----------------");
    node1.initiate_handshake(node2_id);
    
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // PHASE 2: Capability Exchange
    println!("\nPHASE 2: Capability Exchange");
    println!("----------------------------");
    
    let range = NDimensionalRange::new(
        NodeIdRange::All,
        KeyRange::prefix(PathBuilder::new().key("users").build()),
        vec![],
    );
    
    let cap = node1.grant_capability(node2_id, Operation::Read, range.clone());
    println!("[Node 1] Granted read capability to Node 2");
    
    // PHASE 3: Insert Data on Node1
    println!("\nPHASE 3: Insert Test Data");
    println!("-------------------------");
    
    node1.store_entry(
        vec![1, 2, 3],
        QueryEntry {
            author: node1_id,
            rank: ConflictRank::basic(1),
            lamport: LamportClock::new(1, node1_id.to_bytes()[0..8].try_into().unwrap()),
            data: "Test data from Node 1".to_string(),
            data_hash: [1u8; 32],
        },
    );
    
    println!("[Node 1] Stored 1 entry");
    
    // PHASE 4: Query
    println!("\nPHASE 4: Query Execution");
    println!("------------------------");
    
    let query = node2.create_query(cap.clone(), range.clone());
    node2.transport.send(node1_id, ProtocolMessage::Query(query));
    
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // PHASE 5: Sync
    println!("\nPHASE 5: Data Synchronization");
    println!("-----------------------------");
    
    let sync_req = SyncRequest {
        range: range.clone(),
        local_fingerprint: netabase::network::protocol::Fingerprint {
            hash: [0u8; 32],
            count: 0,
            max_clock: 0,
        },
        capability: cap,
        nonce: 5,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    node2.transport.send(node1_id, ProtocolMessage::SyncRequest(sync_req));
    
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // PHASE 6: Disconnect
    println!("\nPHASE 6: Clean Disconnect");
    println!("-------------------------");
    
    node1.transport.send(
        node2_id,
        ProtocolMessage::Disconnect(DisconnectMessage {
            from: node1_id,
            reason: DisconnectReason::Shutdown,
        }),
    );
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("\n=== Test Complete ===");
    
    // Note: In a real implementation, we'd properly shut down the channels
    // For this example, we'll just let it exit
}
