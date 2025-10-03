//! # Getting Started with Netabase
//!
//! This example demonstrates the basic usage of Netabase for both local
//! and distributed database operations. It covers:
//! - Creating simple models with primary and secondary keys
//! - Basic CRUD operations
//! - Secondary key queries
//! - Setting up distributed networking
//!
//! Run with: `cargo run --example getting_started`

use std::time::Duration;

use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseModel, NetabaseSecondaryKeyQuery},
};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

// Step 1: Define your data models using the schema module
#[netabase_schema_module(AppSchema, AppKeys)]
mod app_schema {
    use super::*;

    /// A simple user model with primary and secondary keys
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64, // Primary key
        pub name: String,
        #[secondary_key]
        pub email: String, // Secondary key for efficient email lookups
        #[secondary_key]
        pub active: bool, // Secondary key for status queries
        pub created_at: u64,
    }

    /// A simple task model showing foreign key relationships
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TaskKey)]
    pub struct Task {
        #[key]
        pub id: u64, // Primary key
        pub title: String,
        pub description: String,
        #[secondary_key]
        pub user_id: u64, // Foreign key to User (secondary key for queries)
        #[secondary_key]
        pub completed: bool, // Secondary key for filtering
        pub created_at: u64,
    }
}

use app_schema::*;

async fn local_database_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Local Database Example ===");

    // Step 2: Create a local database
    let db = NetabaseSledDatabase::<AppSchema>::new_with_name("getting_started_db")?;

    // Get trees for each model type
    let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
    let task_tree: NetabaseSledTree<Task, TaskKey> = db.get_main_tree()?;

    println!("✓ Database created and trees initialized");

    // Step 3: Create some sample data
    let users = vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            active: true,
            created_at: chrono::Utc::now().timestamp() as u64,
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            active: true,
            created_at: chrono::Utc::now().timestamp() as u64,
        },
        User {
            id: 3,
            name: "Carol Davis".to_string(),
            email: "carol@example.com".to_string(),
            active: false,
            created_at: chrono::Utc::now().timestamp() as u64,
        },
    ];

    // Step 4: Insert users using CRUD operations
    for user in &users {
        user_tree.insert(user.key(), user.clone())?;
    }
    println!("✓ Inserted {} users", users.len());

    // Step 5: Create tasks for users
    let tasks = vec![
        Task {
            id: 1,
            title: "Learn Rust".to_string(),
            description: "Complete the Rust programming tutorial".to_string(),
            user_id: 1, // Alice's task
            completed: false,
            created_at: chrono::Utc::now().timestamp() as u64,
        },
        Task {
            id: 2,
            title: "Build a web app".to_string(),
            description: "Create a simple web application".to_string(),
            user_id: 1, // Alice's task
            completed: true,
            created_at: chrono::Utc::now().timestamp() as u64,
        },
        Task {
            id: 3,
            title: "Write documentation".to_string(),
            description: "Document the project for other developers".to_string(),
            user_id: 2, // Bob's task
            completed: false,
            created_at: chrono::Utc::now().timestamp() as u64,
        },
    ];

    for task in &tasks {
        task_tree.insert(task.key(), task.clone())?;
    }
    println!("✓ Inserted {} tasks", tasks.len());

    // Step 6: Demonstrate primary key queries
    println!("\n--- Primary Key Queries ---");

    // Get a user by ID (primary key)
    let user_key = UserKey::Primary(UserPrimaryKey(1));
    if let Some(user) = user_tree.get(user_key)? {
        println!("Found user by ID 1: {}", user.name);
    }

    // Get a task by ID
    let task_key = TaskKey::Primary(TaskPrimaryKey(2));
    if let Some(task) = task_tree.get(task_key)? {
        println!(
            "Found task by ID 2: '{}' (completed: {})",
            task.title, task.completed
        );
    }

    // Step 7: Demonstrate secondary key queries
    println!("\n--- Secondary Key Queries ---");

    // Find user by email
    let users_by_email = user_tree
        .query_by_secondary_key(UserSecondaryKeys::EmailKey("alice@example.com".to_string()))?;
    println!(
        "Users with email 'alice@example.com': {}",
        users_by_email.len()
    );

    // Find all active users
    let active_users = user_tree.query_by_secondary_key(UserSecondaryKeys::ActiveKey(true))?;
    println!("Active users: {}", active_users.len());

    // Find tasks by user (foreign key query)
    let alice_tasks = task_tree.query_by_secondary_key(TaskSecondaryKeys::User_idKey(1))?;
    println!("Tasks for user 1 (Alice): {}", alice_tasks.len());

    // Find completed tasks
    let completed_tasks =
        task_tree.query_by_secondary_key(TaskSecondaryKeys::CompletedKey(true))?;
    println!("Completed tasks: {}", completed_tasks.len());

    // Step 8: Demonstrate iteration
    println!("\n--- Database Iteration ---");

    println!("All users in database:");
    for result in user_tree.iter() {
        let (key, user) = result?;
        println!(
            "  - {} ({}) - Active: {}",
            user.name, user.email, user.active
        );
    }

    println!("\nAll tasks in database:");
    for result in task_tree.iter() {
        let (key, task) = result?;
        println!(
            "  - '{}' (User: {}, Completed: {})",
            task.title, task.user_id, task.completed
        );
    }

    // Step 9: Demonstrate updates
    println!("\n--- Update Operations ---");

    if let Some(mut task) = task_tree.get(TaskKey::Primary(TaskPrimaryKey(1)))? {
        task.completed = true;
        task_tree.insert(task.key(), task.clone())?;
        println!("✓ Marked task '{}' as completed", task.title);
    }

    // Step 10: Show final statistics
    println!("\n--- Database Statistics ---");
    println!("Total users: {}", user_tree.len());
    println!("Total tasks: {}", task_tree.len());

    Ok(())
}

async fn distributed_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Distributed Database Example ===");

    // Step 1: Create a distributed Netabase instance
    let mut netabase = Netabase::<AppSchema>::new_with_path("distributed_getting_started")?;

    // Step 2: Start the network swarm
    netabase.start_swarm().await?;
    println!("✓ Network swarm started");

    // Step 3: Join the DHT network (bootstrap)
    println!("Attempting to join the network...");
    match timeout(Duration::from_secs(10), netabase.bootstrap()).await {
        Ok(result) => match result {
            Ok(_) => println!("✓ Successfully joined the DHT network"),
            Err(e) => println!(
                "⚠ Bootstrap completed with issues: {} (normal for single-node)",
                e
            ),
        },
        Err(_) => println!("⚠ Bootstrap timed out (normal for single-node setups)"),
    }

    // Step 4: Create sample data for the network
    let user = User {
        id: 100,
        name: "Network User".to_string(),
        email: "network@example.com".to_string(),
        active: true,
        created_at: chrono::Utc::now().timestamp() as u64,
    };

    let task = Task {
        id: 200,
        title: "Distributed Task".to_string(),
        description: "A task stored in the distributed network".to_string(),
        user_id: 100,
        completed: false,
        created_at: chrono::Utc::now().timestamp() as u64,
    };

    // Step 5: Store records in the DHT
    println!("\n--- Storing Records in DHT ---");

    match timeout(Duration::from_secs(10), netabase.put_record(user.clone())).await {
        Ok(result) => match result {
            Ok(_) => println!("✓ User stored in DHT successfully"),
            Err(e) => println!("⚠ Failed to store user: {}", e),
        },
        Err(_) => println!("⚠ Store operation timed out"),
    }

    match timeout(Duration::from_secs(10), netabase.put_record(task.clone())).await {
        Ok(result) => match result {
            Ok(_) => println!("✓ Task stored in DHT successfully"),
            Err(e) => println!("⚠ Failed to store task: {}", e),
        },
        Err(_) => println!("⚠ Store operation timed out"),
    }

    // Step 6: Retrieve records from the DHT
    println!("\n--- Retrieving Records from DHT ---");

    let user_key = UserKey::Primary(UserPrimaryKey(100));
    match timeout(Duration::from_secs(10), netabase.get_record(user_key)).await {
        Ok(result) => match result {
            Ok(_) => println!("✓ User retrieval query completed"),
            Err(e) => println!("⚠ Failed to retrieve user: {}", e),
        },
        Err(_) => println!("⚠ Retrieval operation timed out"),
    }

    // Step 7: Provider operations
    println!("\n--- Provider Operations ---");

    let task_key = TaskKey::Primary(TaskPrimaryKey(200));
    match timeout(
        Duration::from_secs(10),
        netabase.start_providing(task_key.clone()),
    )
    .await
    {
        Ok(result) => match result {
            Ok(_) => println!("✓ Started providing task record"),
            Err(e) => println!("⚠ Failed to start providing: {}", e),
        },
        Err(_) => println!("⚠ Provider operation timed out"),
    }

    // Find providers for the task
    match timeout(
        Duration::from_secs(10),
        netabase.get_providers(task_key.clone()),
    )
    .await
    {
        Ok(result) => match result {
            Ok(_) => println!("✓ Provider query completed"),
            Err(e) => println!("⚠ Failed to get providers: {}", e),
        },
        Err(_) => println!("⚠ Provider query timed out"),
    }

    // Step 8: Subscribe to network events
    println!("\n--- Network Event Monitoring ---");

    let mut receiver = netabase.subscribe_to_broadcasts();

    // Start a background task to monitor events
    let event_monitor = tokio::spawn(async move {
        let mut event_count = 0;
        while event_count < 5 {
            match timeout(Duration::from_secs(2), receiver.recv()).await {
                Ok(Ok(event)) => {
                    println!("📡 Network event received: {:?}", event);
                    event_count += 1;
                }
                Ok(Err(_)) => break, // Channel closed
                Err(_) => break,     // Timeout
            }
        }
        println!("Event monitoring completed");
    });

    // Wait for some events or timeout
    tokio::time::sleep(Duration::from_secs(5)).await;
    event_monitor.abort();

    // Step 9: Cleanup
    netabase.stop_swarm().await?;
    println!("✓ Network swarm stopped");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see what's happening
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Netabase Getting Started Example\n");

    // Run the local database example
    local_database_example().await?;

    // Run the distributed database example
    distributed_example().await?;

    println!("\n🎉 Getting Started Example Complete!");
    println!("\nWhat you learned:");
    println!("  ✅ How to define models with #[derive(NetabaseModel)]");
    println!("  ✅ How to create schemas with #[netabase_schema_module]");
    println!("  ✅ How to perform CRUD operations on local databases");
    println!("  ✅ How to use secondary keys for efficient queries");
    println!("  ✅ How to set up distributed networking with DHT");
    println!("  ✅ How to store and retrieve records across the network");
    println!("  ✅ How to monitor network events and providers");

    println!("\nNext Steps:");
    println!("  📖 Check out the blog_system.rs example for advanced usage");
    println!("  📖 Read the NETABASE_GUIDE.md for comprehensive documentation");
    println!("  🔨 Try building your own application with Netabase!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_db");

        let db =
            NetabaseSledDatabase::<AppSchema>::new_with_name(&db_path.to_string_lossy()).unwrap();
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree().unwrap();

        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            active: true,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        // Test insert and get
        user_tree.insert(user.key(), user.clone()).unwrap();
        let retrieved = user_tree.get(user.key()).unwrap();
        assert_eq!(retrieved, Some(user.clone()));

        // Test secondary key query
        let users_by_email = user_tree
            .query_by_secondary_key(UserSecondaryKeys::EmailKey("test@example.com".to_string()))
            .unwrap();
        assert_eq!(users_by_email.len(), 1);
        assert_eq!(users_by_email[0], user);
    }

    #[tokio::test]
    async fn test_distributed_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut netabase = Netabase::<AppSchema>::new_with_path(temp_dir.path()).unwrap();

        assert!(netabase.start_swarm().await.is_ok());
        assert!(netabase.stop_swarm().await.is_ok());
    }
}
