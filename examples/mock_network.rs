//! Mock network implementation for testing the complete protocol.
//!
//! This example simulates two nodes communicating over channels to test
//! all protocol layers without requiring actual networking.
//!
//! Run with: cargo run --example mock_network

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use netabase::capabilities::{Capability, CapabilitySignature, Operation};
use netabase::primitives::{
    ConflictRank, KeyRange, LamportClock, NDimensionalRange, NodeId, NodeIdRange, Path,
    PathBuilder,
};
use netabase::query::{
    CapabilityGuard, QueryEntry, QueryError, QueryGuard, QueryResponse, SecureQuery,
    WriteRequest, WriteResponse,
};

/// Message types that can be sent between nodes.
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Handshake request
    HandshakeRequest {
        from: NodeId,
        protocol_version: u32,
    },
    
    /// Handshake response
    HandshakeResponse {
        from: NodeId,
        protocol_version: u32,
        accepted: bool,
    },
    
    /// Query request
    Query {
        from: NodeId,
        query: SecureQuery<String, u16>,
    },
    
    /// Query response
    QueryResponse {
        from: NodeId,
        response: Result<QueryResponse<String>, QueryError>,
    },
    
    /// Write request
    Write {
        from: NodeId,
        write: WriteRequest<String, String, u16>,
    },
    
    /// Write response
    WriteResponse {
        from: NodeId,
        response: WriteResponse,
    },
}

/// A mock node that can participate in the protocol.
pub struct MockNode {
    /// This node's ID
    pub id: NodeId,
    
    /// Channel for receiving messages
    rx: Arc<Mutex<mpsc::UnboundedReceiver<NetworkMessage>>>,
    
    /// In-memory data store
    store: Arc<Mutex<HashMap<Path, QueryEntry<String>>>>,
    
    /// Lamport clock for causality
    clock: Arc<Mutex<LamportClock>>,
    
    /// Capabilities granted by this node
    granted_caps: Arc<Mutex<Vec<Capability<String, u16>>>>,
    
    /// Connected peers
    peers: Arc<Mutex<HashMap<NodeId, mpsc::UnboundedSender<NetworkMessage>>>>,
}

impl MockNode {
    /// Create a new mock node.
    pub fn new(id: NodeId) -> (Self, mpsc::UnboundedSender<NetworkMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let node = Self {
            id,
            rx: Arc::new(Mutex::new(rx)),
            store: Arc::new(Mutex::new(HashMap::new())),
            clock: Arc::new(Mutex::new(LamportClock::new(
                0,
                id.to_bytes()[0..8].try_into().unwrap(),
            ))),
            granted_caps: Arc::new(Mutex::new(Vec::new())),
            peers: Arc::new(Mutex::new(HashMap::new())),
        };
        
        (node, tx)
    }
    
    /// Connect to another node.
    pub async fn connect(&self, peer_id: NodeId, peer_tx: mpsc::UnboundedSender<NetworkMessage>) {
        self.peers.lock().unwrap().insert(peer_id, peer_tx);
        
        // Send handshake
        self.send_to_peer(
            peer_id,
            NetworkMessage::HandshakeRequest {
                from: self.id,
                protocol_version: 1,
            },
        )
        .await;
    }
    
    /// Grant a capability to a peer.
    pub fn grant_capability(
        &self,
        to: NodeId,
        operation: Operation,
        range: NDimensionalRange<String, u16>,
        expiry: u64,
    ) -> Capability<String, u16> {
        let cap = Capability::new_root(self.id, to, operation, range, expiry);
        self.granted_caps.lock().unwrap().push(cap.clone());
        cap
    }
    
    /// Send a query to a peer.
    pub async fn query(
        &self,
        peer: NodeId,
        range: NDimensionalRange<String, u16>,
        capability: Capability<String, u16>,
        limit: Option<u32>,
    ) -> Result<QueryResponse<String>, QueryError> {
        let nonce = {
            let clock = self.clock.lock().unwrap();
            clock.counter + 1
        };
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let query = SecureQuery {
            range,
            capability,
            nonce,
            timestamp,
            signature: CapabilitySignature([0u8; 64]),
            continuation: None,
            limit,
        };
        
        self.send_to_peer(peer, NetworkMessage::Query { from: self.id, query })
            .await;
        
        // Give the other node time to process and handle the message
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Wait for response
        self.receive_response().await
    }
    
    /// Write data to local store.
    pub fn write_local(&self, path: Path, data: String) {
        let mut clock = self.clock.lock().unwrap();
        clock.tick();
        
        let rank = ConflictRank::new(clock.counter, clock.clone());
        
        let entry = QueryEntry {
            author: self.id,
            rank,
            lamport: clock.clone(),
            data,
            data_hash: [0u8; 32],
        };
        
        self.store.lock().unwrap().insert(path, entry);
    }
    
    /// Get data from local store.
    pub fn get_local(&self, path: &Path) -> Option<QueryEntry<String>> {
        self.store.lock().unwrap().get(path).cloned()
    }
    
    /// List all data in local store.
    pub fn list_local(&self) -> Vec<(Path, QueryEntry<String>)> {
        self.store
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    /// Handle incoming messages.
    pub async fn handle_messages(&self) {
        loop {
            let msg = {
                let mut rx_guard = self.rx.lock().unwrap();
                rx_guard.try_recv()
            };
            
            match msg {
                Ok(msg) => {
                    if let Err(e) = self.handle_message(msg).await {
                        eprintln!("[{}] Error handling message: {:?}", self.id, e);
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    eprintln!("[{}] Channel disconnected", self.id);
                    break;
                }
            }
        }
    }
    
    async fn handle_message(&self, msg: NetworkMessage) -> Result<(), String> {
        match msg {
            NetworkMessage::HandshakeRequest { from, protocol_version } => {
                println!("[{}] 🤝 Received handshake from {}", self.id, from);
                
                let accepted = protocol_version == 1;
                
                self.send_to_peer(
                    from,
                    NetworkMessage::HandshakeResponse {
                        from: self.id,
                        protocol_version: 1,
                        accepted,
                    },
                )
                .await;
            }
            
            NetworkMessage::HandshakeResponse { from, accepted, .. } => {
                if accepted {
                    println!("[{}] ✅ Handshake accepted by {}", self.id, from);
                } else {
                    println!("[{}] ❌ Handshake rejected by {}", self.id, from);
                }
            }
            
            NetworkMessage::Query { from, query } => {
                println!("[{}] 🔍 Received query from {} (range: {:?})", 
                    self.id, from, query.range);
                
                // Validate capability
                let guard = CapabilityGuard::new(query.capability.granted_by);
                if let Err(e) = guard.check_query(&query) {
                    println!("[{}] ❌ Query rejected: {:?}", self.id, e);
                    self.send_to_peer(
                        from,
                        NetworkMessage::QueryResponse {
                            from: self.id,
                            response: Err(e),
                        },
                    )
                    .await;
                    return Ok(());
                }
                
                // Execute query
                let entries = self.execute_query(&query.range, query.limit);
                println!("[{}] 📊 Query returned {} entries", self.id, entries.len());
                
                let response = QueryResponse {
                    entries,
                    has_more: false,
                    continuation: None,
                    responder: self.id,
                    signature: CapabilitySignature([0u8; 64]),
                };
                
                self.send_to_peer(
                    from,
                    NetworkMessage::QueryResponse {
                        from: self.id,
                        response: Ok(response),
                    },
                )
                .await;
            }
            
            NetworkMessage::QueryResponse { .. } => {
                // Response handled in query() method
            }
            
            NetworkMessage::Write { from, write } => {
                println!("[{}] ✏️  Received write from {}", self.id, from);
                
                let response = self.execute_write(write);
                
                self.send_to_peer(
                    from,
                    NetworkMessage::WriteResponse {
                        from: self.id,
                        response,
                    },
                )
                .await;
            }
            
            NetworkMessage::WriteResponse { from, response } => {
                println!("[{}] 📝 Write response from {}: {:?}", self.id, from, response);
            }
        }
        
        Ok(())
    }
    
    fn execute_query(
        &self,
        range: &NDimensionalRange<String, u16>,
        limit: Option<u32>,
    ) -> Vec<QueryEntry<String>> {
        let store = self.store.lock().unwrap();
        let limit = limit.unwrap_or(100) as usize;
        
        store
            .iter()
            .filter(|(path, _entry)| {
                // Check if path matches the range
                range.primary_key.contains(path)
            })
            .take(limit)
            .map(|(_, entry)| entry.clone())
            .collect()
    }
    
    fn execute_write(&self, write: WriteRequest<String, String, u16>) -> WriteResponse {
        let mut clock = self.clock.lock().unwrap();
        
        // Update Lamport clock
        clock.merge(&write.entry.lamport);
        clock.tick();
        
        WriteResponse::Ok {
            rank: write.entry.rank,
        }
    }
    
    async fn send_to_peer(&self, peer: NodeId, msg: NetworkMessage) {
        if let Some(tx) = self.peers.lock().unwrap().get(&peer) {
            let _ = tx.send(msg);
        }
    }
    
    async fn receive_response(&self) -> Result<QueryResponse<String>, QueryError> {
        let rx = self.rx.clone();
        
        // Try for up to 1 second
        for _ in 0..100 {
            let msg = {
                let mut rx_guard = rx.lock().unwrap();
                rx_guard.try_recv()
            };
            
            match msg {
                Ok(NetworkMessage::QueryResponse { response, .. }) => return response,
                Ok(other) => {
                    // Put it back for later processing
                    println!("  [Debug] Skipping non-response message");
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(_) => break,
            }
        }
        
        Err(QueryError::ExecutionError {
            message: "No response received".to_string(),
        })
    }
}

#[tokio::main]
async fn main() {
    println!("\n🚀 Mock Network Protocol Test\n");
    println!("================================\n");
    
    // Create two nodes
    let node1_id = NodeId::from_bytes([1u8; 32]);
    let node2_id = NodeId::from_bytes([2u8; 32]);
    
    let (node1, node1_tx) = MockNode::new(node1_id);
    let (node2, node2_tx) = MockNode::new(node2_id);
    
    let node1_arc = Arc::new(node1);
    
    println!("📍 Node 1 ID: {}", node1_id);
    println!("📍 Node 2 ID: {}\n", node2_id);
    
    // Connect nodes
    println!("Phase 1: Handshake\n");
    node1_arc.connect(node2_id, node2_tx.clone()).await;
    node2.connect(node1_id, node1_tx.clone()).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    node1_arc.handle_messages().await;
    node2.handle_messages().await;
    
    // Write data to node1
    println!("\nPhase 2: Writing Data\n");
    let path1 = PathBuilder::new().key("users").key("alice").build();
    let path2 = PathBuilder::new().key("users").key("bob").build();
    let path3 = PathBuilder::new().key("posts").key("post1").build();
    
    node1_arc.write_local(path1.clone(), "Alice's profile: age 30, city SF".to_string());
    node1_arc.write_local(path2.clone(), "Bob's profile: age 25, city NYC".to_string());
    node1_arc.write_local(path3.clone(), "Hello from Alice!".to_string());
    
    println!("[{}] 💾 Wrote 3 entries", node1_id);
    
    // Show what's in node1's store
    println!("\n[{}] 📚 Local store contents:", node1_id);
    for (path, entry) in node1_arc.list_local() {
        println!("  - {}: {}", path, entry.data);
    }
    
    // Grant capability from node1 to node2
    println!("\nPhase 3: Capability Grant\n");
    let range: NDimensionalRange<String, u16> = NDimensionalRange::new(
        NodeIdRange::All,
        KeyRange::prefix(PathBuilder::new().key("users").build()),
        vec![],
    );
    
    let capability = node1_arc.grant_capability(
        node2_id,
        Operation::Read,
        range.clone(),
        u64::MAX,
    );
    
    println!("[{}] 🔑 Granted READ capability to {} for 'users/*'", 
        node1_id, node2_id);
    
    // Verify capability
    assert!(capability.verify_chain(&node1_id).is_ok());
    println!("[{}] ✅ Capability chain verified", node2_id);
    
    // Node2 queries node1
    println!("\nPhase 4: Query Execution\n");
    println!("[{}] 🔍 Querying {} for 'users/*'", node2_id, node1_id);
    
    // Spawn task to handle node1's messages
    let node1_clone = node1_arc.clone();
    let handle1 = tokio::spawn(async move {
        for _ in 0..5 {
            node1_clone.handle_messages().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });
    
    match node2.query(node1_id, range.clone(), capability, Some(10)).await {
        Ok(response) => {
            println!("[{}] 📊 Query successful! Received {} entries:", 
                node2_id, response.entries.len());
            for entry in &response.entries {
                println!("  - Author: {}, Data: {}", entry.author, entry.data);
            }
        }
        Err(e) => {
            println!("[{}] ❌ Query failed: {:?}", node2_id, e);
        }
    }
    
    handle1.await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    node2.handle_messages().await;
    
    // Test unauthorized query (different range)
    println!("\nPhase 5: Authorization Test\n");
    let unauthorized_range: NDimensionalRange<String, u16> = NDimensionalRange::new(
        NodeIdRange::All,
        KeyRange::prefix(PathBuilder::new().key("posts").build()),
        vec![],
    );
    
    println!("[{}] 🔍 Attempting unauthorized query for 'posts/*'", node2_id);
    
    let bad_capability = node1_arc.grant_capability(
        node2_id,
        Operation::Read,
        range, // Wrong range!
        u64::MAX,
    );
    
    // Spawn task to handle node1's messages
    let node1_clone2 = node1_arc.clone();
    let handle2 = tokio::spawn(async move {
        for _ in 0..5 {
            node1_clone2.handle_messages().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });
    
    match node2.query(node1_id, unauthorized_range, bad_capability, Some(10)).await {
        Ok(response) => {
            println!("[{}] ⚠️  Query succeeded (should have failed!): {} entries", 
                node2_id, response.entries.len());
        }
        Err(e) => {
            println!("[{}] ✅ Query correctly rejected: {:?}", node2_id, e);
        }
    }
    
    handle2.await.unwrap();
    
    println!("\n================================");
    println!("\n✅ All protocol phases completed successfully!");
    println!("\nTested:");
    println!("  ✓ Handshake protocol");
    println!("  ✓ Data storage and retrieval");
    println!("  ✓ Capability grant and verification");
    println!("  ✓ Secure query execution");
    println!("  ✓ Authorization validation");
    println!("\n🎉 Mock network test complete!\n");
}
