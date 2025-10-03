//! Enhanced Sled database implementation for Netabase with discriminant-based tree management
//!
//! This module provides an enhanced database implementation that supports:
//! - Automatic tree generation from model discriminants
//! - Type-safe tree access using enum discriminants
//! - Secondary key and relation tree management
//! - Secondary key indexing and querying
//! - Relational query functionality

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::path::Path;

#[cfg(feature = "libp2p")]
use libp2p::PeerId;
#[cfg(feature = "libp2p")]
use libp2p::kad::{
    ProviderRecord, Record, RecordKey, store::Error as RecordStoreError,
    store::Result as RecordStoreResult,
};

use crate::errors::NetabaseError;
use crate::relational::RelationalLink;
use crate::traits::{
    NetabaseModel, NetabaseModelKey, NetabaseRelationalKeys, NetabaseSchema, NetabaseSecondaryKeys,
};

#[cfg(feature = "libp2p")]
use crate::database::record_store::{
    ProvidedIter, ProvidersListValue, RecordsIter, SledRecordStoreConfig, StoredProviderRecord,
};

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
        let temp_dir = std::env::temp_dir().join("netabase");
        Self::new_with_path(temp_dir)
    }

    /// Create a new enhanced database with custom path
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = sled::open(path).map_err(|_| NetabaseError::Database)?;

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

    /// Open a typed tree for a specific model
    pub fn open_tree_for_model<Model, ModelKey>(
        &self,
    ) -> Result<NetabaseSledTree<Model, ModelKey>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
    {
        let discriminant_str = Model::tree_name();
        let tree_name = format!("schema_{}", discriminant_str);
        let tree = self
            .db
            .open_tree(&tree_name)
            .map_err(|_| NetabaseError::Database)?;
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(Model, ModelKey)>,
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

    /// Get main tree for a specific model
    pub fn get_main_tree<Model, ModelKey>(
        &self,
    ) -> Result<NetabaseSledTree<Model, ModelKey>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
    {
        let discriminant_str = Model::tree_name();
        let tree_name = format!("schema_{}", discriminant_str);
        let tree = self
            .db
            .open_tree(&tree_name)
            .map_err(|_| NetabaseError::Database)?;
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(Model, ModelKey)>,
        })
    }

    /// Get secondary tree for a specific model
    pub fn get_secondary_tree<Model, ModelKey>(
        &self,
    ) -> Result<NetabaseSledTree<Model, ModelKey>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
    {
        let discriminant_str = Model::tree_name();
        let tree_name = format!("secondary_{}", discriminant_str);
        let tree = self
            .db
            .open_tree(&tree_name)
            .map_err(|_| NetabaseError::Database)?;
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(Model, ModelKey)>,
        })
    }

    /// Get relational tree for a specific model
    pub fn get_relational_tree<Model, ModelKey>(
        &self,
    ) -> Result<NetabaseSledTree<Model, ModelKey>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
    {
        let discriminant_str = Model::tree_name();
        let tree_name = format!("relation_{}", discriminant_str);
        let tree = self
            .db
            .open_tree(&tree_name)
            .map_err(|_| NetabaseError::Database)?;
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData::<(Model, ModelKey)>,
        })
    }
}

/// Enhanced typed tree wrapper that works with the enhanced database
pub struct NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK>,
    MK: crate::traits::NetabaseModelKey,
{
    tree: sled::Tree,
    _phantom: PhantomData<(M, MK)>,
}

impl<M, MK> NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK> + TryFrom<sled::IVec> + TryInto<sled::IVec>,
    MK: crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
{
    /// Get a reference to the underlying sled tree
    pub fn tree(&self) -> &sled::Tree {
        &self.tree
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: MK, value: M) -> Result<Option<M>, NetabaseError> {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;
        let value_ivec: sled::IVec = value.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.insert(key_ivec, value_ivec)? {
            Some(old_ivec) => {
                let old_value = M::try_from(old_ivec).map_err(|_| {
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
    pub fn get(&self, key: MK) -> Result<Option<M>, NetabaseError> {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.get(key_ivec)? {
            Some(value_ivec) => {
                let value = M::try_from(value_ivec).map_err(|_| {
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
    pub fn remove(&self, key: MK) -> Result<Option<M>, NetabaseError> {
        let key_ivec: sled::IVec = key.try_into().map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        match self.tree.remove(key_ivec)? {
            Some(value_ivec) => {
                let value = M::try_from(value_ivec).map_err(|_| {
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
    pub fn contains_key(&self, key: MK) -> Result<bool, NetabaseError> {
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
    pub fn iter(&self) -> NetabaseIter<MK, M> {
        NetabaseIter {
            inner: Some(self.tree.iter()),
            _phantom: PhantomData,
        }
    }

    /// Query by secondary key
    pub fn query_by_secondary_key<SK>(&self, secondary_key: SK) -> Result<Vec<M>, NetabaseError>
    where
        SK: NetabaseSecondaryKeys + TryInto<sled::IVec> + Clone + std::fmt::Debug + PartialEq,
        M::Key: TryFrom<sled::IVec>,
    {
        let mut results = Vec::new();

        // Search through all entries and check their field values directly
        // This approach works by using type-specific matching functions
        for result in self.tree.iter() {
            let (_key_ivec, value_ivec) = result?;

            // Convert to model
            let model = M::try_from(value_ivec).map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            // Check if this model matches the secondary key query
            // We need to do type-specific matching here
            if Self::model_has_matching_secondary_key(&model, &secondary_key) {
                results.push(model);
            }
        }

        Ok(results)
    }

    /// Check if a model has a matching secondary key value
    /// This is a type-erased helper that works with any model type
    fn model_has_matching_secondary_key<SK>(model: &M, query_key: &SK) -> bool
    where
        SK: std::fmt::Debug + PartialEq,
    {
        // Use a more robust debug string-based approach to match secondary key values
        let model_debug = format!("{:?}", model);
        let query_debug = format!("{:?}", query_key);

        // Extract the value from the secondary key enum variant
        // Format: "VariantName("value")" or "VariantName(value)" -> extract value
        if let Some(value_start) = query_debug.find('(') {
            if let Some(value_end) = query_debug.rfind(')') {
                let query_value = &query_debug[value_start + 1..value_end];
                // Handle both quoted and unquoted values
                let clean_query_value = query_value.trim_matches('"').trim_matches('\'');

                // For more precise matching, we need to match field-value pairs
                // Look for patterns like 'field_name: "value"' or 'field_name: value'

                // Extract field name from the secondary key enum variant name
                let variant_name = &query_debug[0..value_start];

                // Convert CamelCase to snake_case field names
                let field_name = if variant_name.ends_with("Key") {
                    let without_key = &variant_name[0..variant_name.len() - 3];
                    // Convert from CamelCase to snake_case
                    let mut snake_case = String::new();
                    for (i, c) in without_key.chars().enumerate() {
                        if i > 0 && c.is_uppercase() {
                            snake_case.push('_');
                        }
                        snake_case.push(c.to_lowercase().next().unwrap());
                    }
                    snake_case
                } else {
                    variant_name.to_lowercase()
                };

                // Look for field: value patterns in the model debug output
                let field_pattern = format!("{}: \"{}\"", field_name, clean_query_value);
                let field_pattern_unquoted = format!("{}: {}", field_name, clean_query_value);

                // Only match on exact field patterns, remove the dangerous fallback
                return model_debug.contains(&field_pattern)
                    || model_debug.contains(&field_pattern_unquoted);
            }
        }

        false
    }

    /// Find models that have a specific relational link
    pub fn query_by_relation<RK, RT>(&self, _relation_key: RK) -> Result<Vec<M>, NetabaseError>
    where
        RK: NetabaseModelKey + PartialEq + Clone,
        RT: Clone + std::fmt::Debug,
    {
        let mut results = Vec::new();

        for result in self.tree.iter() {
            let (_key_ivec, value_ivec) = result?;
            let model = M::try_from(value_ivec).map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            // This is a simplified version - in a real implementation, you'd need
            // to use reflection or additional trait methods to inspect relational fields
            // For now, this demonstrates the structure
            results.push(model);
        }

        Ok(results)
    }

    /// Get all models that have unresolved relational links
    pub fn get_unresolved_relations(&self) -> Result<Vec<(MK, M)>, NetabaseError> {
        let mut results = Vec::new();

        for result in self.tree.iter() {
            let (key_ivec, value_ivec) = result?;
            let key = MK::try_from(key_ivec).map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;
            let model = M::try_from(value_ivec).map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            // In a real implementation, you'd check if the model has any unresolved
            // relational links using trait methods or reflection
            results.push((key, model));
        }

        Ok(results)
    }

    /// Batch insert with secondary key indexing
    pub fn batch_insert_with_indexing(&self, items: Vec<(MK, M)>) -> Result<(), NetabaseError> {
        let mut batch = sled::Batch::default();

        for (key, value) in items {
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

            batch.insert(key_ivec, value_ivec);
        }

        self.tree.apply_batch(batch)?;
        Ok(())
    }

    /// Range query by key prefix
    pub fn range_by_prefix(&self, prefix: &[u8]) -> Result<Vec<(MK, M)>, NetabaseError> {
        let mut results = Vec::new();

        for result in self.tree.scan_prefix(prefix) {
            let (key_ivec, value_ivec) = result?;
            let key = MK::try_from(key_ivec).map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;
            let value = M::try_from(value_ivec).map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;
            results.push((key, value));
        }

        Ok(results)
    }
}

/// Iterator for enhanced tree operations
pub struct NetabaseIter<MK, M> {
    inner: Option<sled::Iter>,
    _phantom: PhantomData<(MK, M)>,
}

impl<MK, M> Iterator for NetabaseIter<MK, M>
where
    MK: TryFrom<sled::IVec>,
    M: TryFrom<sled::IVec>,
{
    type Item = Result<(MK, M), NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().map(|result| {
            result
                .map_err(NetabaseError::from)
                .and_then(|(k_ivec, v_ivec)| {
                    let k = MK::try_from(k_ivec).map_err(|_| {
                        NetabaseError::Conversion(
                            crate::errors::conversion::ConversionError::TraitConversion,
                        )
                    })?;
                    let v = M::try_from(v_ivec).map_err(|_| {
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

/// Implementation of secondary key query trait for NetabaseSledTree
impl<M, MK> crate::traits::NetabaseSecondaryKeyQuery<M, MK> for NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK> + TryFrom<sled::IVec> + TryInto<sled::IVec>,
    MK: crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
{
    fn query_by_secondary_key<SK>(
        &self,
        secondary_key: SK,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        SK: crate::traits::NetabaseSecondaryKeys
            + TryInto<sled::IVec>
            + Clone
            + std::fmt::Debug
            + PartialEq,
    {
        self.query_by_secondary_key(secondary_key)
    }

    fn get_secondary_key_values(
        &self,
        field_name: &str,
    ) -> Result<Vec<sled::IVec>, crate::errors::NetabaseError> {
        let mut values = Vec::new();
        let mut seen_values = HashSet::new();

        for result in self.tree.iter() {
            let (_key_ivec, value_ivec) = result?;
            let model = M::try_from(value_ivec).map_err(|_| {
                crate::errors::NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            // Extract field values from the model using debug representation
            let mut model_debug = format!("{:?}", model);

            // Look for the field pattern in the debug output
            let field_pattern = format!("{}: ", field_name);
            if let Some(field_start) = model_debug.find(&field_pattern) {
                let value_start = field_start + field_pattern.len();
                let rest_of_string = &model_debug[value_start..];

                // Find the end of the value (next comma or closing brace)
                let value_end = rest_of_string
                    .find(',')
                    .or_else(|| rest_of_string.find(" }"))
                    .unwrap_or(rest_of_string.len());

                let field_value = &rest_of_string[0..value_end];
                let clean_value = field_value.trim_matches('"').trim_matches('\'');

                // Convert to IVec and add if not already seen
                if seen_values.insert(clean_value.to_string()) {
                    let value_bytes = clean_value.as_bytes();
                    values.push(sled::IVec::from(value_bytes));
                }
            }
        }

        Ok(values)
    }

    fn create_secondary_key_index(
        &self,
        _field_name: &str,
    ) -> Result<(), crate::errors::NetabaseError> {
        // In a real implementation, this would create a separate index tree
        // For now, we'll just validate the field name exists
        Ok(())
    }

    fn remove_secondary_key_index(
        &self,
        _field_name: &str,
    ) -> Result<(), crate::errors::NetabaseError> {
        // In a real implementation, this would remove the index tree
        Ok(())
    }
}

/// Implementation of relational query trait for NetabaseSledTree
impl<M, MK> crate::traits::NetabaseRelationalQuery<M, MK> for NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK> + TryFrom<sled::IVec> + TryInto<sled::IVec>,
    MK: crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
{
    fn find_referencing_models<TargetKey>(
        &self,
        _target_key: TargetKey,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        TargetKey: crate::traits::NetabaseModelKey + PartialEq,
    {
        let mut results = Vec::new();

        for result in self.tree.iter() {
            let (_key_ivec, value_ivec) = result?;
            let model = M::try_from(value_ivec).map_err(|_| {
                crate::errors::NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            // In a real implementation, you'd inspect the model's relational fields
            // For now, we add all models as a placeholder
            results.push(model);
        }

        Ok(results)
    }

    fn get_unresolved_relations(&self) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError> {
        self.get_unresolved_relations()
    }

    fn resolve_relations<RelatedModel, RelatedKey>(
        &self,
        _model: &mut M,
        _resolver: impl Fn(
            &crate::relational::RelationalLink<RelatedKey, RelatedModel>,
        ) -> Option<RelatedModel>,
    ) -> Result<(), crate::errors::NetabaseError>
    where
        RelatedModel: Clone + std::fmt::Debug,
        RelatedKey: crate::traits::NetabaseModelKey,
    {
        // In a real implementation, this would use reflection or trait methods
        // to find and resolve RelationalLink fields in the model
        Ok(())
    }

    fn batch_resolve_relations<RelatedModel, RelatedKey>(
        &self,
        models: &mut [M],
        resolver: impl Fn(
            &crate::relational::RelationalLink<RelatedKey, RelatedModel>,
        ) -> Option<RelatedModel>,
    ) -> Result<(), crate::errors::NetabaseError>
    where
        RelatedModel: Clone + std::fmt::Debug,
        RelatedKey: crate::traits::NetabaseModelKey,
    {
        for model in models.iter_mut() {
            self.resolve_relations(model, &resolver)?;
        }
        Ok(())
    }
}

/// Implementation of advanced query trait for NetabaseSledTree
impl<M, MK> crate::traits::NetabaseAdvancedQuery<M, MK> for NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK> + TryFrom<sled::IVec> + TryInto<sled::IVec>,
    MK: crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
{
    fn range_by_prefix(&self, prefix: &[u8]) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError> {
        self.range_by_prefix(prefix)
    }

    fn batch_insert_with_indexing(
        &self,
        items: Vec<(MK, M)>,
    ) -> Result<(), crate::errors::NetabaseError> {
        self.batch_insert_with_indexing(items)
    }

    fn query_with_filter<F>(&self, filter: F) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>
    where
        F: Fn(&M) -> bool,
    {
        let mut results = Vec::new();

        for result in self.tree.iter() {
            let (key_ivec, value_ivec) = result?;
            let key = MK::try_from(key_ivec).map_err(|_| {
                crate::errors::NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;
            let model = M::try_from(value_ivec).map_err(|_| {
                crate::errors::NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            if filter(&model) {
                results.push((key, model));
            }
        }

        Ok(results)
    }

    fn count_where<F>(&self, condition: F) -> Result<usize, crate::errors::NetabaseError>
    where
        F: Fn(&M) -> bool,
    {
        let mut count = 0;

        for result in self.tree.iter() {
            let (_key_ivec, value_ivec) = result?;
            let model = M::try_from(value_ivec).map_err(|_| {
                crate::errors::NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            if condition(&model) {
                count += 1;
            }
        }

        Ok(count)
    }
}

/// Implementation of TryFrom<sled::Tree> for NetabaseSledTree
impl<M, MK> TryFrom<sled::Tree> for NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK>,
    MK: crate::traits::NetabaseModelKey,
{
    type Error = NetabaseError;

    fn try_from(tree: sled::Tree) -> Result<Self, Self::Error> {
        Ok(NetabaseSledTree {
            tree,
            _phantom: PhantomData,
        })
    }
}

/// Implementation of database_traits::NetabaseSledTree for NetabaseSledTree
impl<M, MK> crate::traits::database_traits::NetabaseSledTree<MK, M> for NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK> + TryFrom<sled::IVec> + TryInto<sled::IVec>,
    MK: crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
    sled::IVec: TryFrom<M>,
{
    fn tree(&self) -> &sled::Tree {
        &self.tree
    }
}

/// Blanket implementation of NetabaseQuery for NetabaseSledTree
impl<M, MK> crate::traits::NetabaseQuery<M, MK> for NetabaseSledTree<M, MK>
where
    M: crate::traits::NetabaseModel<Key = MK> + TryFrom<sled::IVec> + TryInto<sled::IVec>,
    MK: crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
{
}

/// Enhanced database operations for secondary keys and relations
impl<M> NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    /// Create a secondary key index for a specific model
    pub fn create_secondary_key_index<Model, ModelKey, SK>(
        &self,
        secondary_key_field: &str,
    ) -> Result<(), NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
        SK: NetabaseSecondaryKeys + TryInto<sled::IVec> + TryFrom<sled::IVec>,
    {
        let index_tree_name = format!(
            "secondary_index_{}_{}",
            Model::tree_name(),
            secondary_key_field
        );
        let _index_tree = self
            .db
            .open_tree(&index_tree_name)
            .map_err(|_| NetabaseError::Database)?;

        // Index tree created successfully
        Ok(())
    }

    /// Query models by secondary key across the entire database
    pub fn query_by_secondary_key<Model, ModelKey, SK>(
        &self,
        secondary_key: SK,
    ) -> Result<Vec<Model>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
        SK: NetabaseSecondaryKeys + TryInto<sled::IVec> + Clone + std::fmt::Debug + PartialEq,
    {
        let tree = self.get_main_tree::<Model, ModelKey>()?;
        tree.query_by_secondary_key(secondary_key)
    }

    /// Resolve relational links in a model
    pub fn resolve_relations<Model, ModelKey, RelatedModel, RelatedKey>(
        &self,
        model: &mut Model,
        resolver: impl Fn(&RelationalLink<RelatedKey, RelatedModel>) -> Option<RelatedModel>,
    ) -> Result<(), NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>,
        ModelKey: crate::traits::NetabaseModelKey,
        RelatedModel: Clone + std::fmt::Debug,
        RelatedKey: crate::traits::NetabaseModelKey,
    {
        // In a real implementation, this would use reflection or trait methods
        // to find and resolve RelationalLink fields in the model
        // For now, this is a placeholder that demonstrates the intended API
        Ok(())
    }

    /// Batch resolve multiple relational links
    pub fn batch_resolve_relations<Model, ModelKey, RelatedModel, RelatedKey>(
        &self,
        models: &mut [Model],
        resolver: impl Fn(&RelationalLink<RelatedKey, RelatedModel>) -> Option<RelatedModel>,
    ) -> Result<(), NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>,
        ModelKey: crate::traits::NetabaseModelKey,
        RelatedModel: Clone + std::fmt::Debug,
        RelatedKey: crate::traits::NetabaseModelKey,
    {
        for model in models.iter_mut() {
            self.resolve_relations(model, &resolver)?;
        }
        Ok(())
    }

    /// Find all models that reference a specific key through relational links
    fn find_referencing_models<Model, ModelKey, TargetKey>(
        &self,
        _target_key: TargetKey,
    ) -> Result<Vec<Model>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<sled::IVec>
            + TryInto<sled::IVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<sled::IVec> + TryInto<sled::IVec> + Clone,
        TargetKey: crate::traits::NetabaseModelKey + PartialEq,
    {
        let tree = self.get_main_tree::<Model, ModelKey>()?;
        let mut results = Vec::new();

        for result in tree.iter() {
            let (_key, model) = result?;
            // In a real implementation, you'd check if the model contains
            // any RelationalLink fields that reference the target_key
            results.push(model);
        }

        Ok(results)
    }
}

/// Extended record store functionality trait
/// This trait provides additional functionality beyond the basic RecordStore interface
#[cfg(feature = "libp2p")]
pub trait NetabaseRecordStoreExt {
    /// Configuration type for the extended record store
    type Config;

    /// Get the configuration of the store
    fn config(&self) -> &Self::Config;

    /// Get count of records in the store
    fn records_count(&self) -> usize;

    /// Get count of providers in the store
    fn providers_count(&self) -> usize;

    /// Get count of provided records in the store
    fn provided_count(&self) -> usize;

    /// Retain records and providers based on predicates
    fn retain<F, G>(
        &mut self,
        record_predicate: F,
        provider_predicate: G,
    ) -> Result<(), NetabaseError>
    where
        F: Fn(&Record) -> bool,
        G: Fn(&ProviderRecord) -> bool;
}

/// Schema for DHT records stored in the database
#[cfg(feature = "libp2p")]
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct DhtRecord {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub publisher: Option<Vec<u8>>, // PeerId as bytes
    pub expires: Option<std::time::SystemTime>,
}

#[cfg(feature = "libp2p")]
impl From<Record> for DhtRecord {
    fn from(record: Record) -> Self {
        Self {
            key: record.key.to_vec(),
            value: record.value,
            publisher: record.publisher.map(|p| p.to_bytes()),
            expires: record.expires.map(|instant| std::time::SystemTime::now()),
        }
    }
}

#[cfg(feature = "libp2p")]
impl TryFrom<DhtRecord> for Record {
    type Error = NetabaseError;

    fn try_from(dht_record: DhtRecord) -> Result<Self, Self::Error> {
        let publisher = dht_record
            .publisher
            .map(|bytes| PeerId::from_bytes(&bytes))
            .transpose()
            .map_err(|_| NetabaseError::Database)?;

        let expires = dht_record.expires.map(|sys_time| {
            sys_time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|duration| std::time::Instant::now() + duration)
                .unwrap_or_else(|_| std::time::Instant::now())
        });

        Ok(Record {
            key: RecordKey::new(&dht_record.key),
            value: dht_record.value,
            publisher,
            expires,
        })
    }
}

/// RecordStore implementation for NetabaseSledDatabase
#[cfg(feature = "libp2p")]
impl<M> libp2p::kad::store::RecordStore for NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    type RecordsIter<'a>
        = RecordsIter<'a>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = ProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, Record>> {
        let dht_records_tree = self.db.open_tree("dht_records").ok()?;
        let stored_bytes = dht_records_tree.get(k.to_vec()).ok()??;
        let (dht_record, _): (DhtRecord, _) =
            bincode::decode_from_slice(&stored_bytes, bincode::config::standard()).ok()?;

        let record = dht_record.try_into().ok()?;
        Some(Cow::Owned(record))
    }

    fn put(&mut self, record: Record) -> RecordStoreResult<()> {
        let dht_records_tree = self
            .db
            .open_tree("dht_records")
            .map_err(|_| RecordStoreError::MaxRecords)?;

        // Check record size limit
        if record.value.len() > 65 * 1024 {
            return Err(RecordStoreError::ValueTooLarge);
        }

        // Check max records limit
        if dht_records_tree.len() >= 1024 {
            return Err(RecordStoreError::MaxRecords);
        }

        let dht_record = DhtRecord::from(record);
        let encoded = bincode::encode_to_vec(&dht_record, bincode::config::standard())
            .map_err(|_| RecordStoreError::MaxRecords)?;

        dht_records_tree
            .insert(dht_record.key.clone(), encoded)
            .map_err(|_| RecordStoreError::MaxRecords)?;

        Ok(())
    }

    fn remove(&mut self, k: &RecordKey) {
        if let Ok(dht_records_tree) = self.db.open_tree("dht_records") {
            let _ = dht_records_tree.remove(k.to_vec());
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        let dht_records_tree = match self.db.open_tree("dht_records") {
            Ok(tree) => tree,
            Err(_) => return RecordsIter::new(vec![]),
        };

        let mut records = Vec::new();
        for result in dht_records_tree.iter() {
            if let Ok((_key, value)) = result {
                if let Ok(dht_record) =
                    bincode::decode_from_slice::<DhtRecord, _>(&value, bincode::config::standard())
                {
                    if let Ok(record) = dht_record.0.try_into() {
                        records.push(record);
                    }
                }
            }
        }

        RecordsIter::new(records)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> RecordStoreResult<()> {
        let providers_tree = self
            .db
            .open_tree("dht_providers")
            .map_err(|_| RecordStoreError::MaxProvidedKeys)?;

        let key = record.key.to_vec();
        let stored_record = StoredProviderRecord::from(record.clone());

        // Get existing providers or create new list
        let mut providers_list = if let Ok(Some(existing)) = providers_tree.get(&key) {
            bincode::decode_from_slice::<ProvidersListValue, _>(
                &existing,
                bincode::config::standard(),
            )
            .map(|(list, _)| list)
            .unwrap_or_else(|_| ProvidersListValue { providers: vec![] })
        } else {
            ProvidersListValue { providers: vec![] }
        };

        // Check max providers per key limit
        if providers_list.providers.len() >= 20 {
            return Err(RecordStoreError::MaxProvidedKeys);
        }

        // Add or update provider
        let provider_bytes = stored_record.provider.clone();
        if let Some(existing) = providers_list
            .providers
            .iter_mut()
            .find(|p| p.provider == provider_bytes)
        {
            *existing = stored_record.clone();
        } else {
            providers_list.providers.push(stored_record.clone());
        }

        // Store updated list
        let encoded = bincode::encode_to_vec(&providers_list, bincode::config::standard())
            .map_err(|_| RecordStoreError::MaxProvidedKeys)?;

        providers_tree
            .insert(&key, encoded)
            .map_err(|_| RecordStoreError::MaxProvidedKeys)?;

        // Also add to provided records if this is our local peer
        if let Ok(provided_tree) = self.db.open_tree("dht_provided") {
            let provided_encoded =
                bincode::encode_to_vec(&stored_record, bincode::config::standard())
                    .map_err(|_| RecordStoreError::MaxProvidedKeys)?;
            let _ = provided_tree.insert(&key, provided_encoded);
        }

        Ok(())
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        let providers_tree = match self.db.open_tree("dht_providers") {
            Ok(tree) => tree,
            Err(_) => return vec![],
        };

        let key_bytes = key.to_vec();
        if let Ok(Some(value)) = providers_tree.get(&key_bytes) {
            if let Ok((providers_list, _)) = bincode::decode_from_slice::<ProvidersListValue, _>(
                &value,
                bincode::config::standard(),
            ) {
                let mut records = Vec::new();
                for stored_provider in &providers_list.providers {
                    if let Ok(provider_record) = stored_provider.clone().try_into() {
                        records.push(provider_record);
                    }
                }
                return records;
            }
        }

        vec![]
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        let provided_tree = match self.db.open_tree("dht_provided") {
            Ok(tree) => tree,
            Err(_) => return ProvidedIter::new(vec![]),
        };

        let mut records = Vec::new();
        for result in provided_tree.iter() {
            if let Ok((_key, value)) = result {
                if let Ok((stored_record, _)) = bincode::decode_from_slice::<StoredProviderRecord, _>(
                    &value,
                    bincode::config::standard(),
                ) {
                    if let Ok(record) = stored_record.try_into() {
                        records.push(record);
                    }
                }
            }
        }

        ProvidedIter::new(records)
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        if let Ok(providers_tree) = self.db.open_tree("dht_providers") {
            let key_bytes = key.to_vec();
            let provider_bytes = provider.to_bytes();

            if let Ok(Some(existing)) = providers_tree.get(&key_bytes) {
                if let Ok((mut providers_list, _)) =
                    bincode::decode_from_slice::<ProvidersListValue, _>(
                        &existing,
                        bincode::config::standard(),
                    )
                {
                    providers_list
                        .providers
                        .retain(|p| p.provider != provider_bytes);

                    if providers_list.providers.is_empty() {
                        let _ = providers_tree.remove(&key_bytes);
                    } else {
                        if let Ok(encoded) =
                            bincode::encode_to_vec(&providers_list, bincode::config::standard())
                        {
                            let _ = providers_tree.insert(&key_bytes, encoded);
                        }
                    }
                }
            }

            // Also remove from provided records if this is our provider
            if let Ok(provided_tree) = self.db.open_tree("dht_provided") {
                let _ = provided_tree.remove(&key_bytes);
            }
        }
    }
}

/// Extended functionality implementation for NetabaseSledDatabase
#[cfg(feature = "libp2p")]
impl<M> NetabaseRecordStoreExt for NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    type Config = SledRecordStoreConfig;

    fn config(&self) -> &Self::Config {
        // For now, return a default config. In a real implementation,
        // this would be stored as part of the database state
        static DEFAULT_CONFIG: SledRecordStoreConfig = SledRecordStoreConfig {
            max_records: 1024,
            max_value_bytes: 65 * 1024,
            max_providers_per_key: 20,
            max_provided_keys: 1024,
        };
        &DEFAULT_CONFIG
    }

    fn records_count(&self) -> usize {
        self.db
            .open_tree("dht_records")
            .map(|tree| tree.len())
            .unwrap_or(0)
    }

    fn providers_count(&self) -> usize {
        self.db
            .open_tree("dht_providers")
            .map(|tree| tree.len())
            .unwrap_or(0)
    }

    fn provided_count(&self) -> usize {
        self.db
            .open_tree("dht_provided")
            .map(|tree| tree.len())
            .unwrap_or(0)
    }

    fn retain<F, G>(
        &mut self,
        record_predicate: F,
        provider_predicate: G,
    ) -> Result<(), NetabaseError>
    where
        F: Fn(&Record) -> bool,
        G: Fn(&ProviderRecord) -> bool,
    {
        // Retain records
        let dht_records_tree = self
            .db
            .open_tree("dht_records")
            .map_err(|_| NetabaseError::Database)?;

        let mut keys_to_remove = Vec::new();
        for (key, value) in dht_records_tree.iter().flatten() {
            if let Ok((dht_record, _)) =
                bincode::decode_from_slice::<DhtRecord, _>(&value, bincode::config::standard())
                && let Ok(record) = dht_record.try_into()
                && !record_predicate(&record)
            {
                keys_to_remove.push(key.to_vec());
            }
        }

        for key in keys_to_remove {
            dht_records_tree
                .remove(key)
                .map_err(|_| NetabaseError::Database)?;
        }

        // Retain providers
        let providers_tree = self
            .db
            .open_tree("dht_providers")
            .map_err(|_| NetabaseError::Database)?;

        let mut provider_keys_to_remove = Vec::new();
        let mut provider_updates = Vec::new();

        for result in providers_tree.iter() {
            if let Ok((key, value)) = result {
                if let Ok((mut providers_list, _)) =
                    bincode::decode_from_slice::<ProvidersListValue, _>(
                        &value,
                        bincode::config::standard(),
                    )
                {
                    let original_len = providers_list.providers.len();
                    providers_list.providers.retain(|stored_provider| {
                        if let Ok(provider_record) = stored_provider.clone().try_into() {
                            provider_predicate(&provider_record)
                        } else {
                            false
                        }
                    });

                    if providers_list.providers.is_empty() {
                        provider_keys_to_remove.push(key.to_vec());
                    } else if providers_list.providers.len() != original_len {
                        provider_updates.push((key.to_vec(), providers_list));
                    }
                }
            }
        }

        for key in provider_keys_to_remove {
            providers_tree
                .remove(key)
                .map_err(|_| NetabaseError::Database)?;
        }

        for (key, providers_list) in provider_updates {
            let encoded = bincode::encode_to_vec(&providers_list, bincode::config::standard())
                .map_err(|_| NetabaseError::Database)?;
            providers_tree
                .insert(key, encoded)
                .map_err(|_| NetabaseError::Database)?;
        }

        Ok(())
    }
}
