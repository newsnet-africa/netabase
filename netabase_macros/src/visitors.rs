//! Visitor implementations for netabase macro processing
//!
//! This module provides comprehensive visitor functionality for processing netabase schemas,
//! including schema discovery, validation, and key generation. The module is organized into
//! several submodules for better maintainability.

// Declare submodules
pub mod errors;
pub mod key_generation;
pub mod schema;
pub mod validation;

// Re-export commonly used types for backward compatibility and ease of use
pub use errors::{NetabaseError, NetabaseResult, ToCompileError};
pub use key_generation::{
    KeyFunctionGenerator, KeyGenerator, KeyGeneratorLabels, generate_key_function,
};
pub use schema::{
    SchemaInfo, SchemaProcessor, SchemaType, SchemaValidator, ValidSchemaFinder, ValidationItem,
};
pub use validation::{ValidationContext, ValidationResult, Validator};

// Legacy compatibility re-exports
pub use schema::{
    SchemaValidator as LegacySchemaValidator, ValidSchemaFinder as LegacyValidSchemaFinder,
};

use proc_macro::Span;
use proc_macro2::{Group, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{ExprClosure, Field, Item, ItemFn, Pat, Type, spanned::Spanned};

/// Enhanced key generation functionality with support for both field-level and item-level closures
pub struct EnhancedKeyProcessor<'ast> {
    processor: SchemaProcessor<'ast>,
}

impl<'ast> EnhancedKeyProcessor<'ast> {
    /// Create a new enhanced key processor
    pub fn new() -> Self {
        Self {
            processor: SchemaProcessor::new(),
        }
    }

    /// Process a field to determine its key generation strategy with enhanced logic
    pub fn analyze_field_with_context(
        &self,
        field: &'ast Field,
        item_context: Option<&Type>,
    ) -> NetabaseResult<KeyGenerator<'ast>> {
        // Start with the basic field analysis
        let mut key_gen = self.processor.analyze_field_for_key(field)?;

        // If we have item context and found a closure, refine the analysis
        if let Some(item_type) = item_context {
            if let KeyGenerator::FieldClosure(closure) = &key_gen {
                if self.should_be_item_closure(closure, field, item_type) {
                    key_gen = KeyGenerator::ItemClosure(closure);
                }
            }
        }

        Ok(key_gen)
    }

    /// Determine if a closure should be treated as item-level based on enhanced heuristics
    fn should_be_item_closure(
        &self,
        closure: &ExprClosure,
        field: &Field,
        item_type: &Type,
    ) -> bool {
        // Check closure input type
        if let Some(input) = closure.inputs.first() {
            if let Pat::Type(pat_type) = input {
                let input_type_str = pat_type.ty.to_token_stream().to_string();
                let field_type_str = field.ty.to_token_stream().to_string();
                let item_type_str = item_type.to_token_stream().to_string();

                // If input matches item type, it's item-level
                if input_type_str == item_type_str {
                    return true;
                }

                // If input is a reference to item type, it's item-level
                if let Type::Reference(type_ref) = pat_type.ty.as_ref() {
                    let referenced_type_str = type_ref.elem.to_token_stream().to_string();
                    if referenced_type_str == item_type_str {
                        return true;
                    }
                }

                // If input doesn't match field type and looks like a complex type, it's likely item-level
                if input_type_str != field_type_str && input_type_str.contains("::") {
                    return true;
                }
            }
        }

        false
    }

    /// Generate key function with enhanced context awareness
    pub fn generate_contextual_key_function(
        &self,
        field: &'ast Field,
        key_gen: KeyGenerator<'ast>,
        item_type: Option<&'ast Type>,
    ) -> NetabaseResult<ItemFn> {
        let labels = if let Some(item_type) = item_type {
            KeyGeneratorLabels::with_item_type(field, key_gen, item_type)
        } else {
            KeyGeneratorLabels::new(field, key_gen)
        };

        generate_key_function(labels)
    }
}

/// Utility functions for common operations
pub mod utils {
    use super::*;

    /// Create a simple field key generator
    pub fn create_field_key(field: &Field) -> KeyGenerator {
        KeyGenerator::Field(field)
    }

    /// Create a field closure key generator with validation
    pub fn create_field_closure_key<'a>(
        closure: &'a ExprClosure,
        field: &'a Field,
    ) -> NetabaseResult<KeyGenerator<'a>> {
        // Basic validation
        if closure.inputs.is_empty() {
            return Err(NetabaseError::KeyGeneration(
                errors::KeyGenerationError::MissingClosureInput {
                    field_name: field.ident.as_ref().map(|i| i.to_string()),
                    span: closure.span(),
                },
            ));
        }

        Ok(KeyGenerator::FieldClosure(closure))
    }

    /// Create an item closure key generator with validation
    pub fn create_item_closure_key<'a>(
        closure: &'a ExprClosure,
        _item_type: &'a Type,
    ) -> NetabaseResult<KeyGenerator<'a>> {
        // Basic validation
        if closure.inputs.is_empty() {
            return Err(NetabaseError::KeyGeneration(
                errors::KeyGenerationError::MissingClosureInput {
                    field_name: None,
                    span: closure.span(),
                },
            ));
        }

        Ok(KeyGenerator::ItemClosure(closure))
    }

    /// Validate that a closure has the correct signature for key generation
    pub fn validate_closure_signature(
        closure: &ExprClosure,
        expected_input_type: &Type,
        field_name: Option<&str>,
    ) -> NetabaseResult<()> {
        if closure.inputs.len() != 1 {
            return Err(NetabaseError::KeyGeneration(
                errors::KeyGenerationError::MissingClosureInput {
                    field_name: field_name.map(|s| s.to_string()),
                    span: closure.span(),
                },
            ));
        }

        if let Some(Pat::Type(pat_type)) = closure.inputs.first() {
            let input_type_str = pat_type.ty.to_token_stream().to_string();
            let expected_type_str = expected_input_type.to_token_stream().to_string();

            if input_type_str != expected_type_str {
                // Check if it's a reference to the expected type
                if let Type::Reference(type_ref) = pat_type.ty.as_ref() {
                    let referenced_type_str = type_ref.elem.to_token_stream().to_string();
                    if referenced_type_str == expected_type_str {
                        return Ok(());
                    }
                }

                return Err(NetabaseError::KeyGeneration(
                    errors::KeyGenerationError::ClosureInputTypeMismatch {
                        field_name: field_name.unwrap_or("unknown").to_string(),
                        field_type: expected_type_str,
                        closure_input_type: input_type_str,
                        span: pat_type.ty.span(),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Extract key fields from any item (struct or enum)
    pub fn extract_all_key_fields(item: &Item) -> Vec<(&Field, KeyGenerator)> {
        let processor = SchemaProcessor::new();

        match item {
            Item::Struct(item_struct) => processor
                .extract_key_fields(&item_struct.fields)
                .unwrap_or_default(),
            Item::Enum(item_enum) => {
                let mut all_fields = Vec::new();
                for variant in &item_enum.variants {
                    let variant_fields = processor
                        .extract_key_fields(&variant.fields)
                        .unwrap_or_default();
                    all_fields.extend(variant_fields);
                }
                all_fields
            }
            _ => Vec::new(),
        }
    }

    /// Check if an item has any key fields
    pub fn has_key_fields(item: &Item) -> bool {
        !extract_all_key_fields(item).is_empty()
    }

    /// Get the first key field from an item
    pub fn get_primary_key_field(item: &Item) -> Option<(&Field, KeyGenerator)> {
        extract_all_key_fields(item).into_iter().next()
    }
}

/// A struct that holds fields and a key generator, capable of converting to a key generator function.
///
/// This struct combines `syn::Fields` with a `KeyGenerator` to produce complete key generation
/// functions through the `ToTokens` trait. It supports various key generation strategies including
/// direct field access, field closures, and item-level closures.
///
/// ## Key Generation Strategies
///
/// ### Direct Field Access
/// Uses the field value directly as the key:
/// ```ignore
/// use netabase_macros::visitors::{FieldKeyGenerator, KeyGenerator};
/// use syn::parse_quote;
///
/// let item_struct: syn::ItemStruct = parse_quote! {
///     struct User {
///         #[key]
///         id: String,
///         name: String,
///     }
/// };
/// let field: syn::Field = parse_quote! { id: String };
/// let key_gen = KeyGenerator::Field(&field);
/// let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);
/// ```
///
/// ### Field Closure
/// Uses a closure that takes the field value and transforms it:
/// ```ignore
/// let closure: syn::ExprClosure = parse_quote! { |id| format!("user_{}", id) };
/// let key_gen = KeyGenerator::FieldClosure(&closure);
/// let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);
/// ```
///
/// ### Item Closure
/// Uses a closure that takes the entire item and extracts a key:
/// ```ignore
/// let closure: syn::ExprClosure = parse_quote! { |user| user.id.clone() };
/// let item_type: syn::Type = parse_quote! { User };
/// let key_gen = KeyGenerator::ItemClosure(&closure);
/// let field_key_gen = FieldKeyGenerator::with_item_type(
///     &item_struct.fields,
///     key_gen,
///     &item_type
/// );
/// ```
///
/// ## Generated Functions
///
/// The `ToTokens` implementation generates functions with signatures appropriate to the
/// key generation strategy:
///
/// - **Field access**: `pub fn key(item: &FieldType) -> impl From<Vec<u8>> + Into<Vec<u8>>`
/// - **Field closure**: `pub fn key(item: &FieldType) -> impl From<Vec<u8>> + Into<Vec<u8>>`
/// - **Item closure**: `pub fn key(item: &ItemType) -> impl From<Vec<u8>> + Into<Vec<u8>>`
///
/// ## Error Handling
///
/// If key generation fails or no fields are available, a fallback function is generated:
/// ```rust,ignore
/// pub fn key() -> String {
///     String::new()
/// }
/// ```
///
/// ## Usage in Proc Macros
///
/// This struct is typically used within proc macro implementations to generate
/// key functions for database schemas:
///
/// ```ignore
/// use quote::ToTokens;
/// use proc_macro2::TokenStream;
///
/// let mut tokens = TokenStream::new();
/// field_key_gen.to_tokens(&mut tokens);
/// // `tokens` now contains the generated key function
/// ```
pub struct FieldKeyGenerator<'ast> {
    /// The fields from a struct or enum variant
    pub fields: &'ast syn::Fields,
    /// The key generation strategy to use
    pub key_generator: KeyGenerator<'ast>,
    /// Optional item type for item-level closures
    pub item_type: Option<&'ast syn::Type>,
}

impl<'ast> FieldKeyGenerator<'ast> {
    /// Create a new FieldKeyGenerator
    pub fn new(fields: &'ast syn::Fields, key_generator: KeyGenerator<'ast>) -> Self {
        Self {
            fields,
            key_generator,
            item_type: None,
        }
    }

    /// Create a new FieldKeyGenerator with item type context
    pub fn with_item_type(
        fields: &'ast syn::Fields,
        key_generator: KeyGenerator<'ast>,
        item_type: &'ast syn::Type,
    ) -> Self {
        Self {
            fields,
            key_generator,
            item_type: Some(item_type),
        }
    }

    /// Get the primary field from the fields (first named field or first field)
    fn get_primary_field(&self) -> Option<&'ast syn::Field> {
        match self.fields {
            syn::Fields::Named(named) => named.named.first(),
            syn::Fields::Unnamed(unnamed) => unnamed.unnamed.first(),
            syn::Fields::Unit => None,
        }
    }
}

impl<'ast> FieldKeyGenerator<'ast> {
    /// Generate a key function with proper signature based on the key generator type
    fn generate_key_function_with_signature(&self) -> syn::ItemFn {
        let field = match self.get_primary_field() {
            Some(field) => field,
            None => {
                // No field available - generate generic fallback
                return syn::parse_quote! {
                    pub fn key<T>(item: &T) -> impl From<Vec<u8>> + Into<Vec<u8>> {
                        Vec::<u8>::new()
                    }
                };
            }
        };

        match &self.key_generator {
            KeyGenerator::Field(_) => {
                let field_type = &field.ty;
                let field_name = field.ident.as_ref().map(|i| i.clone()).unwrap_or_else(|| {
                    syn::Ident::new("field_0", proc_macro::Span::call_site().into())
                });

                syn::parse_quote! {
                    pub fn key(item: &#field_type) -> impl From<Vec<u8>> + Into<Vec<u8>> {
                        item.#field_name.clone().into()
                    }
                }
            }
            KeyGenerator::FieldClosure(closure) => {
                let field_type = &field.ty;
                syn::parse_quote! {
                    pub fn key(item: &#field_type) -> impl From<Vec<u8>> + Into<Vec<u8>> {
                        (#closure)(item.clone())
                    }
                }
            }
            KeyGenerator::ItemClosure(closure) => {
                if let Some(item_type) = self.item_type {
                    syn::parse_quote! {
                        pub fn key(item: &#item_type) -> impl From<Vec<u8>> + Into<Vec<u8>> {
                            (#closure)(item)
                        }
                    }
                } else {
                    let default_type: syn::Type = syn::parse_quote! { Self };
                    syn::parse_quote! {
                        pub fn key(item: &#default_type) -> impl From<Vec<u8>> + Into<Vec<u8>> {
                            (#closure)(item)
                        }
                    }
                }
            }
            KeyGenerator::None => {
                let field_type = &field.ty;
                syn::parse_quote! {
                    pub fn key(item: &#field_type) -> impl From<Vec<u8>> + Into<Vec<u8>> {
                        Vec::<u8>::new()
                    }
                }
            }
        }
    }
}

impl<'ast> quote::ToTokens for FieldKeyGenerator<'ast> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // First try to use the existing key generation infrastructure
        if let Some(field) = self.get_primary_field() {
            let labels = if let Some(item_type) = self.item_type {
                KeyGeneratorLabels::with_item_type(field, self.key_generator.clone(), item_type)
            } else {
                KeyGeneratorLabels::new(field, self.key_generator.clone())
            };

            // Try to generate using existing infrastructure
            if let Ok(item_fn) = generate_key_function(labels) {
                item_fn.to_tokens(tokens);
                return;
            }
        }

        // Fallback to custom generation with proper signatures
        let key_fn = self.generate_key_function_with_signature();
        key_fn.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_key_generator_creation() {
        let field: Field = parse_quote! { id: String };
        let key_gen = utils::create_field_key(&field);

        match key_gen {
            KeyGenerator::Field(_) => assert!(true),
            _ => panic!("Expected field key generator"),
        }
    }

    #[test]
    fn test_closure_validation() {
        let closure: ExprClosure = parse_quote! { |x: String| x.into_bytes() };
        let input_type: Type = parse_quote! { String };

        let result = utils::validate_closure_signature(&closure, &input_type, Some("test_field"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhanced_key_processor() {
        let processor = EnhancedKeyProcessor::new();
        let field: Field = parse_quote! { #[key] id: String };

        let result = processor.analyze_field_with_context(&field, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_key_generator() {
        let item_struct: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                id: String,
                name: String,
            }
        };
        let fields = &item_struct.fields;
        let field: syn::Field = syn::parse_quote! { id: String };
        let key_gen = KeyGenerator::Field(&field);
        let field_key_gen = FieldKeyGenerator::new(fields, key_gen);

        // Test that the struct can be created successfully
        assert!(field_key_gen.get_primary_field().is_some());
    }

    #[test]
    fn test_field_key_generator_with_closure() {
        let item_struct: syn::ItemStruct = syn::parse_quote! {
            struct User {
                #[key]
                id: u64,
                name: String,
            }
        };
        let fields = &item_struct.fields;
        let closure: syn::ExprClosure = syn::parse_quote! { |id| format!("user_{}", id) };
        let key_gen = KeyGenerator::FieldClosure(&closure);
        let field_key_gen = FieldKeyGenerator::new(fields, key_gen);

        // Test that we can generate tokens
        use quote::ToTokens;
        let mut tokens = proc_macro2::TokenStream::new();
        field_key_gen.to_tokens(&mut tokens);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_field_key_generator_with_item_type() {
        let item_struct: syn::ItemStruct = syn::parse_quote! {
            struct Product {
                id: String,
                category: String,
            }
        };
        let fields = &item_struct.fields;
        let item_type: syn::Type = syn::parse_quote! { Product };
        let closure: syn::ExprClosure = syn::parse_quote! { |product| product.id.clone() };
        let key_gen = KeyGenerator::ItemClosure(&closure);
        let field_key_gen = FieldKeyGenerator::with_item_type(fields, key_gen, &item_type);

        // Test that the item type is properly set
        assert!(field_key_gen.item_type.is_some());

        // Test token generation
        use quote::ToTokens;
        let mut tokens = proc_macro2::TokenStream::new();
        field_key_gen.to_tokens(&mut tokens);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_field_key_generator_empty_fields() {
        let fields: syn::Fields = syn::Fields::Unit;
        let field: syn::Field = syn::parse_quote! { id: String };
        let key_gen = KeyGenerator::Field(&field);
        let field_key_gen = FieldKeyGenerator::new(&fields, key_gen);

        // Test that empty fields are handled gracefully
        assert!(field_key_gen.get_primary_field().is_none());

        // Test that fallback function is generated
        use quote::ToTokens;
        let mut tokens = proc_macro2::TokenStream::new();
        field_key_gen.to_tokens(&mut tokens);
        assert!(!tokens.is_empty());
    }
}
