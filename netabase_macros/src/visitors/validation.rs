//! Validation utilities for netabase macro processing
//!
//! This module provides comprehensive validation functionality for schemas,
//! fields, attributes, and other macro constructs used in netabase.

use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    AttrStyle, ExprClosure, Field, Fields, Ident, Item, ItemEnum, ItemStruct, Meta, Pat,
    ReturnType, Type, TypeParamBound, punctuated::Punctuated, spanned::Spanned,
};

use crate::visitors::errors::NetabaseError;

/// Validation context for schema processing
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub current_item_name: Option<Ident>,
    pub current_item_type: Option<Type>,
    pub strict_mode: bool,
    pub outer_keys_attribute: bool,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            current_item_name: None,
            current_item_type: None,
            strict_mode: true,
            outer_keys_attribute: true,
        }
    }
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current item being validated
    pub fn with_item(mut self, name: Ident, item_type: Option<Type>) -> Self {
        self.current_item_name = Some(name);
        self.current_item_type = item_type;
        self
    }

    /// Enable or disable strict validation mode
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Allow multiple key fields in a schema
    pub fn with_outer_key(mut self, allow: bool) -> Self {
        self.outer_keys_attribute = allow;
        self
    }
}

/// Types of validation that can be performed
#[derive(Debug, Clone, Copy)]
pub enum ValidationType {
    /// Validate that an item is a valid schema
    Schema,
    /// Validate key generation attributes
    KeyGeneration,
    /// Validate field types and attributes
    Field,
    /// Validate generic parameters
    Generic,
}

/// Validation result with detailed information
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<NetabaseError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create a validation result with errors
    pub fn with_errors(errors: Vec<NetabaseError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Add a warning to the validation result
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add a suggestion to the validation result
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Combine two validation results
    pub fn combine(mut self, other: ValidationResult) -> Self {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.suggestions.extend(other.suggestions);
        self
    }
}

/// Main validation trait
pub trait Validator {
    /// Validate the item and return a detailed result
    fn validate(&self, context: &ValidationContext) -> ValidationResult;
}

/// Schema validation utilities
pub struct SchemaValidator;

impl SchemaValidator {
    /// Validate that an item is a proper NetabaseSchema
    pub fn validate_schema_item(item: &Item, context: &ValidationContext) -> ValidationResult {
        match item {
            Item::Struct(item_struct) => Self::validate_struct_schema(item_struct, context),
            Item::Enum(item_enum) => Self::validate_enum_schema(item_enum, context),
            _ => {
                let error =
                    NetabaseError::Schema(crate::visitors::errors::SchemaError::InvalidDerive {
                        item_name: context
                            .current_item_name
                            .clone()
                            .unwrap_or_else(|| Ident::new("Unknown", Span::call_site())),
                        expected: "struct or enum".to_string(),
                        found: "other item type".to_string(),
                        span: item.span(),
                    });
                ValidationResult::with_errors(vec![error])
            }
        }
    }

    /// Validate a struct schema
    pub fn validate_struct_schema(
        item_struct: &ItemStruct,
        context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for NetabaseSchema derive
        if !Self::has_netabase_derive(&item_struct.attrs) {
            let error =
                NetabaseError::Schema(crate::visitors::errors::SchemaError::MissingDerive {
                    item_name: item_struct.ident.clone(),
                    span: item_struct.ident.span(),
                });
            result.errors.push(error);
        }

        // Validate fields
        let field_validation = FieldValidator::validate_fields(&item_struct.fields, context);
        result = result.combine(field_validation);

        // Check for key fields
        let key_fields = FieldValidator::find_key_fields(&item_struct.fields);
        if key_fields.is_empty() {
            let error = NetabaseError::Schema(crate::visitors::errors::SchemaError::NoKeyFields {
                item_name: item_struct.ident.clone(),
                span: item_struct.ident.span(),
            });
            result.errors.push(error);
        } else if key_fields.len() > 1 {
            let field_names: Vec<String> = key_fields
                .iter()
                .filter_map(|field| field.ident.as_ref().map(|i| i.to_string()))
                .collect();
            let error =
                NetabaseError::Schema(crate::visitors::errors::SchemaError::MultipleKeyFields {
                    item_name: item_struct.ident.clone(),
                    field_names,
                    span: item_struct.ident.span(),
                });
            result.errors.push(error);
        }

        result
    }

    /// Validate an enum schema
    pub fn validate_enum_schema(
        item_enum: &ItemEnum,
        context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for NetabaseSchema derive
        if !Self::has_netabase_derive(&item_enum.attrs) {
            let error =
                NetabaseError::Schema(crate::visitors::errors::SchemaError::MissingDerive {
                    item_name: item_enum.ident.clone(),
                    span: item_enum.ident.span(),
                });
            result.errors.push(error);
        }

        // Check if enum has a top-level key closure
        let has_enum_level_closure = item_enum.attrs.iter().any(|attr| {
            matches!(&attr.meta, Meta::NameValue(name_value) if name_value.path.is_ident("key"))
        });

        // Validate each variant
        for variant in &item_enum.variants {
            let variant_context = context.clone().with_item(variant.ident.clone(), None);
            let variant_validation =
                FieldValidator::validate_fields(&variant.fields, &variant_context);
            result = result.combine(variant_validation);

            // Each variant should have exactly one key field, unless there's an enum-level closure
            let key_fields = FieldValidator::find_key_fields(&variant.fields);

            if !has_enum_level_closure {
                if key_fields.is_empty() && !matches!(variant.fields, Fields::Unit) {
                    let error =
                        NetabaseError::Schema(crate::visitors::errors::SchemaError::NoKeyFields {
                            item_name: variant.ident.clone(),
                            span: variant.ident.span(),
                        });
                    result.errors.push(error);
                } else if key_fields.len() > 1 {
                    let field_names: Vec<String> = key_fields
                        .iter()
                        .filter_map(|field| field.ident.as_ref().map(|i| i.to_string()))
                        .collect();
                    let error = NetabaseError::Schema(
                        crate::visitors::errors::SchemaError::MultipleKeyFields {
                            item_name: variant.ident.clone(),
                            field_names,
                            span: variant.ident.span(),
                        },
                    );
                    result.errors.push(error);
                }
            } else {
                // If enum has top-level closure, variants shouldn't have individual key fields
                if !key_fields.is_empty() {
                    result = result.with_warning(format!(
                        "Enum variant '{}' has key fields but enum has top-level key closure - variant keys will be ignored",
                        variant.ident
                    ));
                }
            }
        }

        result
    }

    fn has_outer_keygen(item: &Item) -> Option<&ExprClosure> {
        match item {
            Item::Enum(item_enum) => item_enum.attrs.iter().find_map(|a| {
                if AttrStyle::Outer.eq(&a.style)
                    && let Meta::NameValue(mnv) = &a.meta
                    && mnv.path.is_ident("key")
                    && let syn::Expr::Closure(cl) = &mnv.value
                {
                    Some(cl)
                } else {
                    None
                }
            }),
            Item::Struct(item_struct) => item_struct.attrs.iter().find_map(|a| {
                if AttrStyle::Outer.eq(&a.style)
                    && let Meta::NameValue(mnv) = &a.meta
                    && mnv.path.is_ident("key")
                    && let syn::Expr::Closure(cl) = &mnv.value
                {
                    Some(cl)
                } else {
                    None
                }
            }),
            _ => todo!(),
        }
    }

    /// Check if attributes contain NetabaseSchema derive
    fn has_netabase_derive(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if let AttrStyle::Outer = attr.style
                && let Meta::List(meta_list) = &attr.meta
                && meta_list.path.is_ident("derive")
            {
                return attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("NetabaseSchema") {
                            Ok(())
                        } else {
                            Err(syn::Error::new(meta.path.span(), "not NetabaseSchema"))
                        }
                    })
                    .is_ok();
            }
            false
        })
    }
}

/// Field validation utilities
pub struct FieldValidator;

impl FieldValidator {
    /// Validate all fields in a Fields collection
    pub fn validate_fields(fields: &Fields, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        match fields {
            Fields::Named(fields_named) => {
                for (index, field) in fields_named.named.iter().enumerate() {
                    let field_validation = Self::validate_field(field, index, context);
                    result = result.combine(field_validation);
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                for (index, field) in fields_unnamed.unnamed.iter().enumerate() {
                    let field_validation = Self::validate_field(field, index, context);
                    result = result.combine(field_validation);
                }
            }
            Fields::Unit => {
                // Unit fields are always valid
            }
        }

        result
    }

    /// Validate a single field
    pub fn validate_field(
        field: &Field,
        index: usize,
        context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Validate field type
        let type_validation = TypeValidator::validate_type(&field.ty, context);
        result = result.combine(type_validation);

        // Validate key attributes if present
        if Self::has_key_attribute(field) {
            let key_validation = KeyValidator::validate_key_attribute(field, context);
            result = result.combine(key_validation);
        }

        // Check for field name in named structs
        if field.ident.is_none() && context.strict_mode {
            // This might be expected for tuple structs, so just add a suggestion
            result = result.with_suggestion(format!(
                "Consider using named fields for better readability (field at index {})",
                index
            ));
        }

        result
    }

    /// Find all fields that have key attributes
    pub fn find_key_fields(fields: &Fields) -> Vec<&Field> {
        let mut key_fields = Vec::new();

        match fields {
            Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    if Self::has_key_attribute(field) {
                        key_fields.push(field);
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                for field in &fields_unnamed.unnamed {
                    if Self::has_key_attribute(field) {
                        key_fields.push(field);
                    }
                }
            }
            Fields::Unit => {
                // Unit fields can't have keys
            }
        }

        key_fields
    }

    /// Check if a field has key-related attributes
    pub fn has_key_attribute(field: &Field) -> bool {
        field.attrs.iter().any(|attr| match &attr.meta {
            Meta::Path(path) => path.is_ident("key") || path.is_ident("NetabaseKey"),
            Meta::List(meta_list) => {
                meta_list.path.is_ident("key") || meta_list.path.is_ident("NetabaseKey")
            }
            Meta::NameValue(name_value) => {
                name_value.path.is_ident("key") || name_value.path.is_ident("NetabaseKey")
            }
        })
    }
}

/// Key validation utilities
pub struct KeyValidator;

impl KeyValidator {
    /// Validate key generation attributes on a field
    pub fn validate_key_attribute(field: &Field, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        for attr in &field.attrs {
            match &attr.meta {
                Meta::Path(path) => {
                    if path.is_ident("key") || path.is_ident("NetabaseKey") {
                        // Simple key attribute - field value is used directly
                        let type_validation = Self::validate_key_type(&field.ty, context);
                        result = result.combine(type_validation);
                    }
                }
                Meta::List(meta_list) => {
                    if meta_list.path.is_ident("key") || meta_list.path.is_ident("NetabaseKey") {
                        // Key attribute with parameters - validate the contents
                        result = result.with_suggestion(
                            "Key list attributes are supported but may need additional validation"
                                .to_string(),
                        );
                    }
                }
                Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("key") || name_value.path.is_ident("NetabaseKey") {
                        // Key attribute with closure
                        if let syn::Expr::Closure(closure) = &name_value.value {
                            let closure_validation =
                                Self::validate_key_closure(closure, field, context);
                            result = result.combine(closure_validation);
                        } else {
                            let error = NetabaseError::KeyGeneration(
                                crate::visitors::errors::KeyGenerationError::InvalidKeyAttribute {
                                    field_name: field.ident.as_ref().map(|i| i.to_string()),
                                    attribute: name_value.value.to_token_stream().to_string(),
                                    expected_formats: vec![
                                        "#[key]".to_string(),
                                        "#[key = |field_value| { /* closure body */ }]".to_string(),
                                    ],
                                    span: name_value.span(),
                                },
                            );
                            result.errors.push(error);
                        }
                    }
                }
            }
        }

        result
    }

    /// Validate that a type can be used as a key
    pub fn validate_key_type(key_type: &Type, _context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        // For now, we'll be permissive about key types
        // In a more sophisticated implementation, we could check for specific trait bounds
        match key_type {
            Type::Path(_) => {
                // Most path types should be fine
            }
            Type::Reference(_) => {
                result = result.with_suggestion(
                    "Consider using owned types instead of references for keys".to_string(),
                );
            }
            Type::Tuple(_) => {
                result = result.with_suggestion(
                    "Tuple types may require custom serialization for keys".to_string(),
                );
            }
            _ => {
                result = result.with_warning(format!(
                    "Unusual key type '{}' - ensure it can be serialized",
                    key_type.to_token_stream()
                ));
            }
        }

        result
    }

    /// Validate a key generation closure
    pub fn validate_key_closure(
        closure: &ExprClosure,
        field: &Field,
        context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check closure input count
        if closure.inputs.is_empty() {
            let error = NetabaseError::KeyGeneration(
                crate::visitors::errors::KeyGenerationError::MissingClosureInput {
                    field_name: field.ident.as_ref().map(|i| i.to_string()),
                    span: closure.span(),
                },
            );
            result.errors.push(error);
        } else if closure.inputs.len() > 1 {
            result = result.with_warning(
                "Key closures should typically have only one input parameter".to_string(),
            );
        }

        // Validate closure input type
        if let Some(input) = closure.inputs.first() {
            if let Pat::Type(pat_type) = input {
                let input_validation =
                    Self::validate_closure_input_type(&pat_type.ty, &field.ty, field, context);
                result = result.combine(input_validation);
            }
        }

        // Validate closure return type
        if let ReturnType::Type(_, return_type) = &closure.output {
            let return_validation = Self::validate_closure_return_type(return_type, field, context);
            result = result.combine(return_validation);
        } else {
            result = result.with_suggestion(
                "Consider explicitly specifying the return type for key closures".to_string(),
            );
        }

        result
    }

    /// Validate closure input type matches expectations
    fn validate_closure_input_type(
        closure_input: &Type,
        field_type: &Type,
        field: &Field,
        _context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Simple comparison - in practice, this could be more sophisticated
        if closure_input.to_token_stream().to_string() != field_type.to_token_stream().to_string() {
            // Check if it's a reference to the field type
            if let Type::Reference(type_ref) = closure_input {
                if type_ref.elem.to_token_stream().to_string()
                    == field_type.to_token_stream().to_string()
                {
                    // Reference to field type is acceptable
                    return result;
                }
            }

            let error = NetabaseError::KeyGeneration(
                crate::visitors::errors::KeyGenerationError::ClosureInputTypeMismatch {
                    field_name: field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_default(),
                    field_type: field_type.to_token_stream().to_string(),
                    closure_input_type: closure_input.to_token_stream().to_string(),
                    span: closure_input.span(),
                },
            );
            result.errors.push(error);
        }

        result
    }

    /// Validate closure return type has required bounds
    fn validate_closure_return_type(
        return_type: &Type,
        field: &Field,
        _context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        match return_type {
            Type::ImplTrait(impl_trait) => {
                let has_required_bounds = Self::validate_impl_trait_bounds(&impl_trait.bounds);
                if !has_required_bounds {
                    let error = NetabaseError::KeyGeneration(
                        crate::visitors::errors::KeyGenerationError::InvalidClosureReturnType {
                            field_name: field.ident.as_ref().map(|i| i.to_string()),
                            expected: "impl From<Vec<u8>> + Into<Vec<u8>>".to_string(),
                            found: return_type.to_token_stream().to_string(),
                            span: return_type.span(),
                        },
                    );
                    result.errors.push(error);
                }
            }
            _ => {
                result = result.with_suggestion(
                    "Consider using 'impl From<Vec<u8>> + Into<Vec<u8>>' as the return type"
                        .to_string(),
                );
            }
        }

        result
    }

    /// Validate that impl trait bounds include required key traits
    fn validate_impl_trait_bounds(bounds: &Punctuated<TypeParamBound, syn::Token![+]>) -> bool {
        let has_from = bounds.iter().any(|bound| {
            if let TypeParamBound::Trait(trait_bound) = bound {
                trait_bound
                    .path
                    .segments
                    .last()
                    .map_or(false, |seg| seg.ident == "From")
            } else {
                false
            }
        });

        let has_into = bounds.iter().any(|bound| {
            if let TypeParamBound::Trait(trait_bound) = bound {
                trait_bound
                    .path
                    .segments
                    .last()
                    .map_or(false, |seg| seg.ident == "Into")
            } else {
                false
            }
        });

        has_from && has_into
    }
}

/// Type validation utilities
pub struct TypeValidator;

impl TypeValidator {
    /// Validate a type for use in netabase schemas
    pub fn validate_type(ty: &Type, _context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        match ty {
            Type::Path(type_path) => {
                // Most path types are fine, but we could add specific checks
                if Self::is_potentially_problematic_type(&type_path.path) {
                    result = result.with_suggestion(format!(
                        "Type '{}' may need special handling for serialization",
                        type_path.path.to_token_stream()
                    ));
                }
            }
            Type::Reference(_) => {
                result = result.with_warning(
                    "Reference types in schema fields may cause lifetime issues".to_string(),
                );
            }
            Type::Ptr(_) => {
                result = result.with_warning(
                    "Raw pointer types are not recommended for schema fields".to_string(),
                );
            }
            Type::BareFn(_) => {
                result = result
                    .with_warning("Function pointer types may not be serializable".to_string());
            }
            Type::Never(_) => {
                let error = NetabaseError::Type(
                    crate::visitors::errors::TypeValidationError::InvalidKeyType {
                        type_name: "!".to_string(),
                        span: ty.span(),
                    },
                );
                result.errors.push(error);
            }
            _ => {
                // Other types are generally acceptable
            }
        }

        result
    }

    /// Check if a type path represents a potentially problematic type
    fn is_potentially_problematic_type(path: &syn::Path) -> bool {
        if let Some(last_segment) = path.segments.last() {
            matches!(
                last_segment.ident.to_string().as_str(),
                "Rc" | "Arc" | "RefCell" | "Mutex" | "RwLock" | "Box"
            )
        } else {
            false
        }
    }
}

/// Utility functions for common validation tasks
pub mod utils {
    use super::*;

    /// Quick validation for a schema item
    pub fn quick_validate_schema(item: &Item) -> bool {
        let context = ValidationContext::new();
        let result = SchemaValidator::validate_schema_item(item, &context);
        result.is_valid
    }

    /// Get all validation errors for a schema
    pub fn get_schema_errors(item: &Item) -> Vec<NetabaseError> {
        let context = ValidationContext::new();
        let result = SchemaValidator::validate_schema_item(item, &context);
        result.errors
    }

    /// Check if a field can be used as a key
    pub fn is_valid_key_field(field: &Field) -> bool {
        let context = ValidationContext::new();
        let result = KeyValidator::validate_key_attribute(field, &context);
        result.is_valid
    }

    /// Get suggestions for improving a schema
    pub fn get_schema_suggestions(item: &Item) -> Vec<String> {
        let context = ValidationContext::new();
        let result = SchemaValidator::validate_schema_item(item, &context);
        result.suggestions
    }
}
