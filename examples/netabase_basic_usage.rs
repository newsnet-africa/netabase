//! # Basic Netabase Usage Example
//!
//! This example demonstrates the fundamental usage of Netabase for distributed
//! database operations. It shows how to:
//! - Define data models with primary and secondary keys
//! - Create a distributed database instance
//! - Store and retrieve data across the network
//! - Handle basic network operations
//!
//! Run with: `cargo run --example netabase_basic_usage`

use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel;
use serde::{Deserialize, Serialize};

// Define your data models
#[netabase_schema_module(BlogSchema, BlogKeys)]
mod blog {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }
}

use blog::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Netabase basic usage example...");

    // Create a distributed database instance
    let mut netabase = Netabase::<BlogSchema>::new()?;

    println!("Starting network swarm...");
    netabase.start_swarm().await?;

    // Create and store a user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    println!("Storing user: {:?}", user);

    // Store in the distributed hash table
    let put_result = netabase.put_record(user.clone()).await?;
    println!("Put result: {:?}", put_result);

    // Retrieve from the network
    let user_key = UserKey::Primary(UserPrimaryKey(1));
    println!("Retrieving user with key: {:?}", user_key);

    let get_result = netabase.get_record(user_key).await?;
    println!("Get result: {:?}", get_result);

    // Clean shutdown
    println!("Stopping swarm...");
    netabase.stop_swarm().await?;

    println!("Example completed successfully!");
    Ok(())
}
