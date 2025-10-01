use bincode::{Decode, Encode};
use libp2p::kad::{Record, RecordKey};

use crate::errors::NetabaseError;

pub trait NetabaseModel: Encode + Decode<()> + Sized + Clone + Send + Sync {
    type Key: NetabaseModelKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchema:
    Encode
    + Decode<()>
    + Sized
    + TryInto<Record>
    + TryFrom<Record>
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Clone
    + Send
    + Sync
    + 'static
where
    Self: TryFrom<sled::IVec>,
    sled::IVec: TryFrom<Self>,
{
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

    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

pub trait NetabaseModelKey: Encode + Decode<()> + Clone + Sized + Send + Sync + 'static {}

pub trait NetabaseKeys:
    Encode
    + Decode<()>
    + Sized
    + TryInto<RecordKey>
    + TryFrom<RecordKey>
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Clone
    + Send
    + Sync
    + 'static
where
    libp2p::kad::RecordKey: TryFrom<Self>,
    sled::IVec: TryFrom<Self>,
    Self: TryFrom<sled::IVec>,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: std::error::Error,
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

    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

pub trait NetabaseDiscriminants: Into<&'static str> {}

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
    type Item = Result<(K, V), NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().map(|result| {
            result
                .map_err(|e| NetabaseError::from(e))
                .and_then(|(k_ivec, v_ivec)| {
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
                    Ok((k, v))
                })
        })
    }
}

pub mod database_traits {
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
            libp2p::kad::RecordKey: std::convert::TryFrom<K>,
            libp2p::kad::Record: std::convert::TryFrom<V>,
            libp2p::kad::RecordKey: std::convert::TryFrom<<V as NetabaseModel>::Key>,
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
        RecordKey: TryFrom<K>,
        Record: TryFrom<V>,
        RecordKey: TryFrom<<V as NetabaseModel>::Key>,
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
