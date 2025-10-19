use crate::error::NetabaseError;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use std::marker::PhantomData;
use std::path::Path;
use strum::IntoEnumIterator;

/// Type-safe wrapper around sled::Db that works with NetabaseDefinitionTrait
/// types.
///
/// The SledStore provides a type-safe interface to the underlying sled database,
/// using discriminants as tree names and ensuring all operations are type-checked.
pub struct SledStore<D>
where
    D: NetabaseDefinitionTrait,
{
    db: sled::Db,
    _phantom: PhantomData<D>,
}

impl<D> SledStore<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Get direct access to the underlying sled database
    pub fn db(&self) -> &sled::Db {
        &self.db
    }
}

impl<D> SledStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
{
    /// Open a new SledStore at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = sled::open(path)?;
        Ok(Self {
            db,
            _phantom: PhantomData,
        })
    }

    /// Create an in-memory SledStore (useful for testing)
    pub fn temp() -> Result<Self, NetabaseError> {
        let config = sled::Config::new().temporary(true);
        let db = config.open()?;
        Ok(Self {
            db,
            _phantom: PhantomData,
        })
    }

    /// Open a tree for a specific model type
    pub fn open_tree<M>(&self) -> SledStoreTree<D, M>
    where
        M: NetabaseModelTrait + TryFrom<D> + Into<D>,
        D: TryFrom<M> + ToIVec,
    {
        let tree_name = M::discriminant_name();
        SledStoreTree::new(&self.db, tree_name)
    }

    /// Get all tree names (discriminants) in the database
    pub fn tree_names(&self) -> Vec<String> {
        D::Discriminants::iter().map(|d| d.into()).collect()
    }

    // Commenting out iter_all for now as it requires Keys to implement ToIVec
    // which adds complexity. Users can iterate per tree instead.
    // /// Iterate over all records across all trees
    // pub fn iter_all(&self) -> impl Iterator<Item = Result<(D::Keys, D), NetabaseError>> + '_ {
    //     self.db
    //         .tree_names()
    //         .into_iter()
    //         .filter_map(|name| self.db.open_tree(name).ok())
    //         .flat_map(|tree| tree.iter())
    //         .filter_map(|result| {
    //             result.ok().and_then(|(k, v)| {
    //                 let key = D::Keys::from_ivec(&k).ok()?;
    //                 let value = D::from_ivec(&v).ok()?;
    //                 Some(Ok((key, value)))
    //             })
    //         })
    // }

    /// Flush the database to disk
    pub fn flush(&self) -> Result<usize, NetabaseError> {
        Ok(self.db.flush()?)
    }
}

/// Type-safe wrapper around sled::Tree for a specific model type.
///
/// SledStoreTree provides CRUD operations for a single model type with automatic
/// encoding/decoding and secondary key management.
pub struct SledStoreTree<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait,
{
    tree: sled::Tree,
    db: sled::Db,
    _phantom_d: PhantomData<D>,
    _phantom_m: PhantomData<M>,
}

impl<D, M> SledStoreTree<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec,
    M: NetabaseModelTrait + TryFrom<D> + Into<D>,
{
    /// Create a new SledStoreTree
    fn new(db: &sled::Db, tree_name: &str) -> Self {
        let tree = db.open_tree(tree_name).expect("Failed to open tree");
        Self {
            tree,
            db: db.clone(),
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    /// Insert or update a model in the tree
    pub fn put(&self, model: M) -> Result<(), NetabaseError>
    where
        D: From<M>,
    {
        let primary_key = model.primary_key();
        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let definition: D = model.clone().into();
        let value_bytes = definition.to_ivec()?;

        self.tree.insert(key_bytes, value_bytes)?;

        // Handle secondary keys
        let secondary_keys = model.secondary_keys();
        for sec_key in secondary_keys {
            self.insert_secondary_key(&sec_key, &primary_key)?;
        }

        Ok(())
    }

    /// Get a model by its primary key
    pub fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        match self.tree.get(key_bytes)? {
            Some(ivec) => {
                let definition = D::from_ivec(&ivec)?;
                match M::try_from(definition) {
                    Ok(model) => Ok(Some(model)),
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Delete a model by its primary key
    pub fn remove(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        match self.tree.remove(key_bytes)? {
            Some(ivec) => {
                let definition = D::from_ivec(&ivec)?;
                match M::try_from(definition) {
                    Ok(model) => {
                        // Clean up secondary keys
                        let secondary_keys = model.secondary_keys();
                        for sec_key in secondary_keys {
                            self.remove_secondary_key(&sec_key, &key)?;
                        }
                        Ok(Some(model))
                    }
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Iterate over all models in the tree
    pub fn iter(&self) -> SledIter<D, M> {
        SledIter {
            inner: self.tree.iter(),
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    /// Get the number of models in the tree
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Clear all models from the tree
    pub fn clear(&self) -> Result<(), NetabaseError> {
        self.tree.clear()?;
        Ok(())
    }

    /// Insert a secondary key mapping
    fn insert_secondary_key(
        &self,
        secondary_key: &M::SecondaryKeys,
        primary_key: &M::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        // Secondary keys are stored in a separate tree with suffix "_secondary"
        // We use a composite key: secondary_key + primary_key to allow multiple values
        let sec_tree_name = format!("{}_secondary", M::discriminant_name());
        let sec_tree = self.db.open_tree(sec_tree_name)?;

        let mut composite_key = bincode::encode_to_vec(secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        // Append primary key to secondary key to create composite key
        composite_key.extend_from_slice(&prim_key_bytes);

        // Store with empty value (we only need the key)
        sec_tree.insert(composite_key, &[] as &[u8])?;
        Ok(())
    }

    /// Remove a secondary key mapping
    fn remove_secondary_key(
        &self,
        secondary_key: &M::SecondaryKeys,
        primary_key: &M::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        let sec_tree_name = format!("{}_secondary", M::discriminant_name());
        if let Ok(sec_tree) = self.db.open_tree(sec_tree_name) {
            let mut composite_key =
                bincode::encode_to_vec(secondary_key, bincode::config::standard())
                    .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
            let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())
                .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
            composite_key.extend_from_slice(&prim_key_bytes);

            let _: Option<sled::IVec> = sec_tree.remove(composite_key)?;
        }
        Ok(())
    }

    /// Find models by secondary key
    pub fn get_by_secondary_key(
        &self,
        secondary_key: M::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let sec_tree_name = format!("{}_secondary", M::discriminant_name());
        let sec_tree = self.db.open_tree(sec_tree_name)?;

        let sec_key_bytes = bincode::encode_to_vec(&secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let mut results = Vec::new();
        for item in sec_tree.scan_prefix(&sec_key_bytes) {
            let (composite_key, _) = item?;
            // Extract primary key from composite key (skip secondary key bytes)
            let prim_key_start = sec_key_bytes.len();
            if composite_key.len() > prim_key_start {
                let (primary_key, _) = bincode::decode_from_slice::<M::PrimaryKey, _>(
                    &composite_key[prim_key_start..],
                    bincode::config::standard(),
                )
                .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

                if let Some(model) = self.get(primary_key)? {
                    results.push(model);
                }
            }
        }

        Ok(results)
    }
}

/// Iterator over models in a SledStoreTree
pub struct SledIter<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait,
{
    inner: sled::Iter,
    _phantom_d: PhantomData<D>,
    _phantom_m: PhantomData<M>,
}

impl<D, M> Iterator for SledIter<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec,
    M: NetabaseModelTrait + TryFrom<D>,
    M::PrimaryKey: bincode::Decode<()>,
{
    type Item = Result<(M::PrimaryKey, M), NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result.map_err(|e| e.into()).and_then(|(k, v)| {
                let (key, _) =
                    bincode::decode_from_slice::<M::PrimaryKey, _>(&k, bincode::config::standard())
                        .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
                let definition = D::from_ivec(&v)?;
                let model = M::try_from(definition).map_err(|_| {
                    crate::error::NetabaseError::Conversion(
                        crate::error::EncodingDecodingError::Decoding(
                            bincode::error::DecodeError::Other("Type conversion failed"),
                        ),
                    )
                })?;
                Ok((key, model))
            })
        })
    }
}

// Tests are in the tests/ directory
