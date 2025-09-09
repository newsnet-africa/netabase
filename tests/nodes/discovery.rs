use std::time::Duration;

use crate::nodes;
use crate::nodes::discovery::schemas::TestItem;
use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase::netabase_trait::NetabaseSchema;
use netabase_macros::NetabaseSchema;
use netabase_macros::NetabaseSchemaKey;
use netabase_macros::schema_module;
use tokio::time::sleep;

#[schema_module(MyRegistery)]
pub mod schemas {
    use bincode::{Decode, Encode};
    use netabase::netabase_trait::NetabaseSchema as NetabaseSchemaTrait;
    use netabase_macros::NetabaseSchema;
    use netabase_macros::NetabaseSchemaKey;

    #[derive(NetabaseSchema, Encode, Decode, Debug, Clone)]
    pub struct TestItem {
        #[key]
        pub some_key: String,
        pub other_value: String,
    }
}
#[tokio::test]
async fn scratch1() {
    let test_item = TestItem {
        some_key: "Some".to_string(),
        other_value: "Other".to_string(),
    };
    // Use mDNS auto-connect enabled version
    let mut net1 =
        Netabase::<MyRegisterySchemaKey, MyRegisterySchema>::new_test_with_mdns_auto_connect(
            1, true, true,
        );
    let mut list = net1.swarm_event_listener.resubscribe();
    println!("Node 1 (server) started, waiting for events...");

    // Wait longer to allow peer discovery and connections
    for i in 0..10 {
        match tokio::time::timeout(Duration::from_secs(2), list.recv()).await {
            Ok(Ok(event)) => println!("Node 1 Event {}: {:?}", i, event),
            Ok(Err(_)) => break,
            Err(_) => println!("Node 1: No events in 2s ({})", i),
        }
    }

    println!("Node 1 closing...");
    net1.close().await;
}
#[tokio::test]
async fn scratch2() {
    let test_item = TestItem {
        some_key: "Some".to_string(),
        other_value: "Other".to_string(),
    };
    // Use mDNS auto-connect enabled version - also make this node a server so it can store
    let mut net2 =
        Netabase::<MyRegisterySchemaKey, MyRegisterySchema>::new_test_with_mdns_auto_connect(
            2, true, true,
        );
    let mut list = net2.swarm_event_listener.resubscribe();

    println!("Node 2 (server with auto-connect) started, waiting for first event...");
    println!("{:?}", list.recv().await);

    println!("Waiting for peer discovery and connections...");
    sleep(Duration::from_secs(8)).await;

    println!("Attempting to put data to DHT...");
    let put_result = net2
        .put(
            nodes::discovery::MyRegisterySchemaKey::TestItem(test_item.key()),
            nodes::discovery::MyRegisterySchema::TestItem(test_item.clone()),
        )
        .await;

    match put_result {
        Ok(_) => println!("✅ PUT operation successful!"),
        Err(e) => println!("❌ PUT operation failed: {:?}", e),
    }

    // Try to get the data back
    sleep(Duration::from_secs(2)).await;
    println!("Attempting to get data from DHT...");
    let get_result = net2
        .get(nodes::discovery::MyRegisterySchemaKey::TestItem(
            test_item.key(),
        ))
        .await;

    match get_result {
        Ok(Some(retrieved)) => println!("✅ GET operation successful! Retrieved: {:?}", retrieved),
        Ok(None) => println!("⚠️ GET operation successful but no data found"),
        Err(e) => println!("❌ GET operation failed: {:?}", e),
    }

    // Wait a bit more to see any additional events
    println!("Monitoring events for a few more seconds...");
    for i in 0..5 {
        match tokio::time::timeout(Duration::from_secs(1), list.recv()).await {
            Ok(Ok(event)) => println!("Node 2 Event {}: {:?}", i, event),
            Ok(Err(_)) => break,
            Err(_) => println!("Node 2: No events in 1s ({})", i),
        }
    }

    println!("Node 2 closing...");
    net2.close().await;
}
