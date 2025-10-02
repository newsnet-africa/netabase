use bincode::{Decode, Encode};
#[cfg(feature = "libp2p")]
use libp2p::kad::{Record, RecordKey};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::errors::NetabaseError;

pub trait NetabaseModel: Encode + Decode<()> + Sized + Clone + Send + Sync + Debug {
    type Key: NetabaseModelKey;
    type RelationsDiscriminants: strum::IntoEnumIterator + AsRef<str> + Clone + std::hash::Hash + Eq;

    fn key(&self) -> Self::Key;

    /// Return the tree name for this specific model
    fn tree_name() -> &'static str;

    /// Return an iterator of secondary keys for this model
    fn secondary_keys() -> Vec<&'static str> {
        Vec::new()
    }

    /// Return an iterator of relational links for this model
    fn relations() -> Vec<&'static str> {
        Vec::new()
    }

    /// Return discriminant enums for relations
    fn relation_discriminants() -> Vec<Self::RelationsDiscriminants> {
        <Self::RelationsDiscriminants as strum::IntoEnumIterator>::iter().collect()
    }
}

pub trait NetabaseSchema:
    Encode
    + Decode<()>
    + Sized
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Clone
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
{
    type SchemaDiscriminants: strum::IntoEnumIterator + AsRef<str> + Clone + std::hash::Hash + Eq;

    /// Return discriminant enums for schema types
    fn all_schema_discriminants() -> Vec<Self::SchemaDiscriminants> {
        <Self::SchemaDiscriminants as strum::IntoEnumIterator>::iter().collect()
    }

    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }

    // ===== libp2p Methods (available when libp2p feature is enabled) =====
    // These methods are conditionally compiled to maintain trait separation
    // while supporting macro-generated code that expects them on the base trait

    #[cfg(feature = "libp2p")]
    fn to_record(&self) -> Result<libp2p::kad::Record, NetabaseError> {
        use libp2p::kad::{Record, RecordKey};
        Ok(Record {
            key: RecordKey::new(&bincode::encode_to_vec(self, bincode::config::standard())?),
            value: bincode::encode_to_vec(self, bincode::config::standard())?,
            publisher: None,
            expires: None,
        })
    }

    #[cfg(feature = "libp2p")]
    fn from_record(record: libp2p::kad::Record) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.value, bincode::config::standard())?.0)
    }
}

#[cfg(feature = "libp2p")]
pub trait NetabaseSchemaLibp2p: NetabaseSchema + TryInto<Record> + TryFrom<Record> {
    fn to_record(&self) -> Result<Record, NetabaseError>
    where
        <Self as TryInto<libp2p::kad::Record>>::Error: std::marker::Send,
        <Self as TryInto<libp2p::kad::Record>>::Error: std::marker::Sync,
        <Self as TryInto<libp2p::kad::Record>>::Error: 'static,
    {
        Ok(Record {
            key: RecordKey::new(&bincode::encode_to_vec(self, bincode::config::standard())?),
            value: bincode::encode_to_vec(self, bincode::config::standard())?,
            publisher: None,
            expires: None,
        })
    }

    fn from_record(record: Record) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.value, bincode::config::standard())?.0)
    }
}

pub trait NetabaseModelKey:
    Encode + Decode<()> + Clone + Sized + Send + Sync + Debug + 'static
{
    type PrimaryKey: Clone + Send + Sync + Debug + 'static;
    type SecondaryKeys: Clone + Send + Sync + Debug + 'static + TryInto<sled::IVec>;
    type SecondaryKeysDiscriminants: strum::IntoEnumIterator
        + AsRef<str>
        + Clone
        + std::hash::Hash
        + Eq;

    /// Return discriminant enums for secondary keys
    fn secondary_key_discriminants() -> Vec<Self::SecondaryKeysDiscriminants> {
        <Self::SecondaryKeysDiscriminants as strum::IntoEnumIterator>::iter().collect()
    }

    /// Extract and return the primary key from this key if it's a primary key variant
    fn primary_keys(&self) -> Option<&Self::PrimaryKey>;

    /// Extract and return the secondary key from this key if it's a secondary key variant
    fn secondary_keys(&self) -> Option<&Self::SecondaryKeys>;
}

pub trait NetabaseKeys:
    Encode
    + Decode<()>
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Sized
    + Clone
    + std::fmt::Debug
    + Send
    + Sync
{
    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }

    // ===== libp2p Methods (available when libp2p feature is enabled) =====
    // These methods are conditionally compiled to maintain trait separation
    // while supporting macro-generated code that expects them on the base trait

    #[cfg(feature = "libp2p")]
    fn to_record_key(&self) -> Result<libp2p::kad::RecordKey, NetabaseError> {
        use libp2p::kad::RecordKey;
        Ok(RecordKey::new(&bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    #[cfg(feature = "libp2p")]
    fn from_record_key(record: libp2p::kad::RecordKey) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.to_vec(), bincode::config::standard())?.0)
    }
}

#[cfg(feature = "libp2p")]
pub trait NetabaseKeysLibp2p: NetabaseKeys + TryInto<RecordKey> + TryFrom<RecordKey>
where
    libp2p::kad::RecordKey: TryFrom<Self>,
    <libp2p::kad::RecordKey as TryFrom<Self>>::Error: std::marker::Send,
    <libp2p::kad::RecordKey as TryFrom<Self>>::Error: std::marker::Sync,
    <libp2p::kad::RecordKey as TryFrom<Self>>::Error: 'static,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: std::marker::Send,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: std::marker::Sync,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: 'static,
{
    fn to_record_key(&self) -> Result<RecordKey, NetabaseError> {
        Ok(RecordKey::new(&bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_record_key(record: RecordKey) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.to_vec(), bincode::config::standard())?.0)
    }
}

pub trait NetabaseDiscriminants: Into<&'static str> {}

/// Trait for secondary key enums
pub trait NetabaseSecondaryKeys:
    Encode
    + Decode<()>
    + Sized
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Clone
    + Send
    + Sync
    + Debug
    + 'static
{
    fn to_ivec(&self) -> Result<sled::IVec, crate::errors::NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, crate::errors::NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

/// Trait for relational key enums
pub trait NetabaseRelationalKeys:
    Encode
    + Decode<()>
    + Sized
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Clone
    + Send
    + Sync
    + Debug
    + 'static
{
    fn to_ivec(&self) -> Result<sled::IVec, crate::errors::NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, crate::errors::NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

/// Iterator wrapper that automatically converts sled::Iter results to typed (K, V) pairs
pub struct NetabaseIter<K, V> {
    inner: Option<sled::Iter>,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> NetabaseIter<K, V> {
    /// Create a new NetabaseIter from a sled::Iter
    pub fn new(iter: sled::Iter) -> Self {
        Self {
            inner: Some(iter),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create an empty NetabaseIter
    pub fn empty() -> Self {
        Self {
            inner: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Collect all successful results, stopping at the first error
    pub fn collect_results(self) -> Result<Vec<(K, V)>, NetabaseError>
    where
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        self.collect()
    }

    /// Filter and collect only the successful conversions, ignoring errors
    pub fn filter_ok(self) -> impl Iterator<Item = (K, V)>
    where
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        self.filter_map(|result| result.ok())
    }

    /// Get the keys only
    pub fn keys(self) -> impl Iterator<Item = Result<K, NetabaseError>>
    where
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        self.map(|result| result.map(|(k, _v)| k))
    }

    /// Get the values only
    pub fn values(self) -> impl Iterator<Item = Result<V, NetabaseError>>
    where
        K: TryFrom<sled::IVec>,
        V: TryFrom<sled::IVec>,
    {
        self.map(|result| result.map(|(_k, v)| v))
    }
}

impl<K, V> Iterator for NetabaseIter<K, V>
where
    K: TryFrom<sled::IVec>,
    V: TryFrom<sled::IVec>,
{
    type Item = Result<(K, V), crate::errors::NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().map(|result| {
            result
                .map_err(crate::errors::NetabaseError::from)
                .and_then(|(k_ivec, v_ivec)| {
                    let k = K::try_from(k_ivec).map_err(|_| {
                        crate::errors::NetabaseError::Conversion(
                            crate::errors::conversion::ConversionError::TraitConversion,
                        )
                    })?;
                    let v = V::try_from(v_ivec).map_err(|_| {
                        crate::errors::NetabaseError::Conversion(
                            crate::errors::conversion::ConversionError::TraitConversion,
                        )
                    })?;
                    Ok((k, v))
                })
        })
    }
}

/// Iterator specifically for secondary key queries
pub struct SecondaryKeyIter<K, V> {
    inner: Option<sled::Iter>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> SecondaryKeyIter<K, V> {
    pub fn new(iter: sled::Iter) -> Self {
        Self {
            inner: Some(iter),
            _phantom: PhantomData,
        }
    }
}

impl<K, V> Iterator for SecondaryKeyIter<K, V>
where
    K: TryFrom<sled::IVec>,
    V: TryFrom<sled::IVec>,
{
    type Item = Result<(K, V), crate::errors::NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.inner.as_mut()?.next()?;

        match result {
            Ok((k_ivec, v_ivec)) => {
                let k = K::try_from(k_ivec).map_err(|_| {
                    crate::errors::NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                });
                let v = V::try_from(v_ivec).map_err(|_| {
                    crate::errors::NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                });

                match (k, v) {
                    (Ok(key), Ok(value)) => Some(Ok((key, value))),
                    (Err(e), _) | (_, Err(e)) => Some(Err(e)),
                }
            }
            Err(e) => Some(Err(crate::errors::NetabaseError::from(e))),
        }
    }
}

/// Trait for secondary key querying capabilities
pub trait NetabaseSecondaryKeyQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
    /// Query models by a specific secondary key
    fn query_by_secondary_key<SK>(
        &self,
        secondary_key: SK,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        SK: NetabaseSecondaryKeys + TryInto<sled::IVec> + Clone + std::fmt::Debug + PartialEq;

    /// Get all secondary key values for a specific field
    fn get_secondary_key_values(
        &self,
        field_name: &str,
    ) -> Result<Vec<sled::IVec>, crate::errors::NetabaseError>;

    /// Create an index for a secondary key field
    fn create_secondary_key_index(
        &self,
        field_name: &str,
    ) -> Result<(), crate::errors::NetabaseError>;

    /// Remove an index for a secondary key field
    fn remove_secondary_key_index(
        &self,
        field_name: &str,
    ) -> Result<(), crate::errors::NetabaseError>;
}

/// Trait for relational querying capabilities
pub trait NetabaseRelationalQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
    /// Find all models that reference a specific key through relational links
    fn find_referencing_models<TargetKey>(
        &self,
        target_key: TargetKey,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        TargetKey: NetabaseModelKey + PartialEq;

    /// Get all models that have unresolved relational links
    fn get_unresolved_relations(&self) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>;

    /// Resolve relational links in a model using a custom resolver function
    fn resolve_relations<RelatedModel, RelatedKey>(
        &self,
        model: &mut M,
        resolver: impl Fn(
            &crate::relational::RelationalLink<RelatedKey, RelatedModel>,
        ) -> Option<RelatedModel>,
    ) -> Result<(), crate::errors::NetabaseError>
    where
        RelatedModel: Clone + std::fmt::Debug,
        RelatedKey: NetabaseModelKey;

    /// Batch resolve relations for multiple models
    fn batch_resolve_relations<RelatedModel, RelatedKey>(
        &self,
        models: &mut [M],
        resolver: impl Fn(
            &crate::relational::RelationalLink<RelatedKey, RelatedModel>,
        ) -> Option<RelatedModel>,
    ) -> Result<(), crate::errors::NetabaseError>
    where
        RelatedModel: Clone + std::fmt::Debug,
        RelatedKey: NetabaseModelKey;
}

/// Trait for advanced querying capabilities
pub trait NetabaseAdvancedQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
    /// Range query by key prefix
    fn range_by_prefix(&self, prefix: &[u8]) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>;

    /// Batch insert with automatic indexing
    fn batch_insert_with_indexing(
        &self,
        items: Vec<(MK, M)>,
    ) -> Result<(), crate::errors::NetabaseError>;

    /// Query with custom filter function
    fn query_with_filter<F>(&self, filter: F) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>
    where
        F: Fn(&M) -> bool;

    /// Count models matching a condition
    fn count_where<F>(&self, condition: F) -> Result<usize, crate::errors::NetabaseError>
    where
        F: Fn(&M) -> bool;
}

/// Combined trait for all query capabilities
pub trait NetabaseQuery<M, MK>:
    NetabaseSecondaryKeyQuery<M, MK> + NetabaseRelationalQuery<M, MK> + NetabaseAdvancedQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
}

pub mod database_traits {
    #[cfg(feature = "libp2p")]
    use libp2p::kad::{Record, RecordKey};

    use crate::{
        errors::NetabaseError,
        traits::{NetabaseModel, NetabaseModelKey},
    };

    pub trait NetabaseSledDatabase {
        fn new(name: &str) -> Self;
        fn db(&self) -> &sled::Db;
        fn open_tree<K: NetabaseModelKey, V: NetabaseModel, T: NetabaseSledTree<K, V>>(
            &self,
            name: &'static str,
        ) -> Result<T, NetabaseError>
        where
            sled::IVec: std::convert::TryFrom<V>,
        {
            match self.db().open_tree(name) {
                Ok(k) => T::try_from(k).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                }),
                Err(_e) => Err(NetabaseError::Database),
            }
        }
    }
    pub trait NetabaseSledTree<K, V>: TryFrom<sled::Tree>
    where
        sled::IVec: std::convert::TryFrom<V>,
        K: NetabaseModelKey,
        V: NetabaseModel,
    {
        fn tree(&self) -> &sled::Tree;

        fn insert(&self, key: K, value: V) -> Result<Option<V>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            sled::IVec: TryFrom<V>,
            V: TryFrom<sled::IVec>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;
            let value_ivec: sled::IVec = value.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            match self.tree().insert(key_ivec, value_ivec)? {
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

        fn get(&self, key: K) -> Result<Option<V>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            V: TryFrom<sled::IVec>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            match self.tree().get(key_ivec)? {
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

        fn remove(&self, key: K) -> Result<Option<V>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            V: TryFrom<sled::IVec>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            match self.tree().remove(key_ivec)? {
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

        fn contains_key(&self, key: K) -> Result<bool, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            Ok(self.tree().contains_key(key_ivec)?)
        }

        fn update_and_fetch<F>(&self, key: K, f: F) -> Result<Option<V>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            sled::IVec: TryFrom<V>,
            V: TryFrom<sled::IVec>,
            F: Fn(Option<V>) -> Option<V>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            let result = self.tree().update_and_fetch(key_ivec, |old_ivec_opt| {
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

        fn fetch_and_update<F>(&self, key: K, f: F) -> Result<Option<V>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            sled::IVec: TryFrom<V>,
            V: TryFrom<sled::IVec>,
            F: Fn(Option<V>) -> Option<V>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            let result = self.tree().fetch_and_update(key_ivec, |old_ivec_opt| {
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

        fn get_lt(&self, key: K) -> Result<Option<(K, V)>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            K: TryFrom<sled::IVec>,
            V: TryFrom<sled::IVec>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            match self.tree().get_lt(key_ivec)? {
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

        fn get_gt(&self, key: K) -> Result<Option<(K, V)>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            K: TryFrom<sled::IVec>,
            V: TryFrom<sled::IVec>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            match self.tree().get_gt(key_ivec)? {
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

        fn range<R>(&self, range: R) -> crate::traits::NetabaseIter<K, V>
        where
            R: std::ops::RangeBounds<K>,
            sled::IVec: TryFrom<K>,
            K: TryFrom<sled::IVec> + Clone,
            V: TryFrom<sled::IVec>,
        {
            // Convert range bounds to IVec
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

            crate::traits::NetabaseIter::new(self.tree().range((start_bound, end_bound)))
        }

        fn scan_prefix(&self, prefix: K) -> crate::traits::NetabaseIter<K, V>
        where
            sled::IVec: TryFrom<K>,
            K: TryFrom<sled::IVec>,
            V: TryFrom<sled::IVec>,
        {
            match sled::IVec::try_from(prefix) {
                Ok(prefix_ivec) => {
                    crate::traits::NetabaseIter::new(self.tree().scan_prefix(prefix_ivec))
                }
                Err(_) => {
                    // Return empty iterator on conversion failure
                    crate::traits::NetabaseIter::empty()
                }
            }
        }

        /// Iterate over all entries in the tree
        fn iter(&self) -> crate::traits::NetabaseIter<K, V>
        where
            K: TryFrom<sled::IVec>,
            V: TryFrom<sled::IVec>,
        {
            crate::traits::NetabaseIter::new(self.tree().iter())
        }

        fn len(&self) -> usize {
            self.tree().len()
        }

        fn is_empty(&self) -> bool {
            self.tree().is_empty()
        }

        fn clear(&self) -> Result<(), NetabaseError> {
            self.tree().clear()?;
            Ok(())
        }

        fn flush(&self) -> Result<usize, NetabaseError> {
            Ok(self.tree().flush()?)
        }
    }
}
