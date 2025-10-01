//! Sled database implementation for Netabase
//!
//! This module provides concrete implementations of database and tree wrappers
//! for sled, along with iterator types for libp2p's RecordStore trait.

use std::borrow::Cow;
use std::marker::PhantomData;

use libp2p::kad::{ProviderRecord, Record, RecordKey};

use crate::errors::NetabaseError;
use crate::traits::{NetabaseIter, NetabaseModel, NetabaseModelKey};

/// A wrapper around sled::Db that provides typed access to Netabase storage
pub struct NetabaseSledDatabase {
    db: sled::Db,
}

impl NetabaseSledDatabase {
    /// Create a new database with the given name
    pub fn new(name: &str) -> Result<Self, NetabaseError> {
        let db = sled::open(name).map_err(|_| NetabaseError::Database)?;
        Ok(Self { db })
    }

    /// Get a reference to the underlying sled database
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Open a typed tree for storing K,V pairs
    pub fn open_tree<K, V>(
        &self,
        name: &'static str,
    ) -> Result<NetabaseSledTree<K, V>, NetabaseError>
    where
        K: TryFrom<sled::IVec> + Clone,
        V: TryFrom<sled::IVec>,
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
    {
        let tree = self
            .db
            .open_tree(name)
            .map_err(|_| NetabaseError::Database)?;
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData,
        })
    }
}

/// A typed wrapper around sled::Tree that provides automatic conversion
/// between domain types and storage types (IVec)
pub struct NetabaseSledTree<K, V>
where
    K: TryFrom<sled::IVec> + Clone,
    V: TryFrom<sled::IVec>,
    sled::IVec: TryFrom<K>,
    sled::IVec: TryFrom<V>,
{
    tree: sled::Tree,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> NetabaseSledTree<K, V>
where
    K: TryFrom<sled::IVec> + Clone,
    V: TryFrom<sled::IVec>,
    sled::IVec: TryFrom<K>,
    sled::IVec: TryFrom<V>,
{
    /// Get a reference to the underlying sled tree
    pub fn tree(&self) -> &sled::Tree {
        &self.tree
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Result<Option<V>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
        V: TryFrom<sled::IVec>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;
        let value_ivec: sled::IVec = value.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.insert(key_ivec, value_ivec)? {
            Some(old_ivec) => {
                let old_value = V::try_from(old_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some(old_value))
            }
            None => Ok(None),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: K) -> Result<Option<V>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        V: TryFrom<sled::IVec>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.get(key_ivec)? {
            Some(value_ivec) => {
                let value = V::try_from(value_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Remove a key-value pair
    pub fn remove(&self, key: K) -> Result<Option<V>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        V: TryFrom<sled::IVec>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.remove(key_ivec)? {
            Some(value_ivec) => {
                let value = V::try_from(value_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: K) -> Result<bool, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        Ok(self.tree.contains_key(key_ivec)?)
    }

    /// Update and fetch a value atomically
    pub fn update_and_fetch<F>(&self, key: K, f: F) -> Result<Option<V>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
        V: TryFrom<sled::IVec>,
        F: Fn(Option<V>) -> Option<V>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        let result = self.tree.update_and_fetch(key_ivec, |old_ivec_opt| {
            let old_value_opt =
                old_ivec_opt.and_then(|ivec| V::try_from(sled::IVec::from(ivec)).ok());
            let new_value_opt = f(old_value_opt);
            new_value_opt.and_then(|v| {
                sled::IVec::try_from(v)
                    .ok()
                    .map(|ivec| ivec.as_ref().to_vec())
            })
        })?;

        match result {
            Some(value_ivec) => {
                let value = V::try_from(value_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Fetch and update a value atomically
    pub fn fetch_and_update<F>(&self, key: K, f: F) -> Result<Option<V>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
        V: TryFrom<sled::IVec>,
        F: Fn(Option<V>) -> Option<V>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        let result = self.tree.fetch_and_update(key_ivec, |old_ivec_opt| {
            let old_value_opt =
                old_ivec_opt.and_then(|ivec| V::try_from(sled::IVec::from(ivec)).ok());
            let new_value_opt = f(old_value_opt);
            new_value_opt.and_then(|v| {
                sled::IVec::try_from(v)
                    .ok()
                    .map(|ivec| ivec.as_ref().to_vec())
            })
        })?;

        match result {
            Some(value_ivec) => {
                let value = V::try_from(value_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get the first key-value pair less than the given key
    pub fn get_lt(&self, key: K) -> Result<Option<(K, V)>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.get_lt(key_ivec)? {
            Some((k_ivec, v_ivec)) => {
                let k = K::try_from(k_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                let v = V::try_from(v_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some((k, v)))
            }
            None => Ok(None),
        }
    }

    /// Get the first key-value pair greater than the given key
    pub fn get_gt(&self, key: K) -> Result<Option<(K, V)>, NetabaseError>
    where
        sled::IVec: TryFrom<K>,
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.get_gt(key_ivec)? {
            Some((k_ivec, v_ivec)) => {
                let k = K::try_from(k_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                let v = V::try_from(v_ivec).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                })?;
                Ok(Some((k, v)))
            }
            None => Ok(None),
        }
    }

    /// Iterate over a range of key-value pairs
    pub fn range<R>(&self, range: R) -> NetabaseIter<K, V>
    where
        R: std::ops::RangeBounds<K>,
        sled::IVec: TryFrom<K>,
        K: TryFrom<sled::IVec> + Clone,
        V: TryFrom<sled::IVec>,
    {
        use std::ops::Bound;

        let start_bound = match range.start_bound() {
            Bound::Included(k) => {
                if let Ok(ivec) = sled::IVec::try_from(k.clone()) {
                    Bound::Included(ivec)
                } else {
                    Bound::Unbounded
                }
            }
            Bound::Excluded(k) => {
                if let Ok(ivec) = sled::IVec::try_from(k.clone()) {
                    Bound::Excluded(ivec)
                } else {
                    Bound::Unbounded
                }
            }
            Bound::Unbounded => Bound::Unbounded,
        };

        let end_bound = match range.end_bound() {
            Bound::Included(k) => {
                if let Ok(ivec) = sled::IVec::try_from(k.clone()) {
                    Bound::Included(ivec)
                } else {
                    Bound::Unbounded
                }
            }
            Bound::Excluded(k) => {
                if let Ok(ivec) = sled::IVec::try_from(k.clone()) {
                    Bound::Excluded(ivec)
                } else {
                    Bound::Unbounded
                }
            }
            Bound::Unbounded => Bound::Unbounded,
        };

        NetabaseIter::new(self.tree.range((start_bound, end_bound)))
    }

    /// Scan with a key prefix
    pub fn scan_prefix(&self, prefix: K) -> NetabaseIter<K, V>
    where
        sled::IVec: TryFrom<K>,
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        match sled::IVec::try_from(prefix) {
            Ok(prefix_ivec) => NetabaseIter::new(self.tree.scan_prefix(prefix_ivec)),
            Err(_) => NetabaseIter::empty(),
        }
    }

    /// Iterate over all key-value pairs in the tree
    pub fn iter(&self) -> NetabaseIter<K, V>
    where
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        NetabaseIter::new(self.tree.iter())
    }

    /// Get the number of entries in the tree
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Clear all entries from the tree
    pub fn clear(&self) -> Result<(), NetabaseError> {
        self.tree.clear()?;
        Ok(())
    }

    /// Flush the tree to disk
    pub fn flush(&self) -> Result<usize, NetabaseError> {
        Ok(self.tree.flush()?)
    }
}

// Iterator types for libp2p's RecordStore trait
// These wrap NetabaseIter and convert to libp2p types

/// Iterator over records that converts from stored values to libp2p::kad::Record
///
/// This iterator is used by the RecordStore trait's `records()` method.
/// It lazily converts each value to a Record and wraps it in a Cow.
pub struct RecordsIter<'a, K, V>
where
    K: TryFrom<sled::IVec>,
    V: TryFrom<sled::IVec>,
    Record: TryFrom<V>,
{
    inner: NetabaseIter<K, V>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, K, V> RecordsIter<'a, K, V>
where
    K: TryFrom<sled::IVec>,
    V: TryFrom<sled::IVec>,
    Record: TryFrom<V>,
{
    /// Create a new RecordsIter from a NetabaseIter
    pub fn new(iter: NetabaseIter<K, V>) -> Self {
        Self {
            inner: iter,
            _phantom: PhantomData,
        }
    }

    /// Create an empty RecordsIter
    pub fn empty() -> Self {
        Self {
            inner: NetabaseIter::empty(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for RecordsIter<'a, K, V>
where
    K: TryFrom<sled::IVec>,
    V: TryFrom<sled::IVec>,
    Record: TryFrom<V>,
{
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate until we find a valid record or reach the end
        loop {
            match self.inner.next()? {
                Ok((_key, value)) => {
                    // Try to convert the value to a Record
                    if let Ok(record) = Record::try_from(value) {
                        return Some(Cow::Owned(record));
                    }
                    // If conversion fails, skip this entry and continue
                    continue;
                }
                Err(_) => {
                    // Skip errors and continue iterating
                    continue;
                }
            }
        }
    }
}

/// Iterator over provider records
///
/// This iterator is used by the RecordStore trait's `provided()` method.
/// It lazily converts each entry to a ProviderRecord and wraps it in a Cow.
///
/// Note: For the `provided()` method, we're iterating over all records that the local
/// node has advertised as providing. Each entry in the providers tree maps:
///   Key -> RecordKey (the content we're providing)
///   Value -> ProviderRecord (our provider advertisement)
pub struct ProvidedIter<'a, K>
where
    K: TryFrom<sled::IVec>,
{
    inner: NetabaseIter<K, StoredProviderRecord>,
    _phantom: PhantomData<&'a ()>,
}

/// Internal wrapper for storing ProviderRecord in sled
///
/// This is used to store individual provider records that the local node has published.
/// We store a simplified representation that can be serialized with bincode.
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct StoredProviderRecord {
    pub key: Vec<u8>,
    pub provider: Vec<u8>,
    pub expires: Option<u64>, // Store as Unix timestamp in seconds, None if no expiry
    pub addresses: Vec<Vec<u8>>,
}

impl From<ProviderRecord> for StoredProviderRecord {
    fn from(record: ProviderRecord) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let expires = record.expires.and_then(|_instant| {
            // Convert Instant to SystemTime (approximation)
            // Note: This is a best-effort conversion since Instant doesn't have a fixed epoch
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|now| now.as_secs())
        });

        Self {
            key: record.key.as_ref().to_vec(),
            provider: record.provider.to_bytes(),
            expires,
            addresses: record.addresses.iter().map(|addr| addr.to_vec()).collect(),
        }
    }
}

impl TryFrom<StoredProviderRecord> for ProviderRecord {
    type Error = NetabaseError;

    fn try_from(stored: StoredProviderRecord) -> Result<Self, Self::Error> {
        use libp2p::kad::RecordKey;
        use libp2p::{Multiaddr, PeerId};

        let key = RecordKey::from(stored.key);
        let provider = PeerId::from_bytes(&stored.provider).map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        let addresses = stored
            .addresses
            .into_iter()
            .filter_map(|bytes| Multiaddr::try_from(bytes).ok())
            .collect();

        // For now, we don't restore the Instant expiration since we can't reliably
        // convert a Unix timestamp back to an Instant. The RecordStore implementation
        // can handle this based on its own logic.
        let expires = None;

        Ok(ProviderRecord {
            key,
            provider,
            expires,
            addresses,
        })
    }
}

impl TryFrom<sled::IVec> for StoredProviderRecord {
    type Error = NetabaseError;

    fn try_from(ivec: sled::IVec) -> Result<Self, Self::Error> {
        bincode::decode_from_slice(&ivec, bincode::config::standard())
            .map(|(val, _)| val)
            .map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })
    }
}

impl TryFrom<StoredProviderRecord> for sled::IVec {
    type Error = NetabaseError;

    fn try_from(value: StoredProviderRecord) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(&value, bincode::config::standard())
            .map(|vec| sled::IVec::from(vec))
            .map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })
    }
}

impl<'a, K> ProvidedIter<'a, K>
where
    K: TryFrom<sled::IVec>,
{
    /// Create a new ProvidedIter from a NetabaseIter
    pub fn new(iter: NetabaseIter<K, StoredProviderRecord>) -> Self {
        Self {
            inner: iter,
            _phantom: PhantomData,
        }
    }

    /// Create an empty ProvidedIter
    pub fn empty() -> Self {
        Self {
            inner: NetabaseIter::empty(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, K> Iterator for ProvidedIter<'a, K>
where
    K: TryFrom<sled::IVec>,
{
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through the stored provider records
        loop {
            match self.inner.next()? {
                Ok((_key, stored_record)) => {
                    // Try to convert the stored record back to a ProviderRecord
                    if let Ok(provider_record) = ProviderRecord::try_from(stored_record) {
                        return Some(Cow::Owned(provider_record));
                    }
                    // If conversion fails, skip this entry
                    continue;
                }
                Err(_) => {
                    // Skip errors and continue iterating
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let result = NetabaseSledDatabase::new(db_path.to_str().unwrap());
        assert!(result.is_ok());
    }
}
