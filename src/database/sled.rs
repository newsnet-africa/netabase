//! Enhanced Sled database implementation for Netabase with discriminant-based tree management
//!
//! This module provides an enhanced database implementation that supports:
//! - Automatic tree generation from model discriminants
//! - Type-safe tree access using enum discriminants
//! - Secondary key and relation tree management

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::errors::NetabaseError;
use crate::traits::{NetabaseModelKey, NetabaseSchema};

/// Enhanced database that automatically manages trees based on a specific model type
pub struct NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
{
    db: sled::Db,
    main_trees: HashMap<M::SchemaDiscriminants, sled::Tree>,
    secondary_key_trees: HashMap<M::SchemaDiscriminants, sled::Tree>,
    relational_trees: HashMap<M::SchemaDiscriminants, sled::Tree>,
    _phantom: PhantomData<M>,
}

impl<M> NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    /// Create a new enhanced database with default name
    pub fn new() -> Result<Self, NetabaseError> {
        Self::new_with_name("netabase")
    }

    /// Create a new enhanced database with custom name
    pub fn new_with_name(name: &str) -> Result<Self, NetabaseError> {
        let db = sled::open(name).map_err(|_| NetabaseError::Database)?;

        let database = Self {
            db,
            main_trees: HashMap::new(),
            secondary_key_trees: HashMap::new(),
            relational_trees: HashMap::new(),
            _phantom: PhantomData,
        };

        // Don't auto-initialize trees, let user do it manually

        Ok(database)
    }

    /// Initialize trees from model discriminants
    fn initialize_trees(&mut self) -> Result<(), NetabaseError> {
        // Generate main trees from schema discriminants
        for discriminant in M::all_schema_discriminants() {
            let tree_name = format!("schema_{}", discriminant.as_ref());
            let tree = self
                .db
                .open_tree(&tree_name)
                .map_err(|_| NetabaseError::Database)?;
            self.main_trees.insert(discriminant, tree);
        }

        Ok(())
    }

    /// Get a reference to the underlying sled database
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Get a main tree by schema discriminant
    pub fn get_main_tree_by_discriminant(
        &self,
        schema_discriminant: &M::SchemaDiscriminants,
    ) -> Option<&sled::Tree> {
        self.main_trees.get(schema_discriminant)
    }

    /// Open a typed tree using a discriminant
    pub fn open_tree_with_discriminant<K, V>(
        &self,
        discriminant: &M::SchemaDiscriminants,
    ) -> Result<NetabaseSledTree<K, V>, NetabaseError>
    where
        K: TryFrom<sled::IVec> + Clone,
        V: TryFrom<sled::IVec>,
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
    {
        let tree_name = format!("schema_{}", discriminant.as_ref());
        let tree = self
            .db
            .open_tree(&tree_name)
            .map_err(|_| NetabaseError::Database)?;
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(K, V)>,
        })
    }

    /// Get list of all tree names in the database
    pub fn tree_names(&self) -> Vec<String> {
        self.db
            .tree_names()
            .into_iter()
            .map(|name| String::from_utf8_lossy(&name).to_string())
            .collect()
    }

    /// Initialize trees from schema discriminants
    pub fn initialize_trees_from_discriminants(
        &mut self,
        discriminants: &[M::SchemaDiscriminants],
    ) -> Result<(), NetabaseError> {
        for discriminant in discriminants {
            if !self.main_trees.contains_key(discriminant) {
                let tree_name = format!("schema_{}", discriminant.as_ref());
                let tree = self
                    .db
                    .open_tree(&tree_name)
                    .map_err(|_| NetabaseError::Database)?;
                self.main_trees.insert(discriminant.clone(), tree);
            }
        }
        Ok(())
    }

    /// Get main tree by discriminant as typed tree
    pub fn get_main_tree<K, V>(
        &self,
        discriminant: &M::SchemaDiscriminants,
    ) -> Result<NetabaseSledTree<K, V>, NetabaseError>
    where
        K: TryFrom<sled::IVec> + Clone,
        V: TryFrom<sled::IVec>,
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
    {
        let tree = if let Some(tree) = self.main_trees.get(discriminant) {
            tree.clone()
        } else {
            let tree_name = format!("schema_{}", discriminant.as_ref());
            self.db
                .open_tree(&tree_name)
                .map_err(|_| NetabaseError::Database)?
        };
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(K, V)>,
        })
    }

    /// Get secondary tree by discriminant as typed tree
    pub fn get_secondary_tree<K, V>(
        &self,
        discriminant: &M::SchemaDiscriminants,
    ) -> Result<NetabaseSledTree<K, V>, NetabaseError>
    where
        K: TryFrom<sled::IVec> + Clone,
        V: TryFrom<sled::IVec>,
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
    {
        let tree = if let Some(tree) = self.secondary_key_trees.get(discriminant) {
            tree.clone()
        } else {
            let tree_name = format!("secondary_{}", discriminant.as_ref());
            self.db
                .open_tree(&tree_name)
                .map_err(|_| NetabaseError::Database)?
        };
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(K, V)>,
        })
    }

    /// Get relational tree by discriminant as typed tree
    pub fn get_relational_tree<K, V>(
        &self,
        discriminant: &M::SchemaDiscriminants,
    ) -> Result<NetabaseSledTree<K, V>, NetabaseError>
    where
        K: TryFrom<sled::IVec> + Clone,
        V: TryFrom<sled::IVec>,
        sled::IVec: TryFrom<K>,
        sled::IVec: TryFrom<V>,
    {
        let tree = if let Some(tree) = self.relational_trees.get(discriminant) {
            tree.clone()
        } else {
            let tree_name = format!("relation_{}", discriminant.as_ref());
            self.db
                .open_tree(&tree_name)
                .map_err(|_| NetabaseError::Database)?
        };
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(K, V)>,
        })
    }
}

/// Enhanced typed tree wrapper that works with the enhanced database
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
    pub fn insert(&self, key: K, value: V) -> Result<Option<V>, NetabaseError> {
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
    pub fn get(&self, key: K) -> Result<Option<V>, NetabaseError> {
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
    pub fn remove(&self, key: K) -> Result<Option<V>, NetabaseError> {
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
    pub fn contains_key(&self, key: K) -> Result<bool, NetabaseError> {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;
        Ok(self.tree.contains_key(key_ivec)?)
    }

    /// Get the number of items in the tree
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Clear all items from the tree
    pub fn clear(&self) -> Result<(), NetabaseError> {
        self.tree.clear()?;
        Ok(())
    }

    /// Flush pending operations to disk
    pub fn flush(&self) -> Result<(), NetabaseError> {
        self.tree.flush()?;
        Ok(())
    }

    /// Iterate over all key-value pairs
    pub fn iter(&self) -> NetabaseIter<K, V> {
        NetabaseIter {
            inner: Some(self.tree.iter()),
            _phantom: PhantomData,
        }
    }
}

/// Iterator for enhanced tree operations
pub struct NetabaseIter<K, V> {
    inner: Option<sled::Iter>,
    _phantom: PhantomData<(K, V)>,
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
                .map_err(NetabaseError::from)
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

/// Compatibility trait for tree operations
pub trait NetabaseTreeCompatible: Sized {
    /// Convert self to IVec
    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError>;

    /// Convert from IVec
    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError>;
}

impl<T> NetabaseTreeCompatible for T
where
    T: bincode::Encode + bincode::Decode<()>,
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
}
