use crate::errors::NetabaseError;
use crate::traits::NetabaseModelKey;
use bincode::{Decode, Encode};
use std::collections::HashMap;
use std::fmt::Debug;

/// Represents a relational link that can either store a primary key or the actual object
/// for lazy fetching scenarios.
#[derive(Debug, Clone, Encode, Decode, serde::Serialize, serde::Deserialize)]
pub enum RelationalLink<K, T>
where
    K: NetabaseModelKey,
    T: Clone + Debug,
{
    /// Contains only the primary key of the related object
    Key(K),
    /// Contains the actual related object (loaded/resolved)
    Object(T),
}

impl<K, T> RelationalLink<K, T>
where
    K: NetabaseModelKey,
    T: Clone + Debug,
{
    /// Create a new RelationalLink with just a key
    pub fn from_key(key: K) -> Self {
        Self::Key(key)
    }

    /// Create a new RelationalLink with the actual object
    pub fn from_object(object: T) -> Self {
        Self::Object(object)
    }

    /// Get the key if this link contains one
    pub fn key(&self) -> Option<&K> {
        match self {
            Self::Key(k) => Some(k),
            Self::Object(_) => None,
        }
    }

    /// Get the object if this link contains one
    pub fn object(&self) -> Option<&T> {
        match self {
            Self::Key(_) => None,
            Self::Object(obj) => Some(obj),
        }
    }

    /// Check if this link is resolved (contains an object)
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Check if this link is unresolved (contains only a key)
    pub fn is_unresolved(&self) -> bool {
        matches!(self, Self::Key(_))
    }

    /// Resolve the link by replacing the key with an object (consuming version)
    pub fn resolve(self, object: T) -> Self {
        Self::Object(object)
    }

    /// Resolve the link in-place by mutating it to contain the object and return a reference
    pub fn resolve_mut(&mut self, object: T) -> &T {
        *self = Self::Object(object);
        match self {
            Self::Object(obj) => obj,
            _ => unreachable!("We just set it to Object"),
        }
    }

    /// Resolve the link in-place if it's currently unresolved, return reference to object
    pub fn resolve_if_unresolved(&mut self, object: T) -> &T {
        if self.is_unresolved() {
            self.resolve_mut(object)
        } else {
            self.object()
                .expect("Link should be resolved at this point")
        }
    }

    /// Convert to key, consuming the link
    pub fn into_key(self) -> Option<K> {
        match self {
            Self::Key(k) => Some(k),
            Self::Object(_) => None,
        }
    }

    /// Convert to object, consuming the link
    pub fn into_object(self) -> Option<T> {
        match self {
            Self::Key(_) => None,
            Self::Object(obj) => Some(obj),
        }
    }

    /// Get a mutable reference to the object if resolved, or None if unresolved
    pub fn object_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Key(_) => None,
            Self::Object(obj) => Some(obj),
        }
    }
}

impl<K, T> From<K> for RelationalLink<K, T>
where
    K: NetabaseModelKey,
    T: Clone + Debug,
{
    fn from(key: K) -> Self {
        Self::Key(key)
    }
}

impl<K, T> PartialEq for RelationalLink<K, T>
where
    K: NetabaseModelKey + PartialEq,
    T: Clone + Debug + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Key(k1), Self::Key(k2)) => k1 == k2,
            (Self::Object(obj1), Self::Object(obj2)) => obj1 == obj2,
            _ => false,
        }
    }
}

impl<K, T> Eq for RelationalLink<K, T>
where
    K: NetabaseModelKey + Eq,
    T: Clone + Debug + Eq,
{
}

impl<K, T> std::hash::Hash for RelationalLink<K, T>
where
    K: NetabaseModelKey + std::hash::Hash,
    T: Clone + Debug + std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Key(k) => {
                0u8.hash(state);
                k.hash(state);
            }
            Self::Object(obj) => {
                1u8.hash(state);
                obj.hash(state);
            }
        }
    }
}

/// Utility for resolving relational links in bulk
pub struct RelationalResolver<K, T>
where
    K: NetabaseModelKey + Clone,
    T: Clone + Debug,
{
    cache: HashMap<String, T>,
    loader: Box<dyn Fn(&K) -> Result<Option<T>, NetabaseError>>,
}

impl<K, T> RelationalResolver<K, T>
where
    K: NetabaseModelKey + Clone + std::hash::Hash + Eq,
    T: Clone + Debug,
{
    /// Create a new resolver with a custom loader function
    pub fn new<F>(loader: F) -> Self
    where
        F: Fn(&K) -> Result<Option<T>, NetabaseError> + 'static,
    {
        Self {
            cache: HashMap::new(),
            loader: Box::new(loader),
        }
    }

    /// Resolve a single relational link
    pub fn resolve(&mut self, link: &RelationalLink<K, T>) -> Result<Option<T>, NetabaseError> {
        match link {
            RelationalLink::Key(key) => {
                // Check cache first
                let key_str = format!("{:?}", key);
                if let Some(cached) = self.cache.get(&key_str) {
                    return Ok(Some(cached.clone()));
                }

                // Load from storage
                if let Some(object) = (self.loader)(key)? {
                    self.cache.insert(key_str, object.clone());
                    Ok(Some(object))
                } else {
                    Ok(None)
                }
            }
            RelationalLink::Object(obj) => Ok(Some(obj.clone())),
        }
    }

    /// Resolve multiple relational links efficiently
    pub fn resolve_many(
        &mut self,
        links: &[RelationalLink<K, T>],
    ) -> Result<Vec<Option<T>>, NetabaseError> {
        let mut results = Vec::new();

        for link in links {
            results.push(self.resolve(link)?);
        }

        Ok(results)
    }

    /// Clear the internal cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}

/// Batch resolver for handling multiple different types of relations
pub struct BatchRelationalResolver {
    resolvers: HashMap<String, Box<dyn std::any::Any>>,
}

impl BatchRelationalResolver {
    /// Create a new batch resolver
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }

    /// Add a resolver for a specific type
    pub fn add_resolver<K, T>(&mut self, type_name: &str, resolver: RelationalResolver<K, T>)
    where
        K: NetabaseModelKey + Clone + std::hash::Hash + Eq + 'static,
        T: Clone + Debug + 'static,
    {
        self.resolvers
            .insert(type_name.to_string(), Box::new(resolver));
    }

    /// Get a resolver for a specific type
    pub fn get_resolver<K, T>(&mut self, type_name: &str) -> Option<&mut RelationalResolver<K, T>>
    where
        K: NetabaseModelKey + Clone + std::hash::Hash + Eq + 'static,
        T: Clone + Debug + 'static,
    {
        self.resolvers
            .get_mut(type_name)
            .and_then(|resolver| resolver.downcast_mut())
    }
}

impl Default for BatchRelationalResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for models that contain relational links
pub trait HasRelationalLinks {
    /// Get all unresolved relational links in this model
    fn get_unresolved_links(&self) -> Vec<String>;

    /// Resolve all relational links using the provided resolver
    fn resolve_all_links(
        &mut self,
        resolver: &mut BatchRelationalResolver,
    ) -> Result<(), NetabaseError>;
}

/// Utility functions for working with relational links
pub mod utils {
    use super::*;

    /// Extract all keys from a vector of relational links
    pub fn extract_keys<K, T>(links: &[RelationalLink<K, T>]) -> Vec<K>
    where
        K: NetabaseModelKey + Clone,
        T: Clone + Debug,
    {
        links
            .iter()
            .filter_map(|link| match link {
                RelationalLink::Key(key) => Some(key.clone()),
                RelationalLink::Object(_) => None,
            })
            .collect()
    }

    /// Check if any links in a collection are unresolved
    pub fn has_unresolved_links<K, T>(links: &[RelationalLink<K, T>]) -> bool
    where
        K: NetabaseModelKey,
        T: Clone + Debug,
    {
        links.iter().any(|link| link.is_unresolved())
    }

    /// Count unresolved links in a collection
    pub fn count_unresolved<K, T>(links: &[RelationalLink<K, T>]) -> usize
    where
        K: NetabaseModelKey,
        T: Clone + Debug,
    {
        links.iter().filter(|link| link.is_unresolved()).count()
    }

    /// Convert a vector of keys to unresolved relational links
    pub fn keys_to_links<K, T>(keys: Vec<K>) -> Vec<RelationalLink<K, T>>
    where
        K: NetabaseModelKey,
        T: Clone + Debug,
    {
        keys.into_iter().map(RelationalLink::from_key).collect()
    }

    /// Convert a vector of objects to resolved relational links
    pub fn objects_to_links<K, T>(objects: Vec<T>) -> Vec<RelationalLink<K, T>>
    where
        K: NetabaseModelKey,
        T: Clone + Debug,
    {
        objects
            .into_iter()
            .map(RelationalLink::from_object)
            .collect()
    }
}

// Type aliases are now generated by the NetabaseModel derive macro
