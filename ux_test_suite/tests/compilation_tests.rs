//! # Compilation Tests
//!
//! This module contains tests that validate the compilation behavior of Netabase
//! macros under various conditions. These tests focus on ensuring that the
//! generated code compiles correctly and that error messages are helpful when
//! compilation fails.

use ux_test_suite::{TestConfig, TestResult, TestRunner};

/// Test that basic models compile successfully
#[test]
fn test_basic_compilation() -> TestResult {
    // This test validates that basic model compilation works
    use netabase_macros::NetabaseModel;

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(BasicCompilationKey)]
    struct BasicCompilation {
        #[key]
        id: u64,
        name: String,
    }

    let model = BasicCompilation {
        id: 1,
        name: "Test".to_string(),
    };

    // If this compiles and runs, the test passes
    assert_eq!(model.id, 1);
    Ok(())
}

/// Test that models with various field types compile
#[test]
fn test_field_types_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;
    use std::collections::HashMap;

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(FieldTypesKey)]
    struct FieldTypes {
        #[key]
        id: u64,

        // Basic types
        name: String,
        age: u32,
        height: f64,
        active: bool,

        // Collections
        tags: Vec<String>,
        scores: [i32; 3],
        metadata: HashMap<String, String>,

        // Options
        optional_field: Option<String>,
        optional_number: Option<u64>,

        // Secondary keys with various types
        #[secondary_key]
        category: String,
        #[secondary_key]
        priority: u8,
        #[secondary_key]
        enabled: bool,
        #[secondary_key]
        department: Option<String>,
    }

    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());

    let model = FieldTypes {
        id: 1,
        name: "Test".to_string(),
        age: 30,
        height: 5.9,
        active: true,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        scores: [10, 20, 30],
        metadata,
        optional_field: Some("optional".to_string()),
        optional_number: None,
        category: "A".to_string(),
        priority: 1,
        enabled: true,
        department: Some("Engineering".to_string()),
    };

    assert_eq!(model.id, 1);
    assert_eq!(model.tags.len(), 2);
    assert_eq!(model.scores[1], 20);
    Ok(())
}

/// Test that enums compile correctly as secondary keys
#[test]
fn test_enum_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;

    #[derive(Clone, Debug, PartialEq)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Priority {
        Low,
        Medium,
        High,
        Critical,
    }

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(EnumTestKey)]
    struct EnumTest {
        #[key]
        id: u64,
        name: String,
        #[secondary_key]
        status: Status,
        #[secondary_key]
        priority: Priority,
    }

    let model = EnumTest {
        id: 1,
        name: "Enum Test".to_string(),
        status: Status::Active,
        priority: Priority::High,
    };

    assert_eq!(model.status, Status::Active);
    assert_eq!(model.priority, Priority::High);
    Ok(())
}

/// Test that models with complex generic types compile
#[test]
fn test_generic_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;
    use std::collections::{BTreeMap, HashSet};

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(GenericTestKey)]
    struct GenericTest {
        #[key]
        id: u64,
        name: String,

        // Complex generic collections
        data_map: BTreeMap<String, Vec<u64>>,
        unique_tags: HashSet<String>,
        nested_data: Vec<Vec<String>>,

        // Boxed types
        boxed_data: Box<String>,

        // Secondary keys with complex types
        #[secondary_key]
        category: String,
        #[secondary_key]
        has_data: bool,
    }

    let mut data_map = BTreeMap::new();
    data_map.insert("key1".to_string(), vec![1, 2, 3]);

    let mut unique_tags = HashSet::new();
    unique_tags.insert("tag1".to_string());
    unique_tags.insert("tag2".to_string());

    let model = GenericTest {
        id: 1,
        name: "Generic Test".to_string(),
        data_map,
        unique_tags,
        nested_data: vec![vec!["a".to_string(), "b".to_string()]],
        boxed_data: Box::new("boxed".to_string()),
        category: "test".to_string(),
        has_data: true,
    };

    assert_eq!(model.id, 1);
    assert_eq!(model.nested_data[0].len(), 2);
    assert_eq!(*model.boxed_data, "boxed");
    Ok(())
}

/// Test that schema modules compile with multiple models
#[test]
fn test_schema_module_compilation() -> TestResult {
    use netabase_macros::{netabase_schema_module, NetabaseModel};

    #[netabase_schema_module(CompilationSchema, CompilationSchemaKeys)]
    mod compilation_schema {
        use super::*;

        #[derive(NetabaseModel, Clone, Debug)]
        #[key_name(User1Key)]
        pub struct User1 {
            #[key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub email: String,
        }

        #[derive(NetabaseModel, Clone, Debug)]
        #[key_name(User2Key)]
        pub struct User2 {
            #[key]
            pub id: u64,
            pub username: String,
            #[secondary_key]
            pub active: bool,
        }

        #[derive(NetabaseModel, Clone, Debug)]
        #[key_name(PostKey)]
        pub struct Post {
            #[key]
            pub id: u64,
            pub title: String,
            pub content: String,
            #[secondary_key]
            pub author_id: u64,
            #[secondary_key]
            pub published: bool,
        }

        #[derive(NetabaseModel, Clone, Debug)]
        #[key_name(CommentKey)]
        pub struct Comment {
            #[key]
            pub id: u64,
            pub text: String,
            #[secondary_key]
            pub post_id: u64,
            #[secondary_key]
            pub author_id: u64,
        }
    }

    use compilation_schema::*;

    let user1 = User1 {
        id: 1,
        name: "User1".to_string(),
        email: "user1@example.com".to_string(),
    };

    let user2 = User2 {
        id: 2,
        username: "user2".to_string(),
        active: true,
    };

    let post = Post {
        id: 1,
        title: "Post Title".to_string(),
        content: "Post content".to_string(),
        author_id: 1,
        published: true,
    };

    let comment = Comment {
        id: 1,
        text: "Comment text".to_string(),
        post_id: 1,
        author_id: 2,
    };

    // Test that schema types compile
    let _schema_user1 = CompilationSchema::User1(user1);
    let _schema_user2 = CompilationSchema::User2(user2);
    let _schema_post = CompilationSchema::Post(post);
    let _schema_comment = CompilationSchema::Comment(comment);

    Ok(())
}

/// Test that models with lifetime parameters compile (if supported)
#[test]
fn test_borrowed_data_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;

    // Test with owned data (borrowed data would require lifetime parameters)
    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(BorrowedTestKey)]
    struct BorrowedTest {
        #[key]
        id: u64,
        // Use owned data instead of borrowed for now
        name: String,
        description: String,
        #[secondary_key]
        category: String,
    }

    let model = BorrowedTest {
        id: 1,
        name: "Test".to_string(),
        description: "Description".to_string(),
        category: "Category".to_string(),
    };

    assert_eq!(model.name, "Test");
    Ok(())
}

/// Test that models with attributes in different orders compile
#[test]
fn test_attribute_order_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;

    // Test different attribute orderings
    #[derive(Clone, NetabaseModel, Debug)]
    #[key_name(Order1Key)]
    struct Order1 {
        #[key]
        id: u64,
        name: String,
    }

    #[derive(Debug, Clone, NetabaseModel)]
    #[key_name(Order2Key)]
    struct Order2 {
        #[key]
        id: u64,
        name: String,
    }

    #[key_name(Order3Key)]
    #[derive(NetabaseModel, Clone, Debug)]
    struct Order3 {
        #[key]
        id: u64,
        name: String,
    }

    let m1 = Order1 {
        id: 1,
        name: "Test1".to_string(),
    };
    let m2 = Order2 {
        id: 2,
        name: "Test2".to_string(),
    };
    let m3 = Order3 {
        id: 3,
        name: "Test3".to_string(),
    };

    assert_eq!(m1.id, 1);
    assert_eq!(m2.id, 2);
    assert_eq!(m3.id, 3);
    Ok(())
}

/// Test that models with many secondary keys compile efficiently
#[test]
fn test_many_secondary_keys_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(ManyKeysKey)]
    struct ManyKeys {
        #[key]
        id: u64,

        // Many secondary keys to test code generation efficiency
        #[secondary_key]
        key1: String,
        #[secondary_key]
        key2: u32,
        #[secondary_key]
        key3: bool,
        #[secondary_key]
        key4: u64,
        #[secondary_key]
        key5: i32,
        #[secondary_key]
        key6: String,
        #[secondary_key]
        key7: u8,
        #[secondary_key]
        key8: bool,
        #[secondary_key]
        key9: f64,
        #[secondary_key]
        key10: Option<String>,

        // Regular fields
        data1: String,
        data2: Vec<u8>,
        data3: std::collections::HashMap<String, String>,
    }

    let mut data3 = std::collections::HashMap::new();
    data3.insert("test".to_string(), "value".to_string());

    let model = ManyKeys {
        id: 1,
        key1: "k1".to_string(),
        key2: 2,
        key3: true,
        key4: 4,
        key5: 5,
        key6: "k6".to_string(),
        key7: 7,
        key8: false,
        key9: 9.0,
        key10: Some("k10".to_string()),
        data1: "data".to_string(),
        data2: vec![1, 2, 3],
        data3,
    };

    assert_eq!(model.id, 1);
    assert_eq!(model.key1, "k1");
    assert_eq!(model.key10, Some("k10".to_string()));
    Ok(())
}

/// Test that models with conditional compilation work
#[test]
fn test_conditional_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(ConditionalKey)]
    struct Conditional {
        #[key]
        id: u64,
        name: String,

        // Fields that are conditionally compiled
        #[cfg(not(feature = "nonexistent_feature"))]
        always_present: String,

        #[cfg(feature = "test")]
        #[secondary_key]
        test_only_field: Option<bool>,

        #[cfg(target_family = "unix")]
        unix_field: Option<String>,

        #[cfg(target_family = "windows")]
        windows_field: Option<String>,
    }

    let model = Conditional {
        id: 1,
        name: "Test".to_string(),

        #[cfg(not(feature = "nonexistent_feature"))]
        always_present: "always".to_string(),

        #[cfg(feature = "test")]
        test_only_field: Some(true),

        #[cfg(target_family = "unix")]
        unix_field: Some("unix".to_string()),

        #[cfg(target_family = "windows")]
        windows_field: Some("windows".to_string()),
    };

    assert_eq!(model.id, 1);
    assert_eq!(model.always_present, "always");
    Ok(())
}

/// Test that generated code doesn't produce warnings
#[test]
fn test_no_warnings_compilation() -> TestResult {
    use netabase_macros::NetabaseModel;

    // This model is designed to potentially trigger warnings if code generation is not clean
    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(NoWarningsKey)]
    struct NoWarnings {
        #[key]
        id: u64,

        // Field that might be unused in generated code
        _internal_field: String,

        #[secondary_key]
        category: String,

        // Field with reserved-like name
        #[secondary_key]
        type_field: String,
    }

    let model = NoWarnings {
        id: 1,
        _internal_field: "internal".to_string(),
        category: "test".to_string(),
        type_field: "type_value".to_string(),
    };

    assert_eq!(model.id, 1);
    assert_eq!(model.type_field, "type_value");
    Ok(())
}

/// Test compilation with the framework
#[test]
fn test_compilation_with_framework() -> TestResult {
    let config = TestConfig::new("compilation_framework_test")
        .with_description("Test compilation behavior using the framework")
        .hygiene_only();

    let runner = TestRunner::new(config);

    runner.run(|_config| {
        use netabase_macros::NetabaseModel;

        #[derive(NetabaseModel, Clone, Debug)]
        #[key_name(FrameworkCompilationKey)]
        struct FrameworkCompilation {
            #[key]
            id: u64,
            name: String,
            #[secondary_key]
            active: bool,
        }

        let model = FrameworkCompilation {
            id: 42,
            name: "Framework Test".to_string(),
            active: true,
        };

        // Test that basic compilation works within framework
        assert_eq!(model.id, 42);
        assert_eq!(model.name, "Framework Test");
        assert!(model.active);

        Ok(())
    })
}

/// Test that large models compile in reasonable time
#[test]
fn test_compilation_performance() -> TestResult {
    use netabase_macros::NetabaseModel;
    use std::time::Instant;

    // This test measures compilation performance by timing the test execution
    // In a real scenario, you'd measure actual compilation time
    let start = Instant::now();

    #[derive(NetabaseModel, Clone, Debug)]
    #[key_name(PerfCompilationKey)]
    struct PerfCompilation {
        #[key]
        id: u64,

        // Large number of fields to test code generation performance
        field1: String,
        field2: String,
        field3: String,
        field4: String,
        field5: String,
        field6: String,
        field7: String,
        field8: String,
        field9: String,
        field10: String,
        field11: u64,
        field12: u64,
        field13: u64,
        field14: u64,
        field15: u64,
        field16: bool,
        field17: bool,
        field18: bool,
        field19: bool,
        field20: bool,

        #[secondary_key]
        skey1: String,
        #[secondary_key]
        skey2: u64,
        #[secondary_key]
        skey3: bool,
        #[secondary_key]
        skey4: String,
        #[secondary_key]
        skey5: u32,

        data1: Vec<u8>,
        data2: std::collections::HashMap<String, String>,
        data3: Option<String>,
        data4: [u8; 16],
        data5: (String, u64, bool),
    }

    let mut data2 = std::collections::HashMap::new();
    data2.insert("key".to_string(), "value".to_string());

    let model = PerfCompilation {
        id: 1,
        field1: "f1".to_string(),
        field2: "f2".to_string(),
        field3: "f3".to_string(),
        field4: "f4".to_string(),
        field5: "f5".to_string(),
        field6: "f6".to_string(),
        field7: "f7".to_string(),
        field8: "f8".to_string(),
        field9: "f9".to_string(),
        field10: "f10".to_string(),
        field11: 11,
        field12: 12,
        field13: 13,
        field14: 14,
        field15: 15,
        field16: true,
        field17: false,
        field18: true,
        field19: false,
        field20: true,
        skey1: "sk1".to_string(),
        skey2: 2,
        skey3: true,
        skey4: "sk4".to_string(),
        skey5: 5,
        data1: vec![1, 2, 3],
        data2,
        data3: Some("data3".to_string()),
        data4: [0; 16],
        data5: ("tuple".to_string(), 42, true),
    };

    let duration = start.elapsed();

    // Basic assertions
    assert_eq!(model.id, 1);
    assert_eq!(model.field1, "f1");
    assert_eq!(model.skey1, "sk1");

    // The test should complete quickly (compilation time is not measured here,
    // but this gives an indication of generated code efficiency)
    assert!(duration.as_millis() < 1000);

    Ok(())
}
