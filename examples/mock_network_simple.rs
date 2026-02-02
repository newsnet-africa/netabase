//! Simple Mock Network Example
//!
//! This example demonstrates a basic protocol using in-memory channels
//! to simulate network communication between two nodes.

use netabase::primitives::NodeId;
use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use netabase_store::traits::database::transaction::NBTransaction;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

// Define our test schema
#[netabase_macros::netabase_networking]
#[netabase_macros::netabase_definition(TestNetwork)]
pub mod test_models {
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
    }
}

use test_models::*;

/// Simple message type for mock network
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Message {
    Hello(String),
    DataRequest(String),
    DataResponse(Option<UserProfile>),
}

/// Mock network node
struct MockNode {
    id: NodeId,
    store: RedbStore<TestNetwork>,
    rx: mpsc::UnboundedReceiver<(NodeId, Message)>,
    router: Arc<Mutex<HashMap<NodeId, mpsc::UnboundedSender<(NodeId, Message)>>>>,
    _tx: mpsc::UnboundedSender<(NodeId, Message)>,
}

impl MockNode {
    fn new(
        id: NodeId,
        router: Arc<Mutex<HashMap<NodeId, mpsc::UnboundedSender<(NodeId, Message)>>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (store, _temp) = RedbStore::<TestNetwork>::new_temporary()?;
        let (tx, rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            id,
            store,
            rx,
            router: router.clone(),
            _tx: tx,
        })
    }
    
    async fn register(&self) {
        let mut r = self.router.lock().await;
        r.insert(self.id.clone(), self._tx.clone());
    }

    async fn send(&self, to: NodeId, msg: Message) -> Result<(), String> {
        let router = self.router.lock().await;
        if let Some(tx) = router.get(&to) {
            tx.send((self.id.clone(), msg))
                .map_err(|e| format!("Send error: {}", e))?;
            Ok(())
        } else {
            Err(format!("Node {:?} not found", to))
        }
    }

    async fn receive(&mut self) -> Option<(NodeId, Message)> {
        self.rx.recv().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Mock Network Example");
    println!("================================\n");

    // Create shared router
    let router = Arc::new(Mutex::new(HashMap::new()));

    // Create two nodes
    let node_id_a = NodeId::from([1u8; 32]);
    let node_id_b = NodeId::from([2u8; 32]);

    let mut node_a = MockNode::new(node_id_a.clone(), router.clone())?;
    let mut node_b = MockNode::new(node_id_b.clone(), router.clone())?;
    
    // Register nodes in router
    node_a.register().await;
    node_b.register().await;

    println!("✅ Created Node A");
    println!("✅ Created Node B\n");

    // Seed Node A with data
    println!("📝 Seeding Node A with data...");
    {
        let txn = node_a.store.begin_write()?;
        
        txn.create(&UserProfile {
            id: UserProfileID("alice".into()),
            name: "Alice".into(),
            email: "alice@example.com".into(),
            bio: "Software engineer".into(),
        })?;
        
        txn.commit()?;
    }
    println!("✅ Node A seeded with 1 user\n");

    // Protocol Demo
    println!("=== Protocol Demo ===");
    
    // 1. Node B sends hello to Node A
    println!("1. Node B → Node A: Hello");
    node_b.send(node_id_a.clone(), Message::Hello("Hello from B!".into())).await?;
    
    if let Some((from, msg)) = node_a.receive().await {
        println!("   Node A received: {:?}", msg);
        
        // 2. Node A responds
        println!("2. Node A → Node B: Hello response");
        node_a.send(from, Message::Hello("Hi B, this is A!".into())).await?;
    }
    
    if let Some((_, msg)) = node_b.receive().await {
        println!("   Node B received: {:?}\n", msg);
    }
    
    // 3. Node B requests data from Node A
    println!("3. Node B → Node A: Data request for 'alice'");
    node_b.send(node_id_a.clone(), Message::DataRequest("alice".into())).await?;
    
    if let Some((from, msg)) = node_a.receive().await {
        if let Message::DataRequest(id) = msg {
            println!("   Node A processing request for: {}", id);
            
            // Query the store
            let txn = node_a.store.begin_read()?;
            let user: Option<UserProfile> = txn.read(&UserProfileID(id.clone()))?;
            
            // Send response
            println!("4. Node A → Node B: Data response");
            node_a.send(from, Message::DataResponse(user)).await?;
        }
    }
    
    if let Some((_, msg)) = node_b.receive().await {
        if let Message::DataResponse(Some(user)) = msg {
            println!("   Node B received user: {} ({})", user.name, user.email);
            
            // Store in Node B
            let txn = node_b.store.begin_write()?;
            txn.create(&user)?;
            txn.commit()?;
            println!("   Node B stored user locally\n");
        }
    }
    
    // Verify
    println!("=== Verification ===");
    {
        let txn_b = node_b.store.begin_read()?;
        let alice: Option<UserProfile> = txn_b.read(&UserProfileID("alice".into()))?;
        
        if let Some(user) = alice {
            println!("✅ Node B successfully has: {} ({})", user.name, user.email);
        }
    }

    println!("\n🎉 Mock network demonstration complete!");
    Ok(())
}
