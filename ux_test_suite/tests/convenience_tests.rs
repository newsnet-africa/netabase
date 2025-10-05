//! # Convenience Tests
//!
//! This module tests the convenience re-exports provided by `netabase_deps`.
//! While macro hygiene ensures that users don't NEED to import dependencies,
//! the convenience re-exports allow users to easily access the same versions
//! of dependencies that the macros use for their own derives and implementations.

use ux_test_suite::{TestConfig, TestResult, TestRunner};

/// Test that all re-exported dependencies are accessible
#[test]
fn test_basic_dependency_reexports() -> TestResult {
    // Test that we can access all re-exported dependencies
    use netabase_store::{bincode, derive_more, serde, sled, strum};

    // Test serde
    #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
    struct SerdeTest {
        id: u64,
        name: String,
    }

    let test_data = SerdeTest {
        id: 1,
        name: "test".to_string(),
    };

    let json = serde_json::to_string(&test_data).unwrap();
    let deserialized: SerdeTest = serde_json::from_str(&json).unwrap();
    assert_eq!(test_data, deserialized);

    // Test bincode
    #[derive(bincode::Encode, bincode::Decode, Clone, Debug, PartialEq)]
    struct BincodeTest {
        id: u64,
        data: Vec<u8>,
    }

    let bincode_data = BincodeTest {
        id: 42,
        data: vec![1, 2, 3, 4],
    };

    let encoded = bincode::encode_to_vec(&bincode_data, bincode::config::standard())?;
    let (decoded, _): (BincodeTest, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard())?;
    assert_eq!(bincode_data, decoded);

    // Test strum
    #[derive(strum::EnumString, strum::Display, Clone, Debug, PartialEq)]
    enum StrumTest {
        VariantA,
        VariantB,
    }

    let variant = StrumTest::VariantA;
    let string_form = variant.to_string();
    let parsed: StrumTest = string_form.parse()?;
    assert_eq!(variant, parsed);

    // Test derive_more
    #[derive(derive_more::From, derive_more::Into, Clone, Debug, PartialEq)]
    struct DeriveMoreTest(u64);

    let value: DeriveMoreTest = 42u64.into();
    let back: u64 = value.into();
    assert_eq!(back, 42);

    Ok(())
}

/// Test that convenience re-exports work with NetabaseModel
#[test]
fn test_reexports_with_netabase_model() -> TestResult {
    use netabase_store::{bincode, serde, strum, NetabaseModel};

    #[derive(
        strum::EnumString,
        strum::Display,
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
    enum UserStatus {
        #[default]
        Active,
        Inactive,
        Suspended,
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
    #[key_name(ConvenienceUserKey)]
    struct ConvenienceUser {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub status: UserStatus,
        #[secondary_key]
        pub email: String,
        pub metadata: std::collections::HashMap<String, String>,
    }

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("department".to_string(), "engineering".to_string());

    let user = ConvenienceUser {
        id: 1,
        name: "Alice".to_string(),
        status: UserStatus::Active,
        email: "alice@example.com".to_string(),
        metadata,
    };

    // Test that the model works with trait methods
    use netabase_store::traits::NetabaseModel;
    let _key = user.key();

    // Test serialization works
    let json = serde_json::to_string(&user)?;
    let _deserialized: ConvenienceUser = serde_json::from_str(&json)?;

    // Test binary encoding works
    let encoded = bincode::encode_to_vec(&user, bincode::config::standard())?;
    let _decoded: (ConvenienceUser, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard())?;

    // Test enum string conversion works
    assert_eq!(user.status.to_string(), "Active");
    let _parsed_status: UserStatus = "Active".parse()?;

    Ok(())
}

/// Test that re-exports work in schema modules
#[test]
fn test_reexports_in_schema_modules() -> TestResult {
    use netabase_store::{bincode, netabase_schema_module, serde, strum, NetabaseModel};

    #[netabase_schema_module(ConvenienceSchema, ConvenienceSchemaKeys)]
    mod convenience_schema {
        use super::*;
        use netabase_store::{traits::NetabaseModel as _, NetabaseModel};

        #[derive(
            strum::EnumString,
            strum::Display,
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
        pub enum Priority {
            #[default]
            Low,
            Medium,
            High,
            Critical,
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
        #[key_name(TaskKey)]
        pub struct Task {
            #[key]
            pub id: u64,
            pub title: String,
            pub description: String,
            #[secondary_key]
            pub priority: Priority,
            #[secondary_key]
            pub assigned_to: Option<u64>,
            #[secondary_key]
            pub completed: bool,
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
            pub owner_id: u64,
            #[secondary_key]
            pub active: bool,
            pub tags: Vec<String>,
        }
    }

    use convenience_schema::*;

    let task = Task {
        id: 1,
        title: "Test Task".to_string(),
        description: "A test task for convenience testing".to_string(),
        priority: Priority::High,
        assigned_to: Some(42),
        completed: false,
        created_at: 1600000000,
    };

    let project = Project {
        id: 1,
        name: "Test Project".to_string(),
        owner_id: 42,
        active: true,
        tags: vec!["test".to_string(), "convenience".to_string()],
    };

    // Test that schema types work
    let _schema_task = ConvenienceSchema::Task(task.clone());
    let _schema_project = ConvenienceSchema::Project(project.clone());

    // Test serialization
    let task_json = serde_json::to_string(&task)?;
    let _task_back: Task = serde_json::from_str(&task_json)?;

    // Test enum functionality
    assert_eq!(task.priority.to_string(), "High");
    let _priority: Priority = "High".parse()?;

    Ok(())
}

/// Test derive_more features work correctly
#[test]
fn test_derive_more_features() -> TestResult {
    use netabase_store::{bincode, derive_more, serde, NetabaseModel};

    #[derive(
        derive_more::From,
        derive_more::Into,
        // derive_more::AsRef,  // AsRef not available in this version
        // derive_more::AsMut,  // AsMut not available in this version
        Clone,
        Debug,
        PartialEq,
        Eq,
        Hash,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    struct UserId(u64);

    #[derive(
        derive_more::From,
        derive_more::Into,
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
    struct Email(String);

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
    #[key_name(DeriveMoreUserKey)]
    struct DeriveMoreUser {
        #[key]
        pub id: UserId,
        pub name: String,
        #[secondary_key]
        pub email: Email,
        #[secondary_key]
        pub active: bool,
    }

    // Test From/Into conversions
    let user_id: UserId = 42u64.into();
    let id_value: u64 = user_id.clone().into();
    assert_eq!(id_value, 42);

    let email: Email = "test@example.com".to_string().into();
    let email_value: String = email.clone().into();
    assert_eq!(email_value, "test@example.com");

    // Test AsRef/AsMut (commented out due to version compatibility)
    // let id_ref: &u64 = user_id.as_ref();
    // assert_eq!(*id_ref, 42);

    let user = DeriveMoreUser {
        id: user_id,
        name: "Test User".to_string(),
        email,
        active: true,
    };

    // Test that NetabaseModel works with wrapped types
    use netabase_store::traits::NetabaseModel;
    let _key = user.key();

    Ok(())
}

/// Test version compatibility between macro-generated code and user code
#[test]
fn test_version_compatibility() -> TestResult {
    use netabase_store::{bincode, serde, NetabaseModel};

    // Test that user can serialize data created by macros
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
    #[key_name(CompatUserKey)]
    struct CompatUser {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub active: bool,
    }

    let user = CompatUser {
        id: 1,
        name: "Compat Test".to_string(),
        active: true,
    };

    // Test that we can manually serialize using the same versions
    let manual_json = serde_json::to_string(&user)?;
    let manual_bincode = bincode::encode_to_vec(&user, bincode::config::standard())?;

    // Test that we can deserialize what the macro might have serialized
    let _user_from_json: CompatUser = serde_json::from_str(&manual_json)?;
    let (_user_from_bincode, _): (CompatUser, usize) =
        bincode::decode_from_slice(&manual_bincode, bincode::config::standard())?;

    Ok(())
}

/// Test that private re-exports work correctly
#[test]
fn test_private_reexports() -> TestResult {
    // Test that we can use the private re-exports when needed
    use netabase_store::__macro_deps::__private;

    // These should be the same as the public re-exports
    let _bincode = __private::bincode::config::standard();
    // Test that private re-exports are accessible (simplified test)
    let _result = __private::bincode::config::standard();

    // Test that a struct using private re-exports works
    #[derive(Clone, Debug)]
    struct PrivateTest {
        data: String,
    }

    // Manual implementation using private re-exports (simulating macro usage)
    impl __private::serde::Serialize for PrivateTest {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: __private::serde::Serializer,
        {
            self.data.serialize(serializer)
        }
    }

    let test = PrivateTest {
        data: "private test".to_string(),
    };

    let _json = serde_json::to_string(&test)?;

    Ok(())
}

/// Test error handling with re-exported dependencies
#[test]
fn test_error_handling() -> TestResult {
    use netabase_store::{bincode, serde, NetabaseModel};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    #[key_name(ErrorTestKey)]
    struct ErrorTest {
        #[key]
        pub id: u64,
        pub value: String,
    }

    let test = ErrorTest {
        id: 1,
        value: "test".to_string(),
    };

    // Test serialization error handling
    let json = serde_json::to_string(&test)?;

    // Test deserialization with invalid data
    let invalid_json = r#"{"id": "not_a_number", "value": "test"}"#;
    let result: Result<ErrorTest, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());

    // Test binary encoding
    let encoded = bincode::encode_to_vec(&test, bincode::config::standard())?;

    // Test decoding with truncated data
    let truncated = &encoded[..encoded.len() - 1];
    let decode_result: Result<(ErrorTest, usize), _> =
        bincode::decode_from_slice(truncated, bincode::config::standard());
    assert!(decode_result.is_err());

    Ok(())
}

/// Integration test using the test framework
#[test]
fn test_convenience_with_framework() -> TestResult {
    let config = TestConfig::new("convenience_framework_test")
        .with_description("Test convenience re-exports using the test framework");

    let runner = TestRunner::new(config);

    runner.run(|_config| {
        use netabase_store::{bincode, serde, strum, NetabaseModel};

        #[derive(
            strum::EnumString,
            strum::Display,
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
        enum FrameworkTestType {
            #[default]
            TypeOne,
            TypeTwo,
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
        #[key_name(FrameworkConvenienceKey)]
        struct FrameworkConvenience {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub test_type: FrameworkTestType,
        }

        let model = FrameworkConvenience {
            id: 42,
            name: "Framework test".to_string(),
            test_type: FrameworkTestType::TypeOne,
        };

        // Test all functionality works together
        use netabase_store::traits::NetabaseModel;
        let _key = model.key();

        let json = serde_json::to_string(&model)?;
        let _back: FrameworkConvenience = serde_json::from_str(&json)?;

        assert_eq!(model.test_type.to_string(), "TypeOne");

        Ok(())
    })
}

/// Test performance of re-exported dependencies
#[test]
fn test_reexport_performance() -> TestResult {
    use netabase_store::{bincode, serde, NetabaseModel};
    use std::time::Instant;

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
    struct PerfTest {
        #[key]
        pub id: u64,
        pub data: Vec<u8>,
        #[secondary_key]
        pub category: u32,
    }

    let test_data = PerfTest {
        id: 1,
        data: vec![0u8; 1000], // 1KB of data
        category: 42,
    };

    // Test JSON serialization performance
    let start = Instant::now();
    for _ in 0..100 {
        let _json = serde_json::to_string(&test_data)?;
    }
    let json_duration = start.elapsed();

    // Test binary serialization performance
    let start = Instant::now();
    for _ in 0..100 {
        let _encoded = bincode::encode_to_vec(&test_data, bincode::config::standard())?;
    }
    let bincode_duration = start.elapsed();

    // Binary should generally be faster than JSON
    println!("JSON serialization (100x): {:?}", json_duration);
    println!("Bincode serialization (100x): {:?}", bincode_duration);

    // Just verify both complete successfully
    assert!(json_duration.as_millis() < 1000); // Should be much faster
    assert!(bincode_duration.as_millis() < 1000);

    Ok(())
}

/// Test that re-exports don't interfere with user's own dependency versions
#[test]
fn test_no_version_conflicts() -> TestResult {
    // This test simulates a user having their own versions of dependencies
    use netabase_store::{bincode, serde, NetabaseModel}; // Re-exported versions

    // Simulate user's own serde usage (would be their version in real scenario)
    #[derive(
        serde::Serialize, serde::Deserialize, Clone, Debug, bincode::Encode, bincode::Decode,
    )]
    struct UserStruct {
        name: String,
        value: i32,
    }

    // Use Netabase with re-exported versions
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    #[key_name(NoConflictKey)]
    struct NoConflictModel {
        #[key]
        pub id: u64,
        pub user_data: UserStruct,
    }

    let user_struct = UserStruct {
        name: "test".to_string(),
        value: 42,
    };

    let model = NoConflictModel {
        id: 1,
        user_data: user_struct,
    };

    // Test that both work together
    let json = serde_json::to_string(&model)?;
    let _back: NoConflictModel = serde_json::from_str(&json)?;

    use netabase_store::traits::NetabaseModel;
    let _key = model.key();

    Ok(())
}
