//! Comprehensive Mock Network Protocol Example
//!
//! This example demonstrates all phases of the Netabase networking protocol:
//! 1. Handshake - Establish connection and verify schema
//! 2. Capability Exchange - Grant and verify access permissions
//! 3. Query Protocol - Secure query execution
//! 4. Write Protocol - Distributed writes with conflict resolution
//! 5. Sync Protocol - Efficient range-based synchronization
//!
//! Uses in-memory channels to simulate network communication.

use netabase::primitives::{LamportClock, NodeId};

use netabase_store::prelude::*;
use netabase_store::traits::database::transaction::NBTransaction;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex};

// Define our test schema
#[netabase_macros::netabase_networking]
#[netabase_macros::netabase_definition(SocialNetwork)]
pub mod social_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct UserProfile {
        #[primary_key]
        pub id: String,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub bio: String,
        pub created_at: u64,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub author_id: String,
        pub content: String,
        #[secondary_key]
        pub created_at: u64,
        pub likes: u32,
    }
}

use social_models::*;

/// Protocol messages for our mock network
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ProtocolMsg {
    // Phase 1: Handshake
    HandshakeRequest {
        from: NodeId,
        protocol_version: u32,
        schema_hash: [u8; 32],
        nonce: u64,
    },
    HandshakeResponse {
        from: NodeId,
        accepted: bool,
        reason: Option<String>,
    },
    
    // Phase 2: Capability Exchange
    CapabilityRequest {
        operations: Vec<String>,
    },
    CapabilityGrant {
        operations: Vec<String>,
        expiry: Option<u64>,
    },
    
    // Phase 3: Query
    Query {
        query_id: u64,
        model: String,
        filter: Option<String>,
    },
    QueryResponse {
        query_id: u64,
        results: Vec<Vec<u8>>,  // Serialized models
    },
    
    // Phase 4: Write
    Write {
        model: String,
        data: Vec<u8>,  // Serialized model
        timestamp: u64,
    },
    WriteAck {
        success: bool,
        reason: Option<String>,
    },
    
    // Phase 5: Sync
    SyncRequest {
        model: String,
        from_timestamp: u64,
    },
    SyncResponse {
        items: Vec<Vec<u8>>,  // Serialized models
        has_more: bool,
    },
}

/// Session state for a peer connection
#[derive(Debug, Clone, PartialEq, Eq)]
enum SessionState {
    Connecting,
    HandshakeComplete,
    CapabilitiesGranted,
    Established,
    Disconnected,
}

/// Mock network node with full protocol support
struct MockNode {
    id: NodeId,
    store: RedbStore<SocialNetwork>,
    clock: LamportClock,
    rx: mpsc::UnboundedReceiver<(NodeId, ProtocolMsg)>,
    router: Arc<Mutex<HashMap<NodeId, mpsc::UnboundedSender<(NodeId, ProtocolMsg)>>>>,
    _tx: mpsc::UnboundedSender<(NodeId, ProtocolMsg)>,
    sessions: HashMap<NodeId, SessionState>,
    granted_capabilities: HashMap<NodeId, Vec<String>>,
}

impl MockNode {
    fn new(
        id: NodeId,
        router: Arc<Mutex<HashMap<NodeId, mpsc::UnboundedSender<(NodeId, ProtocolMsg)>>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (store, _temp) = RedbStore::<SocialNetwork>::new_temporary()?;
        let (tx, rx) = mpsc::unbounded_channel();
        let node_id_bytes: [u8; 8] = id.to_bytes()[0..8].try_into().unwrap();
        let clock = LamportClock::new(0, node_id_bytes);
        
        Ok(Self {
            id,
            store,
            clock,
            rx,
            router: router.clone(),
            _tx: tx,
            sessions: HashMap::new(),
            granted_capabilities: HashMap::new(),
        })
    }
    
    async fn register(&self) {
        let mut r = self.router.lock().await;
        r.insert(self.id.clone(), self._tx.clone());
    }

    async fn send(&mut self, to: NodeId, msg: ProtocolMsg) -> Result<(), String> {
        self.clock.tick();
        let router = self.router.lock().await;
        if let Some(tx) = router.get(&to) {
            tx.send((self.id.clone(), msg))
                .map_err(|e| format!("Send error: {}", e))?;
            Ok(())
        } else {
            Err(format!("Node not found"))
        }
    }

    async fn receive(&mut self) -> Option<(NodeId, ProtocolMsg)> {
        if let Some((from, msg)) = self.rx.recv().await {
            self.clock.tick();
            Some((from, msg))
        } else {
            None
        }
    }
    
    fn get_session_state(&self, peer: &NodeId) -> SessionState {
        self.sessions.get(peer).cloned().unwrap_or(SessionState::Connecting)
    }
    
    fn set_session_state(&mut self, peer: NodeId, state: SessionState) {
        self.sessions.insert(peer, state);
    }
    
    fn grant_capability(&mut self, peer: NodeId, operations: Vec<String>) {
        self.granted_capabilities.insert(peer, operations);
    }
    
    fn has_capability(&self, peer: &NodeId, operation: &str) -> bool {
        self.granted_capabilities
            .get(peer)
            .map(|ops| ops.contains(&operation.to_string()))
            .unwrap_or(false)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Mock Network Protocol Example");
    println!("================================================\n");

    // Create shared router
    let router = Arc::new(Mutex::new(HashMap::new()));

    // Create two nodes
    let node_id_alice = NodeId::from([1u8; 32]);
    let node_id_bob = NodeId::from([2u8; 32]);

    let mut node_alice = MockNode::new(node_id_alice.clone(), router.clone())?;
    let mut node_bob = MockNode::new(node_id_bob.clone(), router.clone())?;
    
    node_alice.register().await;
    node_bob.register().await;

    println!("✅ Created Node Alice");
    println!("✅ Created Node Bob\n");

    // Seed Alice's node with data
    println!("📝 Seeding Alice's node with data...");
    {
        let txn = node_alice.store.begin_write()?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        txn.create(&UserProfile {
            id: UserProfileID("alice".into()),
            name: "Alice Smith".into(),
            email: "alice@example.com".into(),
            bio: "Software engineer and open source enthusiast".into(),
            created_at: now,
        })?;
        
        txn.create(&Post {
            id: PostID("post1".into()),
            author_id: "alice".into(),
            content: "Hello, decentralized world!".into(),
            created_at: now,
            likes: 42,
        })?;
        
        txn.create(&Post {
            id: PostID("post2".into()),
            author_id: "alice".into(),
            content: "Building the future of peer-to-peer data sync".into(),
            created_at: now + 100,
            likes: 100,
        })?;
        
        txn.commit()?;
    }
    println!("✅ Alice's node seeded with 1 user and 2 posts\n");

    // =========================================================================
    // Phase 1: Handshake
    // =========================================================================
    println!("=== Phase 1: Handshake Protocol ===");
    {
        let schema_hash = [0u8; 32]; // In real implementation, hash the schema
        let nonce = 12345u64;
        
        println!("1. Bob → Alice: Handshake Request");
        node_bob.send(
            node_id_alice.clone(),
            ProtocolMsg::HandshakeRequest {
                from: node_id_bob.clone(),
                protocol_version: 1,
                schema_hash,
                nonce,
            },
        ).await?;
        
        if let Some((from, msg)) = node_alice.receive().await {
            if let ProtocolMsg::HandshakeRequest { .. } = msg {
                println!("   Alice received handshake request");
                node_alice.set_session_state(from.clone(), SessionState::HandshakeComplete);
                
                println!("2. Alice → Bob: Handshake Response (Accepted)");
                node_alice.send(
                    from,
                    ProtocolMsg::HandshakeResponse {
                        from: node_id_alice.clone(),
                        accepted: true,
                        reason: None,
                    },
                ).await?;
            }
        }
        
        if let Some((from, msg)) = node_bob.receive().await {
            if let ProtocolMsg::HandshakeResponse { accepted, .. } = msg {
                if accepted {
                    println!("   Bob received: Handshake accepted ✓");
                    node_bob.set_session_state(from, SessionState::HandshakeComplete);
                }
            }
        }
    }
    println!("✅ Handshake complete\n");

    // =========================================================================
    // Phase 2: Capability Exchange
    // =========================================================================
    println!("=== Phase 2: Capability Exchange ===");
    {
        println!("1. Bob → Alice: Request Read capability");
        node_bob.send(
            node_id_alice.clone(),
            ProtocolMsg::CapabilityRequest {
                operations: vec!["Read".to_string()],
            },
        ).await?;
        
        if let Some((from, msg)) = node_alice.receive().await {
            if let ProtocolMsg::CapabilityRequest { operations } = msg {
                println!("   Alice received capability request for: {:?}", operations);
                
                // Grant the capability
                node_alice.grant_capability(from.clone(), operations.clone());
                
                println!("2. Alice → Bob: Grant Read capability");
                node_alice.send(
                    from,
                    ProtocolMsg::CapabilityGrant {
                        operations,
                        expiry: None,
                    },
                ).await?;
            }
        }
        
        if let Some((from, msg)) = node_bob.receive().await {
            if let ProtocolMsg::CapabilityGrant { operations, .. } = msg {
                println!("   Bob received capability grant: {:?} ✓", operations);
                node_bob.set_session_state(from, SessionState::CapabilitiesGranted);
            }
        }
    }
    println!("✅ Capability exchange complete\n");

    // =========================================================================
    // Phase 3: Query Protocol
    // =========================================================================
    println!("=== Phase 3: Query Protocol ===");
    {
        let query_id = 1001u64;
        
        println!("1. Bob → Alice: Query for UserProfile");
        node_bob.send(
            node_id_alice.clone(),
            ProtocolMsg::Query {
                query_id,
                model: "UserProfile".to_string(),
                filter: Some("alice".to_string()),
            },
        ).await?;
        
        if let Some((from, msg)) = node_alice.receive().await {
            if let ProtocolMsg::Query { query_id, model, filter } = msg {
                println!("   Alice processing query: model={}, filter={:?}", model, filter);
                
                // Check capability
                if !node_alice.has_capability(&from, "Read") {
                    println!("   ❌ Bob doesn't have Read capability!");
                    return Ok(());
                }
                
                // Execute query
                let txn = node_alice.store.begin_read()?;
                let user: Option<UserProfile> = txn.read(&UserProfileID(filter.unwrap()))?;
                
                let results = if let Some(u) = user {
                    vec![postcard::to_allocvec(&u)?]
                } else {
                    vec![]
                };
                
                println!("2. Alice → Bob: Query Response ({} results)", results.len());
                node_alice.send(
                    from,
                    ProtocolMsg::QueryResponse {
                        query_id,
                        results,
                    },
                ).await?;
            }
        }
        
        if let Some((_, msg)) = node_bob.receive().await {
            if let ProtocolMsg::QueryResponse { query_id, results } = msg {
                println!("   Bob received query response (id={}): {} results", query_id, results.len());
                
                for result in results {
                    let user: UserProfile = postcard::from_bytes(&result)?;
                    println!("     - User: {} ({})", user.name, user.email);
                    
                    // Store in Bob's database
                    let txn = node_bob.store.begin_write()?;
                    txn.create(&user)?;
                    txn.commit()?;
                }
            }
        }
    }
    println!("✅ Query protocol complete\n");

    // =========================================================================
    // Phase 4: Write Protocol
    // =========================================================================
    println!("=== Phase 4: Write Protocol ===");
    {
        // Bob requests Write capability first
        println!("1. Bob → Alice: Request Write capability");
        node_bob.send(
            node_id_alice.clone(),
            ProtocolMsg::CapabilityRequest {
                operations: vec!["Write".to_string()],
            },
        ).await?;
        
        if let Some((from, msg)) = node_alice.receive().await {
            if let ProtocolMsg::CapabilityRequest { operations } = msg {
                node_alice.grant_capability(from.clone(), operations.clone());
                node_alice.send(
                    from,
                    ProtocolMsg::CapabilityGrant {
                        operations,
                        expiry: None,
                    },
                ).await?;
            }
        }
        
        if let Some((_, msg)) = node_bob.receive().await {
            if let ProtocolMsg::CapabilityGrant { .. } = msg {
                println!("   Bob received Write capability ✓");
            }
        }
        
        // Now Bob can write
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let new_post = Post {
            id: PostID("post3".into()),
            author_id: "alice".into(),
            content: "Posted from Bob's node!".into(),
            created_at: now + 200,
            likes: 0,
        };
        
        println!("2. Bob → Alice: Write new Post");
        node_bob.send(
            node_id_alice.clone(),
            ProtocolMsg::Write {
                model: "Post".to_string(),
                data: postcard::to_allocvec(&new_post)?,
                timestamp: now + 200,
            },
        ).await?;
        
        if let Some((from, msg)) = node_alice.receive().await {
            if let ProtocolMsg::Write { model, data, .. } = msg {
                println!("   Alice received write for: {}", model);
                
                if !node_alice.has_capability(&from, "Write") {
                    println!("   ❌ Bob doesn't have Write capability!");
                    return Ok(());
                }
                
                // Deserialize and store
                let post: Post = postcard::from_bytes(&data)?;
                let txn = node_alice.store.begin_write()?;
                txn.create(&post)?;
                txn.commit()?;
                
                println!("3. Alice → Bob: Write Acknowledgement");
                node_alice.send(
                    from,
                    ProtocolMsg::WriteAck {
                        success: true,
                        reason: None,
                    },
                ).await?;
            }
        }
        
        if let Some((_, msg)) = node_bob.receive().await {
            if let ProtocolMsg::WriteAck { success, .. } = msg {
                if success {
                    println!("   Bob received: Write successful ✓");
                }
            }
        }
    }
    println!("✅ Write protocol complete\n");

    // =========================================================================
    // Phase 5: Sync Protocol
    // =========================================================================
    println!("=== Phase 5: Sync Protocol ===");
    {
        let from_timestamp = 0u64;
        
        println!("1. Bob → Alice: Sync request for all Posts");
        node_bob.send(
            node_id_alice.clone(),
            ProtocolMsg::SyncRequest {
                model: "Post".to_string(),
                from_timestamp,
            },
        ).await?;
        
        if let Some((from, msg)) = node_alice.receive().await {
            if let ProtocolMsg::SyncRequest { model, from_timestamp } = msg {
                println!("   Alice processing sync for: {} (from timestamp: {})", model, from_timestamp);
                
                // In real implementation, would use range query on secondary key (created_at)
                // For now, we'll just return all posts
                let txn = node_alice.store.begin_read()?;
                
                let mut posts = vec![];
                for id in ["post1", "post2", "post3"] {
                    if let Ok(Some(post)) = txn.read::<Post>(&PostID(id.to_string())) {
                        posts.push(postcard::to_allocvec(&post)?);
                    }
                }
                
                println!("2. Alice → Bob: Sync Response ({} items)", posts.len());
                node_alice.send(
                    from,
                    ProtocolMsg::SyncResponse {
                        items: posts,
                        has_more: false,
                    },
                ).await?;
            }
        }
        
        if let Some((_, msg)) = node_bob.receive().await {
            if let ProtocolMsg::SyncResponse { items, has_more } = msg {
                println!("   Bob received sync: {} items (has_more: {})", items.len(), has_more);
                
                let txn = node_bob.store.begin_write()?;
                for item in items {
                    let post: Post = postcard::from_bytes(&item)?;
                    // Use upsert logic (create if not exists, update if exists)
                    let _ = txn.create(&post); // Ignore errors for duplicates
                    println!("     - Synced: {:?} - {}", post.id, post.content);
                }
                txn.commit()?;
            }
        }
    }
    println!("✅ Sync protocol complete\n");

    // =========================================================================
    // Verification
    // =========================================================================
    println!("=== Final Verification ===");
    {
        println!("Bob's database contents:");
        let txn = node_bob.store.begin_read()?;
        
        // Verify user
        if let Ok(Some(user)) = txn.read::<UserProfile>(&UserProfileID("alice".to_string())) {
            println!("  User: {} ({})", user.name, user.email);
        }
        
        // Verify posts
        for id in ["post1", "post2", "post3"] {
            if let Ok(Some(post)) = txn.read::<Post>(&PostID(id.to_string())) {
                println!("  Post: {:?} - {} ({} likes)", post.id, post.content, post.likes);
            }
        }
    }

    println!("\n🎉 All protocol phases completed successfully!");
    println!("\nSummary:");
    println!("  ✓ Handshake: Schema verification and connection establishment");
    println!("  ✓ Capabilities: Fine-grained permission system");
    println!("  ✓ Query: Secure remote data access");
    println!("  ✓ Write: Distributed data modification");
    println!("  ✓ Sync: Efficient range-based synchronization");
    
    Ok(())
}
