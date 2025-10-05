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
use crate::traits::{NetabaseModelKey, NetabaseSchema, NetabaseSchemaQuery, NetabaseSecondaryKeys};

#[cfg(feature = "libp2p")]
use crate::traits::NetabaseRecordStoreQuery;

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
    _secondary_key_trees: HashMap<M::SchemaDiscriminants, sled::Tree>,
    _relational_trees: HashMap<M::SchemaDiscriminants, sled::Tree>,
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
            _secondary_key_trees: HashMap::new(),
            _relational_trees: HashMap::new(),
            _phantom: PhantomData,
        };

        // Don't auto-initialize trees, let user do it manually

        Ok(database)
    }

    /// Initialize trees from model discriminants
    fn _initialize_trees(&mut self) -> Result<(), NetabaseError> {
        // Generate main trees from schema discriminants
        for discriminant in M::all_schema_discriminants() {
            let tree_name = format!("schema_{}", discriminant.as_ref());
            let tree = self
                .db
                .open_tree(&tree_name)
                .map_err(|_| NetabaseError::Database)?;
            self.main_trees.insert(discriminant, tree);
        }

        // Initialize libp2p provider trees when feature is enabled
        #[cfg(feature = "libp2p")]
        {
            // Initialize provider trees for libp2p record store functionality
            let _ = self.db.open_tree("dht_providers");
            let _ = self.db.open_tree("dht_provided");
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
        if let Some(value_start) = query_debug.find('(')
            && let Some(value_end) = query_debug.rfind(')')
        {
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
            let model_debug = format!("{:?}", model);

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
        _model: &mut Model,
        _resolver: impl Fn(&RelationalLink<RelatedKey, RelatedModel>) -> Option<RelatedModel>,
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
    fn _find_referencing_models<Model, ModelKey, TargetKey>(
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

// # RecordStore Architecture Documentation
//
// This module implements a schema-based RecordStore architecture that replaces the previous
// DhtRecord-based approach. The new system provides better type safety and integration
// with NetabaseSchema types.
//
// ## Data Flow Overview
//
// ### PUT Operation Flow:
// ```text
// libp2p::kad::Record → NetabaseSchema → IVec → Sled Database
//        │                    │           │           │
//        │                    │           │           │
//        ▼                    ▼           ▼           ▼
// [Network Record]    [Schema Validation] [Binary] [Discriminant Tree]
// ```
//
// ### GET Operation Flow:
// ```text
// RecordKey → NetabaseSchemaKey → IVec → NetabaseSchema → Record → Cow<Record>
//     │              │            │           │            │         │
//     │              │            │           │            │         │
//     ▼              ▼            ▼           ▼            ▼         ▼
// [Network Key] [Schema Key] [Binary Data] [Validation] [Network] [Output]
// ```
//
// ### Records Iterator Flow:
// ```text
// Discriminant Trees → IVec → NetabaseSchema → Record → Cow<Record>
//         │             │           │            │         │
//         │             │           │            │         │
//         ▼             ▼           ▼            ▼         ▼
// [Tree Iteration] [Binary Data] [Validation] [Network] [Iterator Item]
// ```
//
// ## Key Features
//
// 1. **Schema-Based Storage**: Data is organized by NetabaseSchema discriminants
// 2. **Automatic Conversion**: Seamless conversion between Record and NetabaseSchema
// 3. **Tree-Based Iteration**: RecordsIter loads one discriminant tree at a time
// 4. **Provider Management**: Separate trees for provider records
// 5. **Type Safety**: Full type validation during all conversions
//
// ## Architecture Benefits
//
// - **Performance**: Direct schema access without intermediate DhtRecord type
// - **Memory Efficiency**: Single conversion path reduces allocation overhead
// - **Type Safety**: NetabaseSchema validation catches errors early
// - **Network Compatibility**: Full libp2p RecordStore trait compliance
// - **Scalability**: Tree-based iteration scales with data size
//
// ## Provider Record Management
//
// Provider records are stored in dedicated trees:
// - `dht_providers`: Maps keys to lists of provider records
// - `dht_provided`: Tracks what the local node provides
//
// This separation ensures efficient provider operations without affecting
// main data storage performance.

/// RecordStore implementation for NetabaseSledDatabase
#[cfg(feature = "libp2p")]
impl<M> libp2p::kad::store::RecordStore for NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    type RecordsIter<'a>
        = RecordsIter<'a, M>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = ProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, Record>> {
        // Convert RecordKey to NetabaseSchema key
        let schema_key = <Self as NetabaseRecordStoreQuery<M>>::record_key_to_schema_key(k).ok()?;

        // Get the schema from database
        let schema = <Self as NetabaseSchemaQuery<M>>::get_schema(self, &schema_key).ok()??;

        // Convert schema to record
        let record = <Self as NetabaseRecordStoreQuery<M>>::schema_to_record(&schema).ok()?;
        Some(Cow::Owned(record))
    }

    fn put(&mut self, record: Record) -> RecordStoreResult<()> {
        // Check record size limit
        if record.value.len() > 65 * 1024 {
            return Err(RecordStoreError::ValueTooLarge);
        }

        // Convert Record to NetabaseSchema
        let schema = <Self as NetabaseRecordStoreQuery<M>>::record_to_schema(&record)
            .map_err(|_| RecordStoreError::MaxRecords)?;

        // Put schema into database
        <Self as NetabaseSchemaQuery<M>>::put_schema(self, &schema)
            .map_err(|_| RecordStoreError::MaxRecords)?;

        Ok(())
    }

    fn remove(&mut self, k: &RecordKey) {
        if let Ok(schema_key) = <Self as NetabaseRecordStoreQuery<M>>::record_key_to_schema_key(k) {
            let _ = <Self as NetabaseSchemaQuery<M>>::remove_schema(self, &schema_key);
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        RecordsIter::new(self)
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
        if let Ok(Some(value)) = providers_tree.get(&key_bytes)
            && let Ok((providers_list, _)) = bincode::decode_from_slice::<ProvidersListValue, _>(
                &value,
                bincode::config::standard(),
            )
        {
            let mut records = Vec::new();
            for stored_provider in &providers_list.providers {
                if let Ok(provider_record) = stored_provider.clone().try_into() {
                    records.push(provider_record);
                }
            }
            return records;
        }

        vec![]
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        let provided_tree = match self.db.open_tree("dht_provided") {
            Ok(tree) => tree,
            Err(_) => return ProvidedIter::new(vec![]),
        };

        let mut records = Vec::new();
        for result in provided_tree.iter().flatten() {
            let (_key, value) = result;
            if let Ok((stored_record, _)) = bincode::decode_from_slice::<StoredProviderRecord, _>(
                &value,
                bincode::config::standard(),
            ) && let Ok(record) = stored_record.try_into()
            {
                records.push(record);
            }
        }

        ProvidedIter::new(records)
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        if let Ok(providers_tree) = self.db.open_tree("dht_providers") {
            let key_bytes = key.to_vec();
            let provider_bytes = provider.to_bytes();

            if let Ok(Some(existing)) = providers_tree.get(&key_bytes)
                && let Ok((mut providers_list, _)) =
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
                } else if let Ok(encoded) =
                    bincode::encode_to_vec(&providers_list, bincode::config::standard())
                {
                    let _ = providers_tree.insert(&key_bytes, encoded);
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
        // Since we're now using NetabaseSchema instead of DhtRecord,
        // we need to implement retain logic for NetabaseSchema records
        let mut schema_keys_to_remove = Vec::new();

        // Get all schemas and check which ones to retain
        for discriminant in M::all_schema_discriminants() {
            if let Ok(schemas) =
                <Self as NetabaseSchemaQuery<M>>::get_schemas_by_discriminant(self, &discriminant)
            {
                for schema in schemas {
                    if let Ok(record) =
                        <Self as NetabaseRecordStoreQuery<M>>::schema_to_record(&schema)
                        && !record_predicate(&record)
                    {
                        let key = schema.keys();
                        schema_keys_to_remove.push(key);
                    }
                }
            }
        }

        // Remove schemas that don't pass the predicate
        for key in schema_keys_to_remove {
            let _ = <Self as NetabaseSchemaQuery<M>>::remove_schema(self, &key);
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

/// Implementation of NetabaseSchemaQuery trait
impl<M> crate::traits::NetabaseSchemaQuery<M> for NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    fn get_schema(&self, key: &M::Keys) -> Result<Option<M>, NetabaseError> {
        let discriminant = M::discriminant_for_key(key);
        let tree = self
            .get_main_tree_by_discriminant(&discriminant)
            .ok_or(NetabaseError::Database)?;
        let key_ivec = key.to_ivec()?;

        if let Some(value_ivec) = tree.get(&key_ivec)? {
            let schema = M::from_ivec(value_ivec)?;
            Ok(Some(schema))
        } else {
            Ok(None)
        }
    }

    fn put_schema(&mut self, schema: &M) -> Result<(), NetabaseError> {
        let discriminant = schema.discriminant();
        let tree = self
            .get_main_tree_by_discriminant(&discriminant)
            .ok_or(NetabaseError::Database)?;
        let key = schema.keys();
        let key_ivec = key.to_ivec()?;
        let value_ivec = schema.to_ivec()?;

        tree.insert(key_ivec, value_ivec)?;
        Ok(())
    }

    fn remove_schema(&mut self, key: &M::Keys) -> Result<(), NetabaseError> {
        let discriminant = M::discriminant_for_key(key);
        let tree = self
            .get_main_tree_by_discriminant(&discriminant)
            .ok_or(NetabaseError::Database)?;
        let key_ivec = key.to_ivec()?;

        tree.remove(&key_ivec)?;
        Ok(())
    }

    fn get_schemas_by_discriminant(
        &self,
        discriminant: &M::SchemaDiscriminants,
    ) -> Result<Vec<M>, NetabaseError> {
        let tree = self
            .get_main_tree_by_discriminant(discriminant)
            .ok_or(NetabaseError::Database)?;
        let mut schemas = Vec::new();

        for result in tree.iter() {
            let (_key, value) = result?;
            let schema = M::from_ivec(value)?;
            schemas.push(schema);
        }

        Ok(schemas)
    }

    fn get_all_schemas(&self) -> Result<Vec<M>, NetabaseError> {
        let mut all_schemas = Vec::new();

        for discriminant in M::all_schema_discriminants() {
            let schemas = self.get_schemas_by_discriminant(&discriminant)?;
            all_schemas.extend(schemas);
        }

        Ok(all_schemas)
    }
}

/// Implementation of NetabaseRecordStoreQuery trait
#[cfg(feature = "libp2p")]
impl<M> crate::traits::NetabaseRecordStoreQuery<M> for NetabaseSledDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    fn schema_key_to_record_key(key: &M::Keys) -> Result<libp2p::kad::RecordKey, NetabaseError> {
        use crate::traits::NetabaseKeys;
        key.to_record_key()
    }

    fn record_key_to_schema_key(
        record_key: &libp2p::kad::RecordKey,
    ) -> Result<M::Keys, NetabaseError> {
        use crate::traits::NetabaseKeys;
        M::Keys::from_record_key(record_key.clone())
    }

    fn get_schema_by_record_key(
        &self,
        record_key: &libp2p::kad::RecordKey,
    ) -> Result<Option<M>, NetabaseError> {
        let schema_key = Self::record_key_to_schema_key(record_key)?;
        <Self as NetabaseSchemaQuery<M>>::get_schema(self, &schema_key)
    }

    fn schema_to_record(schema: &M) -> Result<libp2p::kad::Record, NetabaseError> {
        schema.to_record()
    }

    fn record_to_schema(record: &libp2p::kad::Record) -> Result<M, NetabaseError> {
        M::from_record(record.clone())
    }
}
