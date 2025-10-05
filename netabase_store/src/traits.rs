//! # Netabase Traits
//!
//! This module contains the core traits that define the behavior of Netabase models,
//! schemas, keys, and database operations. These traits are typically implemented
//! automatically by the derive macros, but can also be implemented manually for
//! custom behavior.
//!
//! ## Key Traits
//!
//! - [`NetabaseModel`] - Core trait for all database models
//! - [`NetabaseSchema`] - Trait for schema enums containing multiple models
//! - [`NetabaseModelKey`] - Trait for model key types
//! - [`NetabaseSecondaryKeyQuery`] - Secondary key querying capabilities
//! - [`NetabaseAdvancedQuery`] - Advanced querying operations
//!
//! ## Usage
//!
//! Most users will interact with these traits through the generated implementations
//! from the derive macros:
//!
//! ```rust
//! use netabase_macros::{NetabaseModel, netabase_schema_module};
//! use netabase_store::traits::NetabaseModel;
//!
//! #[derive(NetabaseModel)]
//! #[key_name(UserKey)]
//! struct User {
//!     #[key] id: u64,
//!     name: String,
//! }
//!
//! let user = User { id: 1, name: "Alice".into() };
//! let key = user.key(); // Uses NetabaseModel trait
//! ```

use bincode::{Decode, Encode};
#[cfg(feature = "libp2p")]
use libp2p::kad::{Record, RecordKey};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::errors::NetabaseError;

// Conditional imports for database backend
#[cfg(feature = "native")]
use sled;

// Conditional type aliases for database backend - moved to top to fix compilation
#[cfg(feature = "native")]
pub type DatabaseIVec = sled::IVec;

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub type DatabaseIVec = crate::database::memory::MemoryIVec;

/// Core trait for all Netabase database models.
///
/// This trait defines the fundamental operations and properties that every model
/// must support. It's typically implemented automatically via the `NetabaseModel`
/// derive macro.
///
/// ## Associated Types
///
/// - `Key` - The key type used to identify and query this model
/// - `RelationsDiscriminants` - Enum representing different types of relations
///
/// ## Required Methods
///
/// - [`key()`](NetabaseModel::key) - Extract the primary key from a model instance
/// - [`tree_name()`](NetabaseModel::tree_name) - Get the storage tree name for this model type
///
/// ## Generated Implementation
///
/// When using the derive macro, implementations are generated automatically:
///
/// ```rust
/// use netabase_macros::NetabaseModel;
///
/// #[derive(NetabaseModel)]
/// #[key_name(UserKey)]
/// struct User {
///     #[key] id: u64,
///     name: String,
///     #[secondary_key] email: String,
/// }
///
/// // Generated implementation provides:
/// let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into() };
/// let key = user.key(); // Returns UserKey::Primary(UserPrimaryKey(1))
/// let tree_name = User::tree_name(); // Returns "User"
/// let secondary_keys = User::secondary_keys(); // Returns ["email"]
/// ```
///
/// ## Manual Implementation
///
/// For custom behavior, you can implement this trait manually:
///
/// ```rust
/// use netabase_store::traits::NetabaseModel;
///
/// struct CustomModel {
///     id: String,
///     data: Vec<u8>,
/// }
///
/// #[derive(Clone, Debug)]
/// enum CustomKey {
///     Primary(String),
/// }
///
/// impl NetabaseModel for CustomModel {
///     type Key = CustomKey;
///     type RelationsDiscriminants = (); // No relations
///
///     fn key(&self) -> Self::Key {
///         CustomKey::Primary(self.id.clone())
///     }
///
///     fn tree_name() -> &'static str {
///         "CustomModel"
///     }
/// }
/// ```
pub trait NetabaseModel: Encode + Decode<()> + Sized + Clone + Send + Sync + Debug {
    /// The key type used to identify and query instances of this model.
    ///
    /// This is typically an enum with `Primary` and `Secondary` variants,
    /// generated automatically by the derive macro.
    type Key: NetabaseModelKey;

    /// Enum representing the different types of relations this model can have.
    ///
    /// Used for type-safe relational queries and foreign key relationships.
    type RelationsDiscriminants: strum::IntoEnumIterator + AsRef<str> + Clone + std::hash::Hash + Eq;

    /// Extract the primary key from this model instance.
    ///
    /// Returns the key that uniquely identifies this model in the database.
    ///
    /// # Example
    ///
    /// ```rust
    /// let user = User { id: 1, name: "Alice".into() };
    /// let key = user.key(); // Returns UserKey::Primary(UserPrimaryKey(1))
    /// ```
    fn key(&self) -> Self::Key;

    /// Return the tree name for this specific model type.
    ///
    /// This determines which storage tree the model's data will be stored in.
    /// By default, this returns the struct name as a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// assert_eq!(User::tree_name(), "User");
    /// ```
    fn tree_name() -> &'static str;

    /// Return the names of all secondary key fields for this model.
    ///
    /// Secondary keys are fields marked with `#[secondary_key]` that can be
    /// used for efficient querying without knowing the primary key.
    ///
    /// # Example
    ///
    /// ```rust
    /// // For a User model with #[secondary_key] on email and department
    /// assert_eq!(User::secondary_keys(), vec!["email", "department"]);
    /// ```
    fn secondary_keys() -> Vec<&'static str> {
        Vec::new()
    }

    /// Return the names of all relational fields for this model.
    ///
    /// These are fields that reference other models, enabling foreign key
    /// relationships and join-like operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// // For a Post model with relations to User and Category
    /// assert_eq!(Post::relations(), vec!["author", "category"]);
    /// ```
    fn relations() -> Vec<&'static str> {
        Vec::new()
    }

    /// Return discriminant enums for all relation types.
    ///
    /// This provides type-safe access to the different types of relations
    /// this model can participate in.
    fn relation_discriminants() -> Vec<Self::RelationsDiscriminants> {
        <Self::RelationsDiscriminants as strum::IntoEnumIterator>::iter().collect()
    }
}

/// Trait for schema enums that contain multiple model types.
///
/// A schema represents a collection of related models organized into a single
/// namespace. This trait is implemented automatically by the `netabase_schema_module`
/// attribute macro.
///
/// ## Purpose
///
/// Schemas serve several important purposes:
/// - **Type Unification**: Combine multiple model types into a single enum
/// - **Network Serialization**: Enable sending any model type over the network
/// - **Database Organization**: Group related models together logically
/// - **Query Interface**: Provide unified access to all model types
///
/// ## Generated Implementation
///
/// The `netabase_schema_module` macro generates schema enums automatically:
///
/// ```rust
/// use netabase_macros::{NetabaseModel, netabase_schema_module};
///
/// #[netabase_schema_module(BlogSchema, BlogKeys)]
/// mod blog {
///     #[derive(NetabaseModel)]
///     #[key_name(UserKey)]
///     pub struct User { #[key] id: u64, name: String }
///
///     #[derive(NetabaseModel)]
///     #[key_name(PostKey)]
///     pub struct Post { #[key] id: u64, title: String }
/// }
///
/// // Generated BlogSchema enum:
/// // enum BlogSchema {
/// //     User(User),
/// //     Post(Post),
/// // }
///
/// // Usage:
/// let user = User { id: 1, name: "Alice".into() };
/// let schema_item = BlogSchema::User(user); // Automatic conversion
/// ```
///
/// ## Network Integration
///
/// Schemas enable distributed operations across model types:
///
/// ```rust
/// use netabase::Netabase;
///
/// let mut netabase = Netabase::<BlogSchema>::new()?;
///
/// // Can store any model type in the schema
/// netabase.put_record(user).await?;
/// netabase.put_record(post).await?;
/// ```
///
/// ## Type Safety
///
/// Schemas maintain type safety while providing flexibility:
///
/// ```rust
/// match schema_item {
///     BlogSchema::User(user) => handle_user(user),
///     BlogSchema::Post(post) => handle_post(post),
/// }
/// ```
pub trait NetabaseSchema:
    Encode
    + Decode<()>
    + Sized
    + TryInto<DatabaseIVec>
    + TryFrom<DatabaseIVec>
    + Clone
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
{
    /// Enum representing all model variants in this schema.
    ///
    /// Used for type-safe iteration and discrimination between model types.
    type SchemaDiscriminants: strum::IntoEnumIterator
        + AsRef<str>
        + Clone
        + std::hash::Hash
        + Eq
        + Send
        + Sync;
    type Keys: NetabaseKeys + Encode + Decode<()>; //TODO: some old refactor did something weird that rewuires bincode to be explicit here

    fn keys(&self) -> Self::Keys;

    /// Return discriminant enums for schema types
    fn all_schema_discriminants() -> Vec<Self::SchemaDiscriminants> {
        <Self::SchemaDiscriminants as strum::IntoEnumIterator>::iter().collect()
    }

    /// Get the discriminant for this schema instance
    fn discriminant(&self) -> Self::SchemaDiscriminants;

    /// Get the discriminant that matches a given key
    fn discriminant_for_key(key: &Self::Keys) -> Self::SchemaDiscriminants;

    /// Convert NetabaseSchemaKeys to matching ModelDiscriminants
    fn key_to_discriminant(key: &Self::Keys) -> Self::SchemaDiscriminants {
        Self::discriminant_for_key(key)
    }

    fn to_ivec(&self) -> Result<DatabaseIVec, NetabaseError> {
        Ok(DatabaseIVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: DatabaseIVec) -> Result<Self, NetabaseError> {
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
    #[cfg(feature = "native")]
    type SecondaryKeys: Clone + Send + Sync + Debug + 'static + TryInto<sled::IVec>;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    type SecondaryKeys: Clone + Send + Sync + Debug + 'static + TryInto<DatabaseIVec>;
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
    + TryInto<DatabaseIVec>
    + TryFrom<DatabaseIVec>
    + Sized
    + Clone
    + std::fmt::Debug
    + Send
    + Sync
{
    fn to_ivec(&self) -> Result<DatabaseIVec, NetabaseError> {
        Ok(DatabaseIVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: DatabaseIVec) -> Result<Self, NetabaseError> {
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

/// Trait for schema-based querying in NetabaseSledDatabase
pub trait NetabaseSchemaQuery<M: NetabaseSchema> {
    /// Get a NetabaseSchema by key, converting to the appropriate discriminant
    fn get_schema(&self, key: &M::Keys) -> Result<Option<M>, NetabaseError>;

    /// Put a NetabaseSchema, storing it in the correct tree based on discriminant
    fn put_schema(&mut self, schema: &M) -> Result<(), NetabaseError>;

    /// Remove a NetabaseSchema by key
    fn remove_schema(&mut self, key: &M::Keys) -> Result<(), NetabaseError>;

    /// Get all schemas of a specific discriminant type
    fn get_schemas_by_discriminant(
        &self,
        discriminant: &M::SchemaDiscriminants,
    ) -> Result<Vec<M>, NetabaseError>;

    /// Get all schemas across all discriminants
    fn get_all_schemas(&self) -> Result<Vec<M>, NetabaseError>;
}

/// Trait for Record Store operations using NetabaseSchema
#[cfg(feature = "libp2p")]
pub trait NetabaseRecordStoreQuery<M: NetabaseSchema> {
    /// Convert NetabaseSchema key to Record key
    fn schema_key_to_record_key(key: &M::Keys) -> Result<libp2p::kad::RecordKey, NetabaseError>;

    /// Convert Record key to NetabaseSchema key
    fn record_key_to_schema_key(
        record_key: &libp2p::kad::RecordKey,
    ) -> Result<M::Keys, NetabaseError>;

    /// Get NetabaseSchema by Record key
    fn get_schema_by_record_key(
        &self,
        record_key: &libp2p::kad::RecordKey,
    ) -> Result<Option<M>, NetabaseError>;

    /// Convert NetabaseSchema to Record
    fn schema_to_record(schema: &M) -> Result<libp2p::kad::Record, NetabaseError>;

    /// Convert Record to NetabaseSchema
    fn record_to_schema(record: &libp2p::kad::Record) -> Result<M, NetabaseError>;
}

pub trait NetabaseDiscriminants: Into<&'static str> {}

/// Trait for secondary key enums
#[cfg(feature = "native")]
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
    #[cfg(feature = "native")]
    fn to_ivec(&self) -> Result<sled::IVec, crate::errors::NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    #[cfg(feature = "native")]
    fn from_ivec(ivec: sled::IVec) -> Result<Self, crate::errors::NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub trait NetabaseSecondaryKeys:
    Encode
    + Decode<()>
    + Sized
    + TryInto<DatabaseIVec>
    + TryFrom<DatabaseIVec>
    + Clone
    + Send
    + Sync
    + Debug
    + 'static
{
    fn to_ivec(&self) -> Result<DatabaseIVec, crate::errors::NetabaseError> {
        Ok(DatabaseIVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: DatabaseIVec) -> Result<Self, crate::errors::NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

/// Trait for relational key enums
#[cfg(feature = "native")]
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
    #[cfg(feature = "native")]
    fn to_ivec(&self) -> Result<sled::IVec, crate::errors::NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    #[cfg(feature = "native")]
    fn from_ivec(ivec: sled::IVec) -> Result<Self, crate::errors::NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub trait NetabaseRelationalKeys:
    Encode
    + Decode<()>
    + Sized
    + TryInto<DatabaseIVec>
    + TryFrom<DatabaseIVec>
    + Clone
    + Send
    + Sync
    + Debug
    + 'static
{
    fn to_ivec(&self) -> Result<DatabaseIVec, crate::errors::NetabaseError> {
        Ok(DatabaseIVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: DatabaseIVec) -> Result<Self, crate::errors::NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

/// Iterator wrapper that automatically converts database results to typed (K, V) pairs
#[cfg(feature = "native")]
pub struct NetabaseIter<K, V> {
    inner: Option<sled::Iter>,
    _phantom: PhantomData<(K, V)>,
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub struct NetabaseIter<K, V> {
    inner: Option<crate::database::memory::MemoryIter>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> NetabaseIter<K, V> {
    #[cfg(feature = "native")]
    pub fn new(iter: sled::Iter) -> Self {
        Self {
            inner: Some(iter),
            _phantom: PhantomData,
        }
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub fn new(iter: crate::database::memory::MemoryIter) -> Self {
        Self {
            inner: Some(iter),
            _phantom: PhantomData,
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
        K: TryFrom<DatabaseIVec>,
        V: TryFrom<DatabaseIVec>,
    {
        self.collect()
    }

    /// Filter and collect only the successful conversions, ignoring errors
    pub fn filter_ok(self) -> impl Iterator<Item = (K, V)>
    where
        K: TryFrom<DatabaseIVec>,
        V: TryFrom<DatabaseIVec>,
    {
        self.filter_map(|result| result.ok())
    }

    /// Get the keys only
    pub fn keys(self) -> impl Iterator<Item = Result<K, NetabaseError>>
    where
        K: TryFrom<DatabaseIVec>,
        V: TryFrom<DatabaseIVec>,
    {
        self.map(|result| result.map(|(k, _v)| k))
    }

    /// Get the values only
    pub fn values(self) -> impl Iterator<Item = Result<V, NetabaseError>>
    where
        K: TryFrom<DatabaseIVec>,
        V: TryFrom<DatabaseIVec>,
    {
        self.map(|result| result.map(|(_k, v)| v))
    }
}

impl<K, V> Iterator for NetabaseIter<K, V>
where
    K: TryFrom<DatabaseIVec>,
    V: TryFrom<DatabaseIVec>,
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

#[cfg(all(feature = "wasm", not(feature = "native")))]
impl<K, V> SecondaryKeyIter<K, V> {
    pub fn new(iter: crate::database::memory::MemoryIter) -> Self {
        Self {
            inner: Some(iter),
            _phantom: PhantomData,
        }
    }
}

/// Iterator specifically for secondary key queries
/// Iterator for secondary key queries
#[cfg(feature = "native")]
pub struct SecondaryKeyIter<K, V> {
    inner: Option<sled::Iter>,
    _phantom: PhantomData<(K, V)>,
}

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub struct SecondaryKeyIter<K, V> {
    inner: Option<crate::database::memory::MemoryIter>,
    _phantom: PhantomData<(K, V)>,
}

#[cfg(feature = "native")]
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
    K: TryFrom<DatabaseIVec>,
    V: TryFrom<DatabaseIVec>,
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

/// Trait for secondary key querying capabilities.
///
/// This trait provides efficient querying of models using secondary keys (fields marked
/// with `#[secondary_key]`). Secondary keys enable fast lookups without knowing the primary
/// key, supporting common query patterns like "find all users by email" or "get all posts
/// by author".
///
/// ## Query Performance
///
/// - Secondary key queries are O(m) where m is the number of matching records
/// - Much faster than scanning all records for most use cases
/// - Indexes are automatically maintained during insert/update/delete operations
///
/// ## Usage Examples
///
/// ```rust
/// use netabase_store::traits::NetabaseSecondaryKeyQuery;
///
/// // Query users by email
/// let users = user_tree.query_by_secondary_key(
///     UserSecondaryKeys::EmailKey("alice@example.com".to_string())
/// )?;
///
/// // Query posts by category
/// let tech_posts = post_tree.query_by_secondary_key(
///     PostSecondaryKeys::CategoryKey("Technology".to_string())
/// )?;
///
/// // Query by boolean secondary key
/// let published_posts = post_tree.query_by_secondary_key(
///     PostSecondaryKeys::PublishedKey(true)
/// )?;
/// ```
///
/// ## Index Management
///
/// Secondary key indexes are typically managed automatically, but can be controlled manually:
///
/// ```rust
/// // Create index for better query performance
/// tree.create_secondary_key_index("email")?;
///
/// // Remove index to save space (queries will be slower)
/// tree.remove_secondary_key_index("email")?;
/// ```
pub trait NetabaseSecondaryKeyQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
    /// Query models by a specific secondary key value.
    ///
    /// This method efficiently finds all models where a secondary key field matches
    /// the specified value. It returns a vector of matching models.
    ///
    /// # Arguments
    ///
    /// * `secondary_key` - The secondary key variant with the value to search for
    ///
    /// # Returns
    ///
    /// A vector of all models that match the secondary key criteria.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Find all users in the Engineering department
    /// let engineers = user_tree.query_by_secondary_key(
    ///     UserSecondaryKeys::DepartmentKey("Engineering".to_string())
    /// )?;
    ///
    /// for user in engineers {
    ///     println!("Engineer: {}", user.name);
    /// }
    /// ```
    #[cfg(feature = "native")]
    fn query_by_secondary_key<SK>(
        &self,
        secondary_key: SK,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        SK: NetabaseSecondaryKeys + TryInto<sled::IVec> + Clone + std::fmt::Debug + PartialEq;

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn query_by_secondary_key<SK>(
        &self,
        secondary_key: SK,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        SK: NetabaseSecondaryKeys + TryInto<DatabaseIVec> + Clone + std::fmt::Debug + PartialEq;

    /// Get all unique values for a specific secondary key field.
    ///
    /// This method returns all distinct values that exist for a secondary key field
    /// across all records. Useful for building filters, dropdowns, or analytics.
    ///
    /// # Arguments
    ///
    /// * `field_name` - The name of the secondary key field (e.g., "email", "department")
    ///
    /// # Returns
    ///
    /// A vector of all unique values for the specified field.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Get all departments that have users
    /// let departments = user_tree.get_secondary_key_values("department")?;
    /// for dept_bytes in departments {
    ///     let dept = String::from_utf8(dept_bytes.to_vec())?;
    ///     println!("Department: {}", dept);
    /// }
    /// ```
    #[cfg(feature = "native")]
    fn get_secondary_key_values(
        &self,
        field_name: &str,
    ) -> Result<Vec<sled::IVec>, crate::errors::NetabaseError>;

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn get_secondary_key_values(
        &self,
        field_name: &str,
    ) -> Result<Vec<DatabaseIVec>, crate::errors::NetabaseError>;

    /// Create an index for a secondary key field.
    ///
    /// Manually create or rebuild an index for faster secondary key queries.
    /// Indexes are usually created automatically, but this can be used for
    /// rebuilding corrupted indexes or creating indexes for existing data.
    ///
    /// # Arguments
    ///
    /// * `field_name` - The name of the secondary key field to index
    ///
    /// # Performance Note
    ///
    /// Creating an index requires scanning all existing records, which can be
    /// slow for large datasets.
    fn create_secondary_key_index(
        &self,
        field_name: &str,
    ) -> Result<(), crate::errors::NetabaseError>;

    /// Remove an index for a secondary key field.
    ///
    /// Removes the index to save storage space. Secondary key queries will still
    /// work but will be much slower as they'll require full table scans.
    ///
    /// # Arguments
    ///
    /// * `field_name` - The name of the secondary key field to remove index for
    ///
    /// # Warning
    ///
    /// After removing an index, secondary key queries on that field will be O(n)
    /// instead of O(m) where n is total records and m is matching records.
    fn remove_secondary_key_index(
        &self,
        field_name: &str,
    ) -> Result<(), crate::errors::NetabaseError>;
}

/// Trait for relational querying capabilities.
///
/// This trait enables foreign key relationships and join-like operations between
/// different model types. It supports finding models that reference other models
/// and resolving those relationships to load related data.
///
/// ## Relational Concepts
///
/// - **Foreign Keys**: Fields that reference the primary key of another model
/// - **Referencing Models**: Models that contain foreign keys pointing to other models
/// - **Resolution**: The process of loading related models using foreign key values
///
/// ## Usage Examples
///
/// ```rust
/// use netabase_store::traits::NetabaseRelationalQuery;
///
/// // Find all comments that reference a specific post
/// let post_key = PostKey::Primary(PostPrimaryKey(1));
/// let comments = comment_tree.find_referencing_models(post_key)?;
///
/// // Find all posts by a specific author
/// let user_key = UserKey::Primary(UserPrimaryKey(42));
/// let user_posts = post_tree.find_referencing_models(user_key)?;
/// ```
///
/// ## Relation Resolution
///
/// Relations can be resolved to load complete related data:
///
/// ```rust
/// // Resolve author information for a post
/// let mut post = get_post(post_id)?;
/// post_tree.resolve_relations(&mut post, |link| {
///     // Load user data for the author_id
///     user_tree.get(link.foreign_key()).ok().flatten()
/// })?;
/// ```
pub trait NetabaseRelationalQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
    /// Find all models that reference a specific target through relational links.
    ///
    /// This method performs a reverse lookup to find all models that have foreign
    /// keys pointing to the specified target key. It's equivalent to a "WHERE
    /// foreign_key = target_key" query in SQL.
    ///
    /// # Arguments
    ///
    /// * `target_key` - The key of the target model to find references to
    ///
    /// # Returns
    ///
    /// A vector of all models that reference the target key.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Find all comments on a specific post
    /// let post_key = PostKey::Primary(PostPrimaryKey(1));
    /// let comments = comment_tree.find_referencing_models(post_key)?;
    ///
    /// // Find all posts by a specific author
    /// let author_key = UserKey::Primary(UserPrimaryKey(42));
    /// let author_posts = post_tree.find_referencing_models(author_key)?;
    /// ```
    fn find_referencing_models<TargetKey>(
        &self,
        target_key: TargetKey,
    ) -> Result<Vec<M>, crate::errors::NetabaseError>
    where
        TargetKey: NetabaseModelKey + PartialEq;

    /// Get all models that have unresolved relational links.
    ///
    /// Returns models that have foreign key references but where the referenced
    /// models may not exist or may need to be loaded. Useful for data integrity
    /// checks and lazy loading scenarios.
    ///
    /// # Returns
    ///
    /// A vector of (key, model) pairs for models with unresolved relations.
    fn get_unresolved_relations(&self) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>;

    /// Resolve relational links in a model using a custom resolver function.
    ///
    /// This method allows you to populate foreign key relationships by providing
    /// a resolver function that loads the related data. The resolver is called
    /// for each relational link in the model.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to resolve relations for (modified in place)
    /// * `resolver` - Function that takes a relational link and returns the related model
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut post = get_post(1)?;
    /// post_tree.resolve_relations(&mut post, |link| {
    ///     match link.relation_type() {
    ///         "author" => user_tree.get(link.foreign_key()).ok().flatten(),
    ///         "category" => category_tree.get(link.foreign_key()).ok().flatten(),
    ///         _ => None,
    ///     }
    /// })?;
    /// ```
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

    /// Batch resolve relations for multiple models efficiently.
    ///
    /// Similar to `resolve_relations` but operates on multiple models at once,
    /// allowing for optimizations like batched loading and caching of related data.
    ///
    /// # Arguments
    ///
    /// * `models` - Slice of models to resolve relations for (modified in place)
    /// * `resolver` - Function that takes a relational link and returns the related model
    ///
    /// # Performance Note
    ///
    /// This method can be more efficient than calling `resolve_relations` multiple
    /// times as it can batch load related data and avoid duplicate queries.
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

/// Trait for advanced querying capabilities.
///
/// This trait provides powerful querying operations beyond basic CRUD and secondary
/// key queries. It includes range queries, custom filtering, batch operations, and
/// aggregation functions.
///
/// ## Query Types
///
/// - **Range Queries**: Efficiently query records within a key range
/// - **Custom Filters**: Apply arbitrary predicates to filter records
/// - **Batch Operations**: Perform bulk operations efficiently
/// - **Aggregations**: Count and analyze data without loading full records
///
/// ## Performance Characteristics
///
/// - Range queries: O(log n + m) where n is total records, m is results
/// - Custom filters: O(n) - requires scanning all records
/// - Batch operations: Much faster than individual operations
/// - Count operations: O(n) but without memory allocation for results
///
/// ## Usage Examples
///
/// ```rust
/// use netabase_store::traits::NetabaseAdvancedQuery;
///
/// // Range query by key prefix
/// let prefix = b"user_2023_";
/// let recent_users = user_tree.range_by_prefix(prefix)?;
///
/// // Custom filter query
/// let adults = user_tree.query_with_filter(|user| user.age >= 18)?;
///
/// // Count matching records
/// let active_count = user_tree.count_where(|user| user.active)?;
///
/// // Batch insert for efficiency
/// let users = vec![(user1_key, user1), (user2_key, user2)];
/// user_tree.batch_insert_with_indexing(users)?;
/// ```
pub trait NetabaseAdvancedQuery<M, MK>
where
    M: NetabaseModel<Key = MK>,
    MK: NetabaseModelKey,
{
    /// Execute a range query using a key prefix.
    ///
    /// This method efficiently finds all records whose keys start with the
    /// specified prefix. It's useful for hierarchical data, time-based queries,
    /// or any scenario where keys have a meaningful prefix structure.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The byte prefix to search for in keys
    ///
    /// # Returns
    ///
    /// A vector of (key, model) pairs for all matching records.
    ///
    /// # Performance
    ///
    /// Range queries are O(log n + m) where n is total records and m is results,
    /// making them very efficient for prefix-based searches.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Find all users created in 2023 (assuming keys include date)
    /// let prefix = b"user_2023_";
    /// let users_2023 = user_tree.range_by_prefix(prefix)?;
    ///
    /// // Find all posts in a specific category (assuming hierarchical keys)
    /// let tech_prefix = b"post_technology_";
    /// let tech_posts = post_tree.range_by_prefix(tech_prefix)?;
    /// ```
    fn range_by_prefix(&self, prefix: &[u8]) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>;

    /// Batch insert multiple records with automatic secondary key indexing.
    ///
    /// This method efficiently inserts multiple records at once, automatically
    /// maintaining all secondary key indexes. It's much faster than individual
    /// insert operations for bulk data loading.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of (key, model) pairs to insert
    ///
    /// # Performance
    ///
    /// Batch operations are significantly faster than individual inserts because:
    /// - Reduced transaction overhead
    /// - Bulk index updates
    /// - Optimized disk I/O patterns
    ///
    /// # Example
    ///
    /// ```rust
    /// let users = vec![
    ///     (UserKey::Primary(UserPrimaryKey(1)), user1),
    ///     (UserKey::Primary(UserPrimaryKey(2)), user2),
    ///     (UserKey::Primary(UserPrimaryKey(3)), user3),
    /// ];
    /// user_tree.batch_insert_with_indexing(users)?;
    /// ```
    fn batch_insert_with_indexing(
        &self,
        items: Vec<(MK, M)>,
    ) -> Result<(), crate::errors::NetabaseError>;

    /// Query records using a custom filter predicate.
    ///
    /// This method applies a custom function to every record and returns those
    /// that match the predicate. It's very flexible but requires scanning all
    /// records, so it should be used judiciously on large datasets.
    ///
    /// # Arguments
    ///
    /// * `filter` - Function that takes a model and returns true if it should be included
    ///
    /// # Returns
    ///
    /// A vector of (key, model) pairs for all records that match the filter.
    ///
    /// # Performance Warning
    ///
    /// This operation is O(n) and loads all records into memory for filtering.
    /// Consider using secondary key queries when possible for better performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Find all users with specific criteria
    /// let premium_users = user_tree.query_with_filter(|user| {
    ///     user.subscription_type == "Premium" && user.active
    /// })?;
    ///
    /// // Find posts with high engagement
    /// let popular_posts = post_tree.query_with_filter(|post| {
    ///     post.likes > 100 && post.comments > 50
    /// })?;
    /// ```
    fn query_with_filter<F>(&self, filter: F) -> Result<Vec<(MK, M)>, crate::errors::NetabaseError>
    where
        F: Fn(&M) -> bool;

    /// Count records that match a predicate without loading them into memory.
    ///
    /// This method counts records that match a predicate function without
    /// allocating memory for the results. It's more memory-efficient than
    /// `query_with_filter` when you only need the count.
    ///
    /// # Arguments
    ///
    /// * `predicate` - Function that takes a model and returns true if it should be counted
    ///
    /// # Returns
    ///
    /// The number of records that match the predicate.
    ///
    /// # Performance
    ///
    /// While still O(n), this method is more memory-efficient than loading
    /// all matching records since it only keeps a counter.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Count active users
    /// let active_count = user_tree.count_where(|user| user.active)?;
    ///
    /// // Count published posts
    /// let published_count = post_tree.count_where(|post| post.published)?;
    ///
    /// // Count recent posts (using timestamps)
    /// let recent_threshold = chrono::Utc::now().timestamp() - 86400; // 24 hours ago
    /// let recent_count = post_tree.count_where(|post| post.created_at > recent_threshold)?;
    /// ```
    fn count_where<F>(&self, predicate: F) -> Result<usize, crate::errors::NetabaseError>
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

#[cfg(feature = "native")]
pub mod database_traits {
    use crate::{
        errors::NetabaseError,
        traits::{DatabaseIVec, NetabaseModel, NetabaseModelKey},
    };
    use std::collections::HashMap;

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
                    old_ivec_opt.and_then(|ivec| V::try_from(DatabaseIVec::from(ivec)).ok());
                let new_value_opt = f(old_value_opt);
                new_value_opt.and_then(|v| {
                    DatabaseIVec::try_from(v)
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
                    old_ivec_opt.and_then(|ivec| V::try_from(DatabaseIVec::from(ivec)).ok());
                let new_value_opt = f(old_value_opt);
                new_value_opt.and_then(|v| {
                    DatabaseIVec::try_from(v)
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
                    if let Ok(ivec) = DatabaseIVec::try_from(k.clone()) {
                        Bound::Included(ivec)
                    } else {
                        Bound::Unbounded
                    }
                }
                Bound::Excluded(k) => {
                    if let Ok(ivec) = DatabaseIVec::try_from(k.clone()) {
                        Bound::Excluded(ivec)
                    } else {
                        Bound::Unbounded
                    }
                }
                Bound::Unbounded => Bound::Unbounded,
            };

            let end_bound = match range.end_bound() {
                Bound::Included(k) => {
                    if let Ok(ivec) = DatabaseIVec::try_from(k.clone()) {
                        Bound::Included(ivec)
                    } else {
                        Bound::Unbounded
                    }
                }
                Bound::Excluded(k) => {
                    if let Ok(ivec) = DatabaseIVec::try_from(k.clone()) {
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
            match DatabaseIVec::try_from(prefix) {
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
            K: TryFrom<DatabaseIVec>,
            V: TryFrom<DatabaseIVec>,
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
