//! WASM tests for Netabase Store
//!
//! Run with: wasm-pack test --headless --firefox --features wasm

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(all(feature = "wasm", test))]
mod wasm_store_tests {
    use super::*;
    use netabase_macros::{NetabaseModel, netabase_definition_module};
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    // Define test schema
    #[netabase_definition_module(TestDefinition, TestKeys)]
    mod test_schema {
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
        pub struct TestModel {
            #[primary_key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub category: String,
        }
    }

    use test_schema::*;

    #[wasm_bindgen_test]
    async fn test_indexeddb_create_and_get() {
        let db_name = format!("test_db_{}", js_sys::Date::now());
        let store = IndexedDBStore::<TestDefinition>::new(&db_name)
            .await
            .expect("Failed to create store");

        let tree = store.open_tree::<TestModel>();

        let model = TestModel {
            id: 1,
            name: "Test Item".to_string(),
            category: "test".to_string(),
        };

        tree.put(model.clone())
            .await
            .expect("Failed to put model");

        let retrieved = tree
            .get(TestModelPrimaryKey(1))
            .await
            .expect("Failed to get model")
            .expect("Model not found");

        assert_eq!(retrieved, model);
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_secondary_key_query() {
        let db_name = format!("test_db_{}", js_sys::Date::now());
        let store = IndexedDBStore::<TestDefinition>::new(&db_name)
            .await
            .expect("Failed to create store");

        let tree = store.open_tree::<TestModel>();

        // Insert multiple items with same category
        let model1 = TestModel {
            id: 1,
            name: "Item 1".to_string(),
            category: "books".to_string(),
        };

        let model2 = TestModel {
            id: 2,
            name: "Item 2".to_string(),
            category: "books".to_string(),
        };

        let model3 = TestModel {
            id: 3,
            name: "Item 3".to_string(),
            category: "movies".to_string(),
        };

        tree.put(model1.clone()).await.expect("Failed to put model1");
        tree.put(model2.clone()).await.expect("Failed to put model2");
        tree.put(model3.clone()).await.expect("Failed to put model3");

        // Query by secondary key
        let books = tree
            .get_by_secondary_key(CategorySecondaryKey("books".to_string()))
            .await
            .expect("Failed to query by secondary key");

        assert_eq!(books.len(), 2);
        assert!(books.contains(&model1));
        assert!(books.contains(&model2));
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_remove() {
        let db_name = format!("test_db_{}", js_sys::Date::now());
        let store = IndexedDBStore::<TestDefinition>::new(&db_name)
            .await
            .expect("Failed to create store");

        let tree = store.open_tree::<TestModel>();

        let model = TestModel {
            id: 1,
            name: "To Remove".to_string(),
            category: "temp".to_string(),
        };

        tree.put(model.clone()).await.expect("Failed to put model");

        let removed = tree
            .remove(TestModelPrimaryKey(1))
            .await
            .expect("Failed to remove model");

        assert!(removed);

        let retrieved = tree
            .get(TestModelPrimaryKey(1))
            .await
            .expect("Failed to get model");

        assert!(retrieved.is_none());
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_iteration() {
        let db_name = format!("test_db_{}", js_sys::Date::now());
        let store = IndexedDBStore::<TestDefinition>::new(&db_name)
            .await
            .expect("Failed to create store");

        let tree = store.open_tree::<TestModel>();

        // Insert multiple items
        for i in 1..=5 {
            let model = TestModel {
                id: i,
                name: format!("Item {}", i),
                category: "test".to_string(),
            };
            tree.put(model).await.expect("Failed to put model");
        }

        // Iterate and count
        let items = tree.iter().await.expect("Failed to iterate");

        assert_eq!(items.len(), 5);
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_update() {
        let db_name = format!("test_db_{}", js_sys::Date::now());
        let store = IndexedDBStore::<TestDefinition>::new(&db_name)
            .await
            .expect("Failed to create store");

        let tree = store.open_tree::<TestModel>();

        let original = TestModel {
            id: 1,
            name: "Original".to_string(),
            category: "test".to_string(),
        };

        tree.put(original.clone())
            .await
            .expect("Failed to put original");

        let updated = TestModel {
            id: 1,
            name: "Updated".to_string(),
            category: "modified".to_string(),
        };

        tree.put(updated.clone())
            .await
            .expect("Failed to put updated");

        let retrieved = tree
            .get(TestModelPrimaryKey(1))
            .await
            .expect("Failed to get model")
            .expect("Model not found");

        assert_eq!(retrieved, updated);
        assert_ne!(retrieved, original);
    }
}
