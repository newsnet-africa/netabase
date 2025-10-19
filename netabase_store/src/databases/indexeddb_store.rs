#[cfg(feature = "wasm")]
use crate::error::NetabaseError;
#[cfg(feature = "wasm")]
use crate::traits::convert::ToIVec;
#[cfg(feature = "wasm")]
use crate::traits::definition::NetabaseDefinitionTrait;
#[cfg(feature = "wasm")]
use crate::traits::model::NetabaseModelTrait;
#[cfg(feature = "wasm")]
use indexed_db_futures::prelude::*;
#[cfg(feature = "wasm")]
use std::marker::PhantomData;
#[cfg(feature = "wasm")]
use strum::IntoEnumIterator;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;
#[cfg(feature = "wasm")]
use web_sys::{IdbCursorDirection, IdbTransactionMode};

/// Type-safe wrapper around IndexedDB that works with NetabaseDefinitionTrait types.
///
/// The IndexedDBStore provides a type-safe interface to the browser's IndexedDB,
/// using discriminants as object store names and ensuring all operations are type-checked.
#[cfg(feature = "wasm")]
pub struct IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait,
{
    db: IdbDatabase,
    db_name: String,
    _phantom: PhantomData<D>,
}

#[cfg(feature = "wasm")]
impl<D> IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Get direct access to the underlying IndexedDB database
    pub fn db(&self) -> &IdbDatabase {
        &self.db
    }

    /// Get the database name
    pub fn db_name(&self) -> &str {
        &self.db_name
    }
}

#[cfg(feature = "wasm")]
impl<D> IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
{
    /// Open a new IndexedDBStore with the given database name
    pub async fn new(db_name: &str) -> Result<Self, NetabaseError> {
        Self::new_with_version(db_name, 1).await
    }

    /// Open a new IndexedDBStore with a specific version number
    pub async fn new_with_version(db_name: &str, version: u32) -> Result<Self, NetabaseError> {
        let mut db_req = IdbDatabase::open_u32(db_name, version)
            .map_err(|e| NetabaseError::Storage(format!("Failed to open IndexedDB: {:?}", e)))?;

        // Set up object stores on upgrade
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            let db = evt.db();

            // Create object stores for each discriminant
            for disc in D::Discriminants::iter() {
                let store_name: String = disc.into();

                // Check if object store already exists
                if !db.object_store_names().any(|name| name == store_name) {
                    // Create object store with auto-incrementing keys disabled
                    // We'll use our own keys
                    let _store = db.create_object_store(&store_name)?;
                }
            }

            // Create secondary key stores
            for disc in D::Discriminants::iter() {
                let store_name: String = disc.into();
                let sec_store_name = format!("{}_secondary", store_name);

                if !db.object_store_names().any(|name| name == sec_store_name) {
                    let _ = db.create_object_store(&sec_store_name)?;
                }
            }

            Ok(())
        }));

        let db = db_req
            .await
            .map_err(|e| NetabaseError::Storage(format!("Failed to open IndexedDB: {:?}", e)))?;

        Ok(Self {
            db,
            db_name: db_name.to_string(),
            _phantom: PhantomData,
        })
    }

    /// Open a tree for a specific model type
    pub fn open_tree<M>(&self) -> IndexedDBStoreTree<'_, D, M>
    where
        M: NetabaseModelTrait + TryFrom<D> + Into<D>,
        D: TryFrom<M> + ToIVec,
    {
        let store_name = M::discriminant_name();
        IndexedDBStoreTree::new(&self.db, store_name)
    }

    /// Get all store names (discriminants) in the database
    pub fn store_names(&self) -> Vec<String> {
        D::Discriminants::iter().map(|d| d.into()).collect()
    }

    /// Close the database connection
    pub fn close(&self) {
        self.db.close();
    }
}

/// Type-safe wrapper around an IndexedDB object store for a specific model type.
///
/// IndexedDBStoreTree provides async CRUD operations for a single model type with automatic
/// encoding/decoding and secondary key management.
#[cfg(feature = "wasm")]
pub struct IndexedDBStoreTree<'a, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait,
{
    db: &'a IdbDatabase,
    store_name: String,
    _phantom_d: PhantomData<D>,
    _phantom_m: PhantomData<M>,
}

#[cfg(feature = "wasm")]
impl<'a, D, M> IndexedDBStoreTree<'a, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec,
    M: NetabaseModelTrait + TryFrom<D> + Into<D>,
{
    /// Create a new IndexedDBStoreTree
    fn new(db: &'a IdbDatabase, store_name: &str) -> Self {
        Self {
            db,
            store_name: store_name.to_string(),
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    /// Insert or update a model in the store
    pub async fn put(&self, model: M) -> Result<(), NetabaseError>
    where
        D: From<M>,
    {
        let primary_key = model.primary_key();
        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let definition: D = model.clone().into();
        let value_bytes = definition.to_ivec()?;

        // Convert bytes to JsValue
        let key_js = js_sys::Uint8Array::from(&key_bytes[..]);
        let value_js = js_sys::Uint8Array::from(&value_bytes[..]);

        // Create transaction
        let tx = self
            .db
            .transaction_on_one_with_mode(&self.store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&self.store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let _request = store
            .put_key_val(&key_js, &value_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to put value: {:?}", e)))?;

        // Wait for transaction to complete
        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        // Handle secondary keys
        let secondary_keys = model.secondary_keys();
        for sec_key in secondary_keys {
            self.insert_secondary_key(&sec_key, &primary_key).await?;
        }

        Ok(())
    }

    /// Get a model by its primary key
    pub async fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let key_js = js_sys::Uint8Array::from(&key_bytes[..]);

        let tx = self.db.transaction_on_one(&self.store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let value = store
            .get(&key_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get value: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Get request failed: {:?}", e)))?;

        match value {
            Some(js_value) => {
                // Convert JsValue to bytes
                let uint8_array = js_sys::Uint8Array::new(&js_value);
                let mut bytes = vec![0u8; uint8_array.length() as usize];
                uint8_array.copy_to(&mut bytes);

                let definition = D::from_ivec(&bytes)?;
                match M::try_from(definition) {
                    Ok(model) => Ok(Some(model)),
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Delete a model by its primary key
    pub async fn remove(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        // First get the model so we can clean up secondary keys
        let model = self.get(key.clone()).await?;

        if model.is_none() {
            return Ok(None);
        }

        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let key_js = js_sys::Uint8Array::from(&key_bytes[..]);

        let tx = self
            .db
            .transaction_on_one_with_mode(&self.store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&self.store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .delete(&key_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to delete value: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Delete request failed: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        // Clean up secondary keys
        if let Some(ref m) = model {
            let secondary_keys = m.secondary_keys();
            for sec_key in secondary_keys {
                self.remove_secondary_key(&sec_key, &key).await?;
            }
        }

        Ok(model)
    }

    /// Get the number of models in the store
    pub async fn len(&self) -> Result<usize, NetabaseError> {
        let tx = self.db.transaction_on_one(&self.store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let count = store
            .count()
            .map_err(|e| NetabaseError::Storage(format!("Failed to count: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Count request failed: {:?}", e)))?;

        Ok(count as usize)
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len().await? == 0)
    }

    /// Clear all models from the store
    pub async fn clear(&self) -> Result<(), NetabaseError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(&self.store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&self.store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .clear()
            .map_err(|e| NetabaseError::Storage(format!("Failed to clear: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Clear request failed: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        // Also clear secondary keys
        let sec_store_name = format!("{}_secondary", self.store_name);
        let tx2 = self
            .db
            .transaction_on_one_with_mode(&sec_store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let sec_store = tx2
            .object_store(&sec_store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        sec_store
            .clear()
            .map_err(|e| NetabaseError::Storage(format!("Failed to clear: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Clear request failed: {:?}", e)))?;

        let _ = tx2
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        Ok(())
    }

    /// Iterate over all models in the store
    pub async fn iter(&self) -> Result<Vec<(M::PrimaryKey, M)>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let tx = self.db.transaction_on_one(&self.store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let cursor_request = store
            .open_cursor()
            .map_err(|e| NetabaseError::Storage(format!("Failed to open cursor: {:?}", e)))?;

        let cursor = cursor_request
            .await
            .map_err(|e| NetabaseError::Storage(format!("Cursor request failed: {:?}", e)))?;

        let mut results = Vec::new();

        if let Some(mut cursor) = cursor {
            loop {
                let key_js = cursor.key().ok_or_else(|| {
                    NetabaseError::Storage("Failed to get cursor key".to_string())
                })?;
                let value_js = cursor.value();

                // Convert key
                let key_array = js_sys::Uint8Array::new(&key_js);
                let mut key_bytes = vec![0u8; key_array.length() as usize];
                key_array.copy_to(&mut key_bytes);

                let (key, _) = bincode::decode_from_slice::<M::PrimaryKey, _>(
                    &key_bytes,
                    bincode::config::standard(),
                )
                .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

                // Convert value
                let value_array = js_sys::Uint8Array::new(&value_js);
                let mut value_bytes = vec![0u8; value_array.length() as usize];
                value_array.copy_to(&mut value_bytes);

                let definition = D::from_ivec(&value_bytes)?;
                let model = M::try_from(definition).map_err(|_| {
                    crate::error::NetabaseError::Conversion(
                        crate::error::EncodingDecodingError::Decoding(
                            bincode::error::DecodeError::Other("Type conversion failed"),
                        ),
                    )
                })?;

                results.push((key, model));

                // Move to next
                let continue_request = cursor.continue_cursor().map_err(|e| {
                    NetabaseError::Storage(format!("Failed to continue cursor: {:?}", e))
                })?;

                let has_next = continue_request.await.map_err(|e| {
                    NetabaseError::Storage(format!("Cursor continue failed: {:?}", e))
                })?;

                if !has_next {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Insert a secondary key mapping
    async fn insert_secondary_key(
        &self,
        secondary_key: &M::SecondaryKeys,
        primary_key: &M::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        let sec_store_name = format!("{}_secondary", self.store_name);

        let mut composite_key = bincode::encode_to_vec(secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        composite_key.extend_from_slice(&prim_key_bytes);

        let key_js = js_sys::Uint8Array::from(&composite_key[..]);
        let empty_value = js_sys::Uint8Array::new_with_length(0);

        let tx = self
            .db
            .transaction_on_one_with_mode(&sec_store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&sec_store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .put_key_val(&key_js, &empty_value)
            .map_err(|e| NetabaseError::Storage(format!("Failed to put secondary key: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        Ok(())
    }

    /// Remove a secondary key mapping
    async fn remove_secondary_key(
        &self,
        secondary_key: &M::SecondaryKeys,
        primary_key: &M::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        let sec_store_name = format!("{}_secondary", self.store_name);

        let mut composite_key = bincode::encode_to_vec(secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        composite_key.extend_from_slice(&prim_key_bytes);

        let key_js = js_sys::Uint8Array::from(&composite_key[..]);

        let tx = self
            .db
            .transaction_on_one_with_mode(&sec_store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&sec_store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store.delete(&key_js).map_err(|e| {
            NetabaseError::Storage(format!("Failed to delete secondary key: {:?}", e))
        })?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        Ok(())
    }

    /// Find models by secondary key
    pub async fn get_by_secondary_key(
        &self,
        secondary_key: M::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let sec_store_name = format!("{}_secondary", self.store_name);

        let sec_key_bytes = bincode::encode_to_vec(&secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let tx = self.db.transaction_on_one(&sec_store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&sec_store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        // Use cursor to scan for keys with the prefix
        let cursor_request = store
            .open_cursor()
            .map_err(|e| NetabaseError::Storage(format!("Failed to open cursor: {:?}", e)))?;

        let cursor = cursor_request
            .await
            .map_err(|e| NetabaseError::Storage(format!("Cursor request failed: {:?}", e)))?;

        let mut results = Vec::new();

        if let Some(mut cursor) = cursor {
            loop {
                let key_js = cursor.key().ok_or_else(|| {
                    NetabaseError::Storage("Failed to get cursor key".to_string())
                })?;

                let key_array = js_sys::Uint8Array::new(&key_js);
                let mut composite_key = vec![0u8; key_array.length() as usize];
                key_array.copy_to(&mut composite_key);

                // Check if this composite key starts with our secondary key
                if composite_key.starts_with(&sec_key_bytes) {
                    let prim_key_start = sec_key_bytes.len();
                    if composite_key.len() > prim_key_start {
                        let (primary_key, _) = bincode::decode_from_slice::<M::PrimaryKey, _>(
                            &composite_key[prim_key_start..],
                            bincode::config::standard(),
                        )
                        .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

                        if let Some(model) = self.get(primary_key).await? {
                            results.push(model);
                        }
                    }
                }

                let continue_request = cursor.continue_cursor().map_err(|e| {
                    NetabaseError::Storage(format!("Failed to continue cursor: {:?}", e))
                })?;

                let has_next = continue_request.await.map_err(|e| {
                    NetabaseError::Storage(format!("Cursor continue failed: {:?}", e))
                })?;

                if !has_next {
                    break;
                }
            }
        }

        Ok(results)
    }
}
