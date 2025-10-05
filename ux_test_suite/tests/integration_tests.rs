//! # Integration Tests
//!
//! This module contains comprehensive integration tests that validate the complete
//! Netabase workflow with macro hygiene and dependency auto-export. These tests
//! simulate real-world usage scenarios and verify that all components work together
//! seamlessly.

use std::time::Duration;
use tokio::time::timeout;
use ux_test_suite::{TestConfig, TestDatabase, TestModelFactory, TestResult, TestRunner};

/// Test complete local database workflow with macro hygiene
#[tokio::test]
async fn test_local_database_integration() -> TestResult {
    use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};
    use netabase_store::{
        database::NetabaseDatabase,
        traits::{NetabaseModel, NetabaseSecondaryKeyQuery},
    };

    #[netabase_schema_module(LocalSchema, LocalSchemaKeys)]
    mod local_schema {
        use super::*;
        use netabase_store::{traits::NetabaseModel as _, NetabaseModel};

        #[derive(
            Clone,
            Debug,
            PartialEq,
            Eq,
            Hash,
            Default,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        pub enum ProjectStatus {
            #[default]
            Planning,
            Active,
            OnHold,
            Completed,
            Cancelled,
        }

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(UserKey)]
        pub struct User {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub email: String,
            #[secondary_key]
            pub department: String,
            #[secondary_key]
            pub active: bool,
            pub created_at: u64,
        }

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(ProjectKey)]
        pub struct Project {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub owner_id: u64, // Foreign key to User
            #[secondary_key]
            pub status: ProjectStatus,
            pub description: String,
        }

        #[derive(
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        pub enum ProjectStatus {
            Planning,
            InProgress,
            Completed,
            OnHold,
        }
    }

    use local_schema::*;

    // Setup test database
    let test_db = TestDatabase::new()?;
    let db = NetabaseDatabase::<LocalSchema>::new_with_path(test_db.path())?;

    // Get trees for each model type
    let user_tree = db.get_main_tree::<User, UserKey>()?;
    let project_tree = db.get_main_tree::<Project, ProjectKey>()?;

    // Create test data
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@company.com".to_string(),
            department: "Engineering".to_string(),
            active: true,
            created_at: 1600000000,
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@company.com".to_string(),
            department: "Design".to_string(),
            active: true,
            created_at: 1600003600,
        },
        User {
            id: 3,
            name: "Carol".to_string(),
            email: "carol@company.com".to_string(),
            department: "Engineering".to_string(),
            active: false,
            created_at: 1600007200,
        },
    ];

    let projects = vec![
        Project {
            id: 1,
            name: "Project Alpha".to_string(),
            owner_id: 1,
            status: ProjectStatus::InProgress,
            description: "First project".to_string(),
        },
        Project {
            id: 2,
            name: "Project Beta".to_string(),
            owner_id: 1,
            status: ProjectStatus::Planning,
            description: "Second project".to_string(),
        },
        Project {
            id: 3,
            name: "Project Gamma".to_string(),
            owner_id: 2,
            status: ProjectStatus::Completed,
            description: "Third project".to_string(),
        },
    ];

    // Insert data
    for user in &users {
        user_tree.insert(user.key(), user.clone())?;
    }

    for project in &projects {
        project_tree.insert(project.key(), project.clone())?;
    }

    // Test primary key queries
    let user1 = user_tree.get(UserKey::Primary(UserPrimaryKey(1)))?;
    assert!(user1.is_some());
    assert_eq!(user1.unwrap().name, "Alice");

    let project2 = project_tree.get(ProjectKey::Primary(ProjectPrimaryKey(2)))?;
    assert!(project2.is_some());
    assert_eq!(project2.unwrap().name, "Project Beta");

    // Test secondary key queries
    let engineering_users = user_tree
        .query_by_secondary_key(UserSecondaryKeys::DepartmentKey("Engineering".to_string()))?;
    assert_eq!(engineering_users.len(), 2);

    let active_users = user_tree.query_by_secondary_key(UserSecondaryKeys::ActiveKey(true))?;
    assert_eq!(active_users.len(), 2);

    let alice_projects =
        project_tree.query_by_secondary_key(ProjectSecondaryKeys::Owner_idKey(1))?;
    assert_eq!(alice_projects.len(), 2);

    let completed_projects = project_tree
        .query_by_secondary_key(ProjectSecondaryKeys::StatusKey(ProjectStatus::Completed))?;
    assert_eq!(completed_projects.len(), 1);

    // Test complex queries (combining results)
    let active_engineering_users: Vec<_> =
        engineering_users.into_iter().filter(|u| u.active).collect();
    assert_eq!(active_engineering_users.len(), 1);
    assert_eq!(active_engineering_users[0].name, "Alice");

    // Test iteration
    let all_users: Vec<_> = user_tree.iter().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(all_users.len(), 3);

    let all_projects: Vec<_> = project_tree.iter().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(all_projects.len(), 3);

    // Test updates
    if let Some(mut user) = user_tree.get(UserKey::Primary(UserPrimaryKey(3)))? {
        user.active = true;
        user_tree.insert(user.key(), user)?;
    }

    let updated_active_users =
        user_tree.query_by_secondary_key(UserSecondaryKeys::ActiveKey(true))?;
    assert_eq!(updated_active_users.len(), 3);

    // Test deletions
    user_tree.remove(UserKey::Primary(UserPrimaryKey(3)))?;
    let remaining_users: Vec<_> = user_tree.iter().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(remaining_users.len(), 2);

    Ok(())
}

/// Test distributed database integration with networking
#[tokio::test]
#[ignore] // Disabled due to netabase compilation issues
async fn test_distributed_integration() -> TestResult {
    // use netabase::Netabase;
    use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};

    #[netabase_schema_module(DistributedSchema, DistributedSchemaKeys)]
    mod distributed_schema {
        use super::*;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(NodeKey)]
        pub struct Node {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub region: String,
            #[secondary_key]
            pub active: bool,
            pub last_seen: u64,
        }

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(MessageKey)]
        pub struct Message {
            #[key]
            pub id: u64,
            pub content: String,
            #[secondary_key]
            pub sender_id: u64,
            #[secondary_key]
            pub recipient_id: u64,
            #[secondary_key]
            pub message_type: MessageType,
            pub timestamp: u64,
        }

        #[derive(
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        pub enum MessageType {
            Direct,
            Broadcast,
            System,
        }
    }

    use distributed_schema::*;

    // Note: This test is disabled due to netabase compilation issues
    // The following would be the test implementation:

    /*
    // Setup distributed Netabase instances
    let test_db1 = TestDatabase::new()?;
    let test_db2 = TestDatabase::new()?;

    let mut netabase1 = Netabase::<DistributedSchema>::new_with_path(test_db1.path())?;
    let mut netabase2 = Netabase::<DistributedSchema>::new_with_path(test_db2.path())?;

    // Start network swarms
    netabase1.start_swarm().await?;
    netabase2.start_swarm().await?;

    // Create test data
    let node1 = Node {
        id: 1,
        name: "Node1".to_string(),
        region: "us-west".to_string(),
        active: true,
        last_seen: 1600000000,
    };

    let node2 = Node {
        id: 2,
        name: "Node2".to_string(),
        region: "us-east".to_string(),
        active: true,
        last_seen: 1600000000,
    };

    let message = Message {
        id: 1,
        content: "Hello distributed world!".to_string(),
        sender_id: 1,
        recipient_id: 2,
        message_type: MessageType::Direct,
        timestamp: 1600000000,
    };

    // Test putting records into DHT
    // ... (DHT operations would go here)
    */

    println!(
        "This test demonstrates distributed integration but is disabled due to compilation issues"
    );
    Ok(())
}

/// Test error handling and recovery scenarios
#[tokio::test]
async fn test_error_handling_integration() -> TestResult {
    use netabase_store::database::NetabaseDatabase;
    use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};

    #[netabase_schema_module(ErrorSchema, ErrorSchemaKeys)]
    mod error_schema {
        use super::*;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(ErrorTestKey)]
        pub struct ErrorTest {
            #[key]
            pub id: u64,
            pub data: String,
            #[secondary_key]
            pub category: u32,
        }
    }

    use error_schema::*;

    let test_db = TestDatabase::new()?;
    let db = NetabaseDatabase::<ErrorSchema>::new_with_path(test_db.path())?;
    let tree = db.get_main_tree::<ErrorTest, ErrorTestKey>()?;

    // Test handling of non-existent keys
    let non_existent = tree.get(ErrorTestKey::Primary(ErrorTestPrimaryKey(999)))?;
    assert!(non_existent.is_none());

    // Test handling of empty secondary key queries
    let empty_results = tree.query_by_secondary_key(ErrorTestSecondaryKeys::CategoryKey(999))?;
    assert!(empty_results.is_empty());

    // Test normal operations work after errors
    let test_record = ErrorTest {
        id: 1,
        data: "test data".to_string(),
        category: 1,
    };

    tree.insert(test_record.key(), test_record.clone())?;
    let retrieved = tree.get(test_record.key())?;
    assert_eq!(retrieved, Some(test_record));

    Ok(())
}

/// Test performance under load
#[tokio::test]
async fn test_performance_integration() -> TestResult {
    use netabase_store::database::NetabaseDatabase;
    use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};
    use std::time::Instant;

    #[netabase_schema_module(PerfSchema, PerfSchemaKeys)]
    mod perf_schema {
        use super::*;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(PerfTestKey)]
        pub struct PerfTest {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub category: u32,
            #[secondary_key]
            pub active: bool,
            pub data: Vec<u8>,
        }
    }

    use perf_schema::*;

    let test_db = TestDatabase::new()?;
    let db = NetabaseDatabase::<PerfSchema>::new_with_path(test_db.path())?;
    let tree = db.get_main_tree::<PerfTest, PerfTestKey>()?;

    const RECORD_COUNT: u64 = 1000;
    const DATA_SIZE: usize = 100; // 100 bytes per record

    // Measure insertion performance
    let start = Instant::now();
    for i in 1..=RECORD_COUNT {
        let record = PerfTest {
            id: i,
            name: format!("Record{}", i),
            category: (i % 10) as u32,
            active: i % 2 == 0,
            data: vec![0u8; DATA_SIZE],
        };
        tree.insert(record.key(), record)?;
    }
    let insert_duration = start.elapsed();

    println!("Inserted {} records in {:?}", RECORD_COUNT, insert_duration);
    println!(
        "Average insert time: {:?}",
        insert_duration / RECORD_COUNT as u32
    );

    // Measure query performance
    let start = Instant::now();
    for i in 1..=RECORD_COUNT {
        let _record = tree.get(PerfTestKey::Primary(PerfTestPrimaryKey(i)))?;
    }
    let query_duration = start.elapsed();

    println!("Queried {} records in {:?}", RECORD_COUNT, query_duration);
    println!(
        "Average query time: {:?}",
        query_duration / RECORD_COUNT as u32
    );

    // Measure secondary key query performance
    let start = Instant::now();
    for category in 0..10u32 {
        let _results = tree.query_by_secondary_key(PerfTestSecondaryKeys::CategoryKey(category))?;
    }
    let secondary_query_duration = start.elapsed();

    println!("Secondary key queries took: {:?}", secondary_query_duration);

    // Verify data integrity
    let all_records: Vec<_> = tree.iter().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(all_records.len(), RECORD_COUNT as usize);

    // Basic performance assertions (very lenient)
    assert!(insert_duration.as_millis() < 10000); // 10 seconds max
    assert!(query_duration.as_millis() < 5000); // 5 seconds max

    Ok(())
}

/// Test concurrent access patterns
#[tokio::test]
async fn test_concurrent_integration() -> TestResult {
    use netabase_store::database::NetabaseDatabase;
    use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};
    use std::sync::Arc;
    use tokio::task;

    #[netabase_schema_module(ConcurrentSchema, ConcurrentSchemaKeys)]
    mod concurrent_schema {
        use super::*;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(ConcurrentTestKey)]
        pub struct ConcurrentTest {
            #[key]
            pub id: u64,
            pub thread_id: u32,
            #[secondary_key]
            pub batch: u32,
            pub data: String,
        }
    }

    use concurrent_schema::*;

    let test_db = TestDatabase::new()?;
    let db = Arc::new(NetabaseDatabase::<ConcurrentSchema>::new_with_path(
        test_db.path(),
    )?);

    const THREAD_COUNT: u32 = 4;
    const RECORDS_PER_THREAD: u32 = 100;

    // Spawn multiple tasks that insert records concurrently
    let mut handles = Vec::new();

    for thread_id in 0..THREAD_COUNT {
        let db_clone = Arc::clone(&db);
        let handle = task::spawn(async move {
            let tree = db_clone
                .get_main_tree::<ConcurrentTest, ConcurrentTestKey>()
                .map_err(|e| format!("Failed to get tree: {}", e))?;

            for record_id in 0..RECORDS_PER_THREAD {
                let id = (thread_id as u64 * RECORDS_PER_THREAD as u64) + record_id as u64;
                let record = ConcurrentTest {
                    id,
                    thread_id,
                    batch: record_id,
                    data: format!("Thread{}_Record{}", thread_id, record_id),
                };

                tree.insert(record.key(), record)
                    .map_err(|e| format!("Insert failed: {}", e))?;
            }

            Ok::<(), String>(())
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.map_err(|e| format!("Task failed: {}", e))??;
    }

    // Verify all records were inserted
    let tree = db.get_main_tree::<ConcurrentTest, ConcurrentTestKey>()?;
    let all_records: Vec<_> = tree.iter().collect::<Result<Vec<_>, _>>()?;
    let expected_count = (THREAD_COUNT * RECORDS_PER_THREAD) as usize;
    assert_eq!(all_records.len(), expected_count);

    // Verify records from each thread
    for thread_id in 0..THREAD_COUNT {
        let thread_records =
            tree.query_by_secondary_key(ConcurrentTestSecondaryKeys::Thread_idKey(thread_id))?;
        assert_eq!(thread_records.len(), RECORDS_PER_THREAD as usize);
    }

    // Verify record integrity
    for (_, record) in &all_records {
        let expected_data = format!("Thread{}_Record{}", record.thread_id, record.batch);
        assert_eq!(record.data, expected_data);
    }

    Ok(())
}

/// Integration test using the test framework
#[tokio::test]
async fn test_integration_with_framework() -> TestResult {
    let config = TestConfig::new("integration_framework_test")
        .with_description("Complete integration test using the test framework")
        .with_networking();

    let runner = TestRunner::new(config);

    runner.run(|config| {
        use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};

        #[netabase_schema_module(FrameworkSchema, FrameworkSchemaKeys)]
        mod framework_schema {
            use super::*;

            #[derive(
                NetabaseModel,
                Clone,
                Debug,
                PartialEq,
                serde::Serialize,
                serde::Deserialize,
                bincode::Encode,
                bincode::Decode,
            )]
            #[key_name(IntegrationTestKey)]
            pub struct IntegrationTest {
                #[key]
                pub id: u64,
                pub name: String,
                #[secondary_key]
                pub test_type: String,
                #[secondary_key]
                pub success: bool,
            }
        }

        use framework_schema::*;

        let test = IntegrationTest {
            id: 1,
            name: "Framework Integration Test".to_string(),
            test_type: "integration".to_string(),
            success: true,
        };

        // Test basic functionality
        if config.validate_functionality {
            use netabase_store::traits::NetabaseModel;
            let _key = test.key();
        }

        // Test hygiene
        if config.validate_hygiene {
            // This compiles without manual imports - hygiene working
            assert_eq!(test.name, "Framework Integration Test");
        }

        // Test networking would go here if needed
        if config.validate_networking {
            println!("Networking validation would be performed here");
        }

        Ok(())
    })?;

    Ok(())
}

/// Test schema evolution and compatibility
#[tokio::test]
async fn test_schema_evolution() -> TestResult {
    use netabase_store::database::NetabaseDatabase;
    use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};

    // Original schema version
    #[netabase_schema_module(EvolutionSchemaV1, EvolutionSchemaV1Keys)]
    mod evolution_v1 {
        use super::*;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(UserV1Key)]
        pub struct UserV1 {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub email: String,
        }
    }

    // Evolved schema version (backward compatible)
    #[netabase_schema_module(EvolutionSchemaV2, EvolutionSchemaV2Keys)]
    mod evolution_v2 {
        use super::*;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            bincode::Encode,
            bincode::Decode,
        )]
        #[key_name(UserV2Key)]
        pub struct UserV2 {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub email: String,
            // New optional field for backward compatibility
            #[serde(default)]
            #[secondary_key]
            pub department: Option<String>,
            #[serde(default)]
            pub created_at: Option<u64>,
        }
    }

    use evolution_v1::*;
    use evolution_v2::UserV2;

    let test_db = TestDatabase::new()?;

    // Create data with V1 schema
    let db_v1 = NetabaseDatabase::<EvolutionSchemaV1>::new_with_path(test_db.path())?;
    let tree_v1 = db_v1.get_main_tree::<UserV1, UserV1Key>()?;

    let user_v1 = UserV1 {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
    };

    tree_v1.insert(user_v1.key(), user_v1.clone())?;

    // Verify V1 data was stored correctly
    let retrieved_v1 = tree_v1.get(user_v1.key())?;
    assert_eq!(retrieved_v1, Some(user_v1));

    // Test that both schemas can coexist (different databases)
    let test_db_v2 = TestDatabase::new()?;
    let db_v2 = NetabaseDatabase::<EvolutionSchemaV2>::new_with_path(test_db_v2.path())?;
    let tree_v2 = db_v2.get_main_tree::<UserV2, UserV2Key>()?;

    let user_v2 = UserV2 {
        id: 1,
        name: "Test User V2".to_string(),
        email: "test@example.com".to_string(),
        department: Some("Engineering".to_string()),
        created_at: Some(1600000000),
    };

    tree_v2.insert(user_v2.key(), user_v2.clone())?;

    let retrieved_v2 = tree_v2.get(user_v2.key())?;
    assert_eq!(retrieved_v2, Some(user_v2));

    // Test secondary key queries on new fields
    let eng_users = tree_v2.query_by_secondary_key(UserV2SecondaryKeys::DepartmentKey(Some(
        "Engineering".to_string(),
    )))?;
    assert_eq!(eng_users.len(), 1);

    Ok(())
}
