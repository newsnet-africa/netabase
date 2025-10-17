#![cfg(all(feature = "libp2p", feature = "native"))]

use libp2p::kad::store::RecordStore;
use libp2p::kad::{Record, RecordKey as Key};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::sled_store::SledStore;

// Test schema
#[netabase_definition_module(DhtDefinition, DhtKeys)]
mod dht_schema {
    use netabase_deps::{bincode, serde};
    use netabase_macros::NetabaseModel;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    pub struct Article {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub published: bool,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    pub struct Counter {
        #[primary_key]
        pub id: String,
        pub count: u64,
    }
}

use dht_schema::*;

/// Test that demonstrates serialization/deserialization between stores
/// This simulates how data could be transferred between SledStore and IndexedDB
#[test]
fn test_cross_store_record_serialization() {
    let mut sled_store = SledStore::<DhtDefinition>::temp().unwrap();

    // Create test data
    let article = Article {
        id: 1,
        title: "Cross-Store Compatibility".to_string(),
        content: "Testing data portability between stores".to_string(),
        author_id: 42,
        published: true,
    };

    // Encode as a Record (as would be done for DHT/P2P)
    let article_key = ArticlePrimaryKey(1);
    let key_bytes = bincode::encode_to_vec(
        &DhtKeys::ArticleKey(dht_schema::ArticleKey::Primary(article_key)),
        bincode::config::standard(),
    )
    .unwrap();
    let record_key = Key::from(key_bytes);

    let definition = DhtDefinition::Article(article.clone());
    let value_bytes = bincode::encode_to_vec(&definition, bincode::config::standard()).unwrap();

    let record = Record {
        key: record_key.clone(),
        value: value_bytes.clone(),
        publisher: None,
        expires: None,
    };

    // Put into SledStore via RecordStore trait
    assert!(sled_store.put(record.clone()).is_ok());

    // Retrieve from SledStore
    let retrieved_record = sled_store.get(&record_key);
    assert!(retrieved_record.is_some());

    let retrieved_record = retrieved_record.unwrap();
    assert_eq!(record.key, retrieved_record.key);
    assert_eq!(record.value, retrieved_record.value);

    // Demonstrate parsing: decode the record value back to Definition
    let (decoded_definition, _): (DhtDefinition, _) =
        bincode::decode_from_slice(&retrieved_record.value, bincode::config::standard()).unwrap();

    // Verify we can convert back to the original model
    match decoded_definition {
        DhtDefinition::Article(decoded_article) => {
            assert_eq!(article.id, decoded_article.id);
            assert_eq!(article.title, decoded_article.title);
            assert_eq!(article.content, decoded_article.content);
            assert_eq!(article.author_id, decoded_article.author_id);
            assert_eq!(article.published, decoded_article.published);
        }
        _ => panic!("Expected Article variant"),
    }

    // This demonstrates that the same Record format could be used with IndexedDB:
    // 1. The key is already serialized bytes (compatible with IndexedDB keys)
    // 2. The value is serialized Definition bytes (compatible with IndexedDB values)
    // 3. The bincode encoding/decoding is the same across both stores
}

/// Test multiple record types through RecordStore trait
#[test]
fn test_multi_model_record_parsing() {
    let mut sled_store = SledStore::<DhtDefinition>::temp().unwrap();

    // Create Article
    let article = Article {
        id: 100,
        title: "First Article".to_string(),
        content: "Content here".to_string(),
        author_id: 1,
        published: true,
    };

    // Create Counter
    let counter = Counter {
        id: "views".to_string(),
        count: 42,
    };

    // Store Article as Record
    let article_key = ArticlePrimaryKey(100);
    let article_key_bytes = bincode::encode_to_vec(
        &DhtKeys::ArticleKey(dht_schema::ArticleKey::Primary(article_key)),
        bincode::config::standard(),
    )
    .unwrap();
    let article_record_key = Key::from(article_key_bytes);

    let article_definition = DhtDefinition::Article(article.clone());
    let article_value_bytes =
        bincode::encode_to_vec(&article_definition, bincode::config::standard()).unwrap();

    let article_record = Record {
        key: article_record_key.clone(),
        value: article_value_bytes,
        publisher: None,
        expires: None,
    };

    // Store Counter as Record
    let counter_key = CounterPrimaryKey("views".to_string());
    let counter_key_bytes = bincode::encode_to_vec(
        &DhtKeys::CounterKey(dht_schema::CounterKey::Primary(counter_key)),
        bincode::config::standard(),
    )
    .unwrap();
    let counter_record_key = Key::from(counter_key_bytes);

    let counter_definition = DhtDefinition::Counter(counter.clone());
    let counter_value_bytes =
        bincode::encode_to_vec(&counter_definition, bincode::config::standard()).unwrap();

    let counter_record = Record {
        key: counter_record_key.clone(),
        value: counter_value_bytes,
        publisher: None,
        expires: None,
    };

    // Put both via RecordStore
    assert!(sled_store.put(article_record).is_ok());
    assert!(sled_store.put(counter_record).is_ok());

    // Retrieve and parse Article
    let retrieved_article = sled_store.get(&article_record_key).unwrap();
    let (article_def, _): (DhtDefinition, _) =
        bincode::decode_from_slice(&retrieved_article.value, bincode::config::standard()).unwrap();

    match article_def {
        DhtDefinition::Article(a) => {
            assert_eq!(article.id, a.id);
            assert_eq!(article.title, a.title);
        }
        _ => panic!("Expected Article"),
    }

    // Retrieve and parse Counter
    let retrieved_counter = sled_store.get(&counter_record_key).unwrap();
    let (counter_def, _): (DhtDefinition, _) =
        bincode::decode_from_slice(&retrieved_counter.value, bincode::config::standard()).unwrap();

    match counter_def {
        DhtDefinition::Counter(c) => {
            assert_eq!(counter.id, c.id);
            assert_eq!(counter.count, c.count);
        }
        _ => panic!("Expected Counter"),
    }

    // Demonstrate cross-store compatibility:
    // The records() iterator provides all records in a format that could be
    // transferred to any other store implementing RecordStore
    let all_records: Vec<_> = sled_store.records().collect();
    assert_eq!(2, all_records.len());

    // Each record can be decoded regardless of which store it came from
    for record_cow in all_records {
        let (def, _): (DhtDefinition, _) =
            bincode::decode_from_slice(&record_cow.value, bincode::config::standard()).unwrap();

        match def {
            DhtDefinition::Article(a) => {
                println!("Found article: {}", a.title);
            }
            DhtDefinition::Counter(c) => {
                println!("Found counter: {} = {}", c.id, c.count);
            }
        }
    }
}

/// Test demonstrating how Records could be migrated between stores
#[test]
fn test_record_migration_pattern() {
    // Create source store
    let mut source_store = SledStore::<DhtDefinition>::temp().unwrap();

    // Add some data
    let articles = vec![
        Article {
            id: 1,
            title: "Article 1".to_string(),
            content: "Content 1".to_string(),
            author_id: 1,
            published: true,
        },
        Article {
            id: 2,
            title: "Article 2".to_string(),
            content: "Content 2".to_string(),
            author_id: 1,
            published: false,
        },
        Article {
            id: 3,
            title: "Article 3".to_string(),
            content: "Content 3".to_string(),
            author_id: 2,
            published: true,
        },
    ];

    for article in &articles {
        let key = ArticlePrimaryKey(article.id);
        let key_bytes = bincode::encode_to_vec(
            &DhtKeys::ArticleKey(dht_schema::ArticleKey::Primary(key)),
            bincode::config::standard(),
        )
        .unwrap();

        let definition = DhtDefinition::Article(article.clone());
        let value_bytes = bincode::encode_to_vec(&definition, bincode::config::standard()).unwrap();

        let record = Record {
            key: Key::from(key_bytes),
            value: value_bytes,
            publisher: None,
            expires: None,
        };

        source_store.put(record).unwrap();
    }

    // Create destination store
    let mut dest_store = SledStore::<DhtDefinition>::temp().unwrap();

    // Migration: Read all records from source and write to destination
    let records_to_migrate: Vec<_> = source_store.records().map(|r| r.into_owned()).collect();

    assert_eq!(3, records_to_migrate.len());

    for record in records_to_migrate {
        // This is the key operation: Records are portable between stores
        dest_store.put(record).unwrap();
    }

    // Verify all data was migrated
    assert_eq!(3, dest_store.records().count());

    // Verify data integrity after migration
    for article in &articles {
        let key = ArticlePrimaryKey(article.id);
        let key_bytes = bincode::encode_to_vec(
            &DhtKeys::ArticleKey(dht_schema::ArticleKey::Primary(key)),
            bincode::config::standard(),
        )
        .unwrap();

        let record_key = Key::from(key_bytes);
        let retrieved = dest_store.get(&record_key).unwrap();

        let (def, _): (DhtDefinition, _) =
            bincode::decode_from_slice(&retrieved.value, bincode::config::standard()).unwrap();

        match def {
            DhtDefinition::Article(a) => {
                assert_eq!(article.id, a.id);
                assert_eq!(article.title, a.title);
                assert_eq!(article.content, a.content);
            }
            _ => panic!("Expected Article"),
        }
    }

    // This pattern demonstrates how:
    // 1. SledStore (native) can export Records
    // 2. Those Records can be imported into another store
    // 3. The same would work for IndexedDB (WASM) with the RecordStore trait
    // 4. Data format is consistent across implementations
}
