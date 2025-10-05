//! # Netabase Macros
//!
//! This crate provides procedural macros for the Netabase distributed database system.
//! It generates type-safe database models, keys, and schemas with support for primary keys,
//! secondary keys, and relational queries.
//!
//! ## Main Macros
//!
//! - [`NetabaseModel`] - Derive macro for creating database models
//! - [`netabase_schema_module`] - Attribute macro for organizing models into schemas
//! - [`NetabaseModelKey`] - Derive macro for custom key types
//!
//! ## Basic Usage
//!
//! ```rust
//! use netabase_macros::{NetabaseModel, netabase_schema_module};
//! use netabase_deps::{bincode, serde}; // Re-exported for convenience
//!
//! #[netabase_schema_module(BlogSchema, BlogKeys)]
//! mod blog {
//!     use super::*;
//!
//!     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
//!     #[key_name(UserKey)]
//!     pub struct User {
//!         #[key]
//!         pub id: u64,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, ItemMod, parse_macro_input, visit::Visit, visit_mut::VisitMut};

use crate::{
    generators::{
        append_ident, models::impls::key_impl::generate_netabase_model_key_trait,
        schema::model_enum::generate_module_schema,
    },
    visitors::{
        netabase_schema_derive::{DeriveVisitor, NetabaseModelVisitor},
        schema_module::SchemaModuleVisitor,
    },
};

mod generators;
mod util;
mod visitors;

/// Derive macro for creating Netabase database models.
///
/// This macro generates all the necessary types and trait implementations to make a struct
/// work as a Netabase model, including key types, serialization support, and query capabilities.
///
/// ## Required Attributes
///
/// - `#[key]` - Must be applied to exactly one field to mark it as the primary key
/// - `#[key_name(KeyTypeName)]` - Must be applied to the struct to name the generated key type
///
/// ## Optional Attributes
///
/// - `#[secondary_key]` - Applied to fields that should be indexed for efficient querying
///
/// ## Generated Types
///
/// For a struct named `User` with `#[key_name(UserKey)]`:
///
/// - `UserKey` - Main key enum with `Primary` and `Secondary` variants
/// - `UserPrimaryKey` - Newtype wrapper for the primary key value
/// - `UserSecondaryKeys` - Enum containing all secondary key variants
/// - `UserRelations` - Enum for relational keys (if any relations are defined)
///
/// ## Generated Trait Implementations
///
/// - `NetabaseModel` - Core model trait with key extraction and metadata methods
/// - Serialization traits for storage (`TryFrom<IVec>`, `TryInto<IVec>`)
/// - Network serialization support (when libp2p feature is enabled)
///
/// ## Example
///
/// ```rust
/// use netabase_macros::NetabaseModel;
/// use netabase_deps::{bincode, serde}; // Re-exported for convenience
///
/// #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
/// #[key_name(UserKey)]
/// pub struct User {
///     #[key]
///     pub id: u64,                    // Primary key
///     pub name: String,
///     #[secondary_key]
///     pub email: String,              // Indexed for efficient queries
///     #[secondary_key]
///     pub department: String,         // Another indexed field
///     pub created_at: u64,
/// }
///
/// // Generated usage:
/// let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into(),
///                   department: "Engineering".into(), created_at: 1234567890 };
/// let key = user.key(); // Returns UserKey::Primary(UserPrimaryKey(1))
///
/// // Secondary key queries:
/// let email_key = UserSecondaryKeys::EmailKey("alice@example.com".into());
/// let dept_key = UserSecondaryKeys::DepartmentKey("Engineering".into());
/// ```
///
/// ## Error Cases
///
/// The macro will produce a compile error if:
/// - No field is marked with `#[key]`
/// - Multiple fields are marked with `#[key]`
/// - The `#[key_name]` attribute is missing
///
/// ## Performance Notes
///
/// - Primary key access is O(log n)
/// - Secondary key queries are O(m) where m is the number of matching records
/// - Each secondary key adds storage and indexing overhead
#[proc_macro_derive(NetabaseModel, attributes(key, secondary_key, key_name))]
pub fn netabase_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    // Analyze the input without transformation
    let mut visitor = NetabaseModelVisitor::default();
    visitor.visit_derive_input(&derive_input);

    // Check if we found the key field
    if visitor.key_field.is_none() {
        return quote! {
            compile_error!("NetabaseModel requires a field marked with #[key]");
        }
        .into();
    }

    // Generate each component with error handling
    let (main_key_enum, secondary_keys_enum, primary_key_struct, relations_enum) =
        visitor.generate_key();

    let netabase_impl = visitor.generate_netabase_model_trait();

    let main_key_impl = visitor.generate_main_key_impl();

    let primary_key_impl = visitor.generate_primary_key_impl();

    let primary_key_from_impl = visitor.generate_primary_key_from_impl();

    let secondary_keys_impl = visitor.generate_secondary_keys_impl();

    let secondary_keys_try_from_ivec_impl = visitor.generate_secondary_keys_try_from_ivec_impl();

    let secondary_keys_try_into_ivec_impl = visitor.generate_secondary_keys_try_into_ivec_impl();

    let secondary_keys_fn = visitor.generate_secondary_keys_fn();

    let relations_impl = visitor.generate_relations_impl();

    let relations_try_from_ivec_impl = visitor.generate_relations_try_from_ivec_impl();

    let relations_try_into_ivec_impl = visitor.generate_relations_try_into_ivec_impl();

    let relations_fn = visitor.generate_relations_fn();

    let secondary_keys_placeholder_impls = visitor.generate_secondary_keys_placeholder_impls();
    let relations_placeholder_impls = visitor.generate_relations_placeholder_impls();

    let type_alias = visitor.generate_type_alias();

    let main_key_ivec_impls = visitor.generate_main_key_ivec_impls();
    let model_ivec_impls = visitor.generate_model_ivec_impls();

    let final_tokens = quote! {
        #primary_key_struct
        #primary_key_impl
        #primary_key_from_impl
        #secondary_keys_enum
        #secondary_keys_impl
        #secondary_keys_try_from_ivec_impl
        #secondary_keys_try_into_ivec_impl
        #secondary_keys_placeholder_impls
        #relations_enum
        #relations_impl
        #relations_try_from_ivec_impl
        #relations_try_into_ivec_impl
        #relations_placeholder_impls
        #main_key_enum
        #main_key_impl
        #main_key_ivec_impls
        #model_ivec_impls
        #netabase_impl
        #secondary_keys_fn
        #relations_fn
        #type_alias
    };

    final_tokens.into()
}

/// Derive macro for creating custom Netabase key types.
///
/// This macro is used for advanced scenarios where you need custom key behavior.
/// Most users should use the `NetabaseModel` derive macro instead, which automatically
/// generates appropriate key types.
///
/// ## Usage
///
/// ```rust
/// use netabase_macros::NetabaseModelKey;
/// use netabase_deps::{bincode, serde}; // Re-exported for convenience
///
/// #[derive(NetabaseModelKey, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
/// pub struct CustomKey {
///     pub field1: String,
///     pub field2: u64,
/// }
/// ```
///
/// ## Generated Implementations
///
/// - `NetabaseModelKey` trait implementation
/// - Serialization support for storage and networking
#[proc_macro_derive(NetabaseModelKey)]
pub fn netabase_key_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let netabase_impl = generate_netabase_model_key_trait(&derive_input);
    quote! {
        #netabase_impl
    }
    .into()
}

/// Internal attribute macro for key derivation.
///
/// This is an implementation detail used by other macros and should not be used directly.
#[proc_macro_attribute]
pub fn key_derive(_derives: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let mut der_visitor = DeriveVisitor::new();
    der_visitor.visit_derive_input_mut(&mut input);
    quote::quote!(#input).into()
}

/// Attribute macro for creating Netabase schema modules.
///
/// This macro transforms a module containing Netabase models into a unified schema
/// with centralized types for all models and their keys. It enables type-safe
/// operations across multiple model types and provides network serialization support.
///
/// ## Syntax
///
/// ```rust
/// #[netabase_schema_module(SchemaName, SchemaKeysName)]
/// mod module_name {
///     // Model definitions here
/// }
/// ```
///
/// ## Parameters
///
/// - `SchemaName` - Name for the generated schema enum containing all models
/// - `SchemaKeysName` - Name for the generated keys enum containing all key types
///
/// ## Generated Types
///
/// For `#[netabase_schema_module(BlogSchema, BlogKeys)]`:
///
/// - `BlogSchema` - Enum with variants for each model type (e.g., `User(User)`, `Post(Post)`)
/// - `BlogKeys` - Enum with variants for each key type (e.g., `UserKey(UserKey)`, `PostKey(PostKey)`)
///
/// ## Generated Implementations
///
/// - `NetabaseSchema` trait for the schema enum
/// - `From` implementations to convert models to schema variants
/// - `From` implementations to convert keys to schema key variants
/// - Serialization support for storage and networking
/// - libp2p integration (when libp2p feature is enabled)
///
/// ## Example
///
/// ```rust
/// use netabase_macros::{NetabaseModel, netabase_schema_module};
/// use netabase_deps::{bincode, serde}; // Re-exported for convenience
///
/// #[netabase_schema_module(BlogSchema, BlogKeys)]
/// mod blog {
///     use super::*;
///
///     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
///     #[key_name(UserKey)]
///     pub struct User {
///         #[key] pub id: u64,
///         pub name: String,
///         #[secondary_key] pub email: String,
///     }
///
///     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
///     #[key_name(PostKey)]
///     pub struct Post {
///         #[key] pub id: u64,
///         pub title: String,
///         #[secondary_key] pub author_id: u64,
///     }
/// }
///
/// use blog::*;
///
/// // Usage:
/// let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into() };
/// let schema_item = BlogSchema::User(user);  // Automatic conversion
///
/// let user_key = UserKey::Primary(UserPrimaryKey(1));
/// let schema_key = BlogKeys::UserKey(user_key);  // Key unification
/// ```
///
/// ## Database Integration
///
/// Schema modules integrate with the database layer:
///
/// ```rust
/// use netabase_store::database::NetabaseSledDatabase;
///
/// let db = NetabaseSledDatabase::<BlogSchema>::new()?;
/// let user_tree = db.get_main_tree::<User, UserKey>()?;
/// let post_tree = db.get_main_tree::<Post, PostKey>()?;
/// ```
///
/// ## Network Integration
///
/// Schema modules enable distributed operations:
///
/// ```rust
/// use netabase::Netabase;
///
/// let mut netabase = Netabase::<BlogSchema>::new()?;
/// netabase.start_swarm().await?;
///
/// // Put any model type into the DHT
/// netabase.put_record(user).await?;
/// netabase.put_record(post).await?;
/// ```
#[proc_macro_attribute]
pub fn netabase_schema_module(name: TokenStream, input: TokenStream) -> TokenStream {
    // let name = parse_macro_input!(name as Ident);
    let binding = name.to_string();
    let mut split = binding.split(",");
    let schema_ident = match split.next() {
        Some(sp) => Ident::new(sp.trim(), proc_macro2::Span::call_site()),
        None => panic!("Schema needs a name"),
    };
    let key_ident = match split.next() {
        Some(sp) => Ident::new(sp.trim(), proc_macro2::Span::call_site()),
        None => append_ident(&schema_ident, "Key"),
    };
    let mut input = parse_macro_input!(input as ItemMod);
    let mut visitor = SchemaModuleVisitor::new(schema_ident, key_ident);
    visitor.visit_item_mod(&input);

    let (schema, key, impls) = generate_module_schema(visitor);
    let temp_cont = input.content.unwrap();
    let mut new_vec = temp_cont.1;
    new_vec.push(syn::Item::Enum(schema));
    new_vec.push(syn::Item::Enum(key));

    // Add all the generated implementations
    for impl_item in impls {
        new_vec.push(syn::Item::Impl(impl_item));
    }

    input.content = Some((temp_cont.0, new_vec));
    quote! {
        #input
    }
    .into()
}

/// Attribute macro for marking relational key schemas.
///
/// This is used in advanced relational scenarios to mark fields that reference
/// other model types. It's primarily used internally by the macro system.
///
/// ## Usage
///
/// ```rust
/// #[key_schema]
/// pub some_field: RelatedModelKey,
/// ```
#[proc_macro_attribute]
pub fn key_schema(_item: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Attribute macro for specifying custom key names in relational contexts.
///
/// This is an internal attribute used by the macro system for advanced
/// relational key naming. Most users won't need to use this directly.
///
/// ## Usage
///
/// ```rust
/// #[key_name(CustomKeyName)]
/// pub field: SomeType,
/// ```
#[proc_macro_attribute]
pub fn key_name(_item: TokenStream, input: TokenStream) -> TokenStream {
    input
}
