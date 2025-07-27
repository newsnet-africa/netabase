//! Error handling for netabase macro processing
//!
//! This module provides structured error types and utilities for reporting
//! compilation errors during macro expansion with helpful diagnostics.

use proc_macro2::Span;
use syn::{Error as SynError, ExprClosure, Field, Ident, Type, spanned::Spanned};

/// Main error type for netabase macro processing
#[derive(Debug)]
pub enum NetabaseError {
    /// Schema validation errors
    Schema(SchemaError),
    /// Key generation errors
    KeyGeneration(KeyGenerationError),
    /// Type validation errors
    Type(TypeValidationError),
    /// Generic parameter errors
    Generic(GenericError),
    /// Field processing errors
    Field(FieldError),
    /// Conversion errors for TryFrom implementations
    Conversion(ConversionError),
}

#[derive(Debug)]
pub enum SchemaError {
    /// Missing NetabaseSchema derive attribute
    MissingDerive { item_name: Ident, span: Span },
    /// Invalid derive attribute format
    InvalidDerive {
        item_name: Ident,
        expected: String,
        found: String,
        span: Span,
    },
    /// No key fields found in schema
    NoKeyFields { item_name: Ident, span: Span },
    /// Multiple key fields found (when only one expected)
    MultipleKeyFields {
        item_name: Ident,
        field_names: Vec<String>,
        span: Span,
    },
}

#[derive(Debug)]
pub enum KeyGenerationError {
    /// Invalid closure signature for key generation
    InvalidClosureSignature {
        field_name: Option<String>,
        expected: String,
        found: String,
        span: Span,
    },
    /// Missing required closure input parameter
    MissingClosureInput {
        field_name: Option<String>,
        span: Span,
    },
    /// Invalid closure return type
    InvalidClosureReturnType {
        field_name: Option<String>,
        expected: String,
        found: String,
        span: Span,
    },
    /// Closure input type doesn't match field type
    ClosureInputTypeMismatch {
        field_name: String,
        field_type: String,
        closure_input_type: String,
        span: Span,
    },
    /// Invalid key attribute format
    InvalidKeyAttribute {
        field_name: Option<String>,
        attribute: String,
        expected_formats: Vec<String>,
        span: Span,
    },
}

#[derive(Debug)]
pub enum TypeValidationError {
    /// Type doesn't implement required trait
    MissingTraitImplementation {
        type_name: String,
        trait_name: String,
        span: Span,
    },
    /// Invalid type for key generation
    InvalidKeyType { type_name: String, span: Span },
    /// Unsupported generic type
    UnsupportedGeneric { type_name: String, span: Span },
}

#[derive(Debug)]
pub enum GenericError {
    /// Missing generic parameter bounds
    MissingBounds {
        param_name: String,
        required_bounds: Vec<String>,
        span: Span,
    },
    /// Invalid where clause
    InvalidWhereClause { clause: String, span: Span },
    /// Conflicting generic constraints
    ConflictingConstraints {
        param_name: String,
        conflict_description: String,
        span: Span,
    },
}

#[derive(Debug)]
pub enum ConversionError {
    /// UTF-8 conversion error when converting record key
    InvalidUtf8Key { error_message: String, span: Span },
    /// Deserialization error when converting record value
    DeserializationError {
        type_name: String,
        error_message: String,
        span: Span,
    },
    /// Invalid record format
    InvalidRecordFormat {
        expected: String,
        found: String,
        span: Span,
    },
    /// Missing required data in record
    MissingData { field_name: String, span: Span },
}

#[derive(Debug)]
pub enum FieldError {
    /// Field has no identifier (for named fields)
    MissingFieldName { field_index: usize, span: Span },
    /// Invalid field type
    InvalidFieldType {
        field_name: Option<String>,
        type_name: String,
        reason: String,
        span: Span,
    },
    /// Field attribute parsing error
    AttributeParsingError {
        field_name: Option<String>,
        attribute_name: String,
        error_message: String,
        span: Span,
    },
}

impl NetabaseError {
    /// Convert to syn::Error for proc macro diagnostics
    pub fn to_syn_error(self) -> SynError {
        match self {
            NetabaseError::Schema(err) => err.to_syn_error(),
            NetabaseError::KeyGeneration(err) => err.to_syn_error(),
            NetabaseError::Type(err) => err.to_syn_error(),
            NetabaseError::Generic(err) => err.to_syn_error(),
            NetabaseError::Field(err) => err.to_syn_error(),
            NetabaseError::Conversion(err) => err.to_syn_error(),
        }
    }
}

impl SchemaError {
    pub fn to_syn_error(self) -> SynError {
        match self {
            SchemaError::MissingDerive { item_name, span } => SynError::new(
                span,
                format!(
                    "Schema '{}' must derive NetabaseSchema. Add #[derive(NetabaseSchema)] to the item.",
                    item_name
                ),
            ),
            SchemaError::InvalidDerive {
                item_name,
                expected,
                found,
                span,
            } => SynError::new(
                span,
                format!(
                    "Invalid derive attribute for schema '{}'. Expected: {}, Found: {}",
                    item_name, expected, found
                ),
            ),
            SchemaError::NoKeyFields { item_name, span } => SynError::new(
                span,
                format!(
                    "Schema '{}' has no key fields. Each schema must have at least one field marked with #[key] or #[key = closure]",
                    item_name
                ),
            ),
            SchemaError::MultipleKeyFields {
                item_name,
                field_names,
                span,
            } => SynError::new(
                span,
                format!(
                    "Schema '{}' has multiple key fields: [{}]. Structs can have at most 1 key field, enum variants can have at most 1 key field.",
                    item_name,
                    field_names.join(", ")
                ),
            ),
        }
    }
}

impl KeyGenerationError {
    pub fn to_syn_error(self) -> SynError {
        match self {
            KeyGenerationError::InvalidClosureSignature {
                field_name,
                expected,
                found,
                span,
            } => {
                let field_part = field_name
                    .map(|name| format!(" for field '{}'", name))
                    .unwrap_or_default();
                SynError::new(
                    span,
                    format!(
                        "Invalid closure signature{}. Expected: {}, Found: {}",
                        field_part, expected, found
                    ),
                )
            }
            KeyGenerationError::MissingClosureInput { field_name, span } => {
                let field_part = field_name
                    .map(|name| format!(" for field '{}'", name))
                    .unwrap_or_default();
                SynError::new(
                    span,
                    format!(
                        "Key generation closure{} must have exactly one input parameter",
                        field_part
                    ),
                )
            }
            KeyGenerationError::InvalidClosureReturnType {
                field_name,
                expected,
                found,
                span,
            } => {
                let field_part = field_name
                    .map(|name| format!(" for field '{}'", name))
                    .unwrap_or_default();
                SynError::new(
                    span,
                    format!(
                        "Invalid closure return type{}. Expected: {}, Found: {}. The closure must return a type that implements both From<Vec<u8>> and Into<Vec<u8>>",
                        field_part, expected, found
                    ),
                )
            }
            KeyGenerationError::ClosureInputTypeMismatch {
                field_name,
                field_type,
                closure_input_type,
                span,
            } => SynError::new(
                span,
                format!(
                    "Closure input type mismatch for field '{}'. Field type: {}, Closure input type: {}. The closure input must match the field type.",
                    field_name, field_type, closure_input_type
                ),
            ),
            KeyGenerationError::InvalidKeyAttribute {
                field_name,
                attribute,
                expected_formats,
                span,
            } => {
                let field_part = field_name
                    .map(|name| format!(" for field '{}'", name))
                    .unwrap_or_default();
                SynError::new(
                    span,
                    format!(
                        "Invalid key attribute{}: '{}'. Expected one of: [{}]",
                        field_part,
                        attribute,
                        expected_formats.join(", ")
                    ),
                )
            }
        }
    }
}

impl TypeValidationError {
    pub fn to_syn_error(self) -> SynError {
        match self {
            TypeValidationError::MissingTraitImplementation {
                type_name,
                trait_name,
                span,
            } => SynError::new(
                span,
                format!(
                    "Type '{}' must implement '{}' to be used as a key type",
                    type_name, trait_name
                ),
            ),
            TypeValidationError::InvalidKeyType { type_name, span } => SynError::new(
                span,
                format!(
                    "Type '{}' cannot be used as a key type. Key types must implement serialization traits",
                    type_name
                ),
            ),
            TypeValidationError::UnsupportedGeneric { type_name, span } => SynError::new(
                span,
                format!(
                    "Generic type '{}' is not supported in this context",
                    type_name
                ),
            ),
        }
    }
}

impl GenericError {
    pub fn to_syn_error(self) -> SynError {
        match self {
            GenericError::MissingBounds {
                param_name,
                required_bounds,
                span,
            } => SynError::new(
                span,
                format!(
                    "Generic parameter '{}' is missing required bounds: [{}]",
                    param_name,
                    required_bounds.join(", ")
                ),
            ),
            GenericError::InvalidWhereClause { clause, span } => {
                SynError::new(span, format!("Invalid where clause: '{}'", clause))
            }
            GenericError::ConflictingConstraints {
                param_name,
                conflict_description,
                span,
            } => SynError::new(
                span,
                format!(
                    "Conflicting constraints for generic parameter '{}': {}",
                    param_name, conflict_description
                ),
            ),
        }
    }
}

impl FieldError {
    pub fn to_syn_error(self) -> SynError {
        match self {
            FieldError::MissingFieldName { field_index, span } => SynError::new(
                span,
                format!(
                    "Field at index {} is missing a name. Named structs require all fields to have names.",
                    field_index
                ),
            ),
            FieldError::InvalidFieldType {
                field_name,
                type_name,
                reason,
                span,
            } => {
                let field_part = field_name
                    .map(|name| format!("Field '{}' has", name))
                    .unwrap_or_else(|| "Field has".to_string());
                SynError::new(
                    span,
                    format!("{} invalid type '{}': {}", field_part, type_name, reason),
                )
            }
            FieldError::AttributeParsingError {
                field_name,
                attribute_name,
                error_message,
                span,
            } => {
                let field_part = field_name
                    .map(|name| format!(" for field '{}'", name))
                    .unwrap_or_default();
                SynError::new(
                    span,
                    format!(
                        "Error parsing attribute '{}'{}: {}",
                        attribute_name, field_part, error_message
                    ),
                )
            }
        }
    }
}

impl ConversionError {
    pub fn to_syn_error(self) -> SynError {
        match self {
            ConversionError::InvalidUtf8Key {
                error_message,
                span,
            } => SynError::new(
                span,
                format!("Failed to convert record key to UTF-8: {}", error_message),
            ),
            ConversionError::DeserializationError {
                type_name,
                error_message,
                span,
            } => SynError::new(
                span,
                format!(
                    "Failed to deserialize {} from record: {}",
                    type_name, error_message
                ),
            ),
            ConversionError::InvalidRecordFormat {
                expected,
                found,
                span,
            } => SynError::new(
                span,
                format!(
                    "Invalid record format. Expected: {}, Found: {}",
                    expected, found
                ),
            ),
            ConversionError::MissingData { field_name, span } => SynError::new(
                span,
                format!("Missing required data for field '{}'", field_name),
            ),
        }
    }
}

/// Utility functions for creating common errors
pub mod utils {
    use super::*;
    use quote::ToTokens;

    /// Create a schema error for missing derive attribute
    pub fn missing_derive_error(item_name: &Ident) -> NetabaseError {
        NetabaseError::Schema(SchemaError::MissingDerive {
            item_name: item_name.clone(),
            span: item_name.span(),
        })
    }

    /// Create a key generation error for invalid closure signature
    pub fn invalid_closure_signature_error(
        closure: &ExprClosure,
        field_name: Option<&str>,
    ) -> NetabaseError {
        let expected = "fn(FieldType) -> impl From<Vec<u8>> + Into<Vec<u8>>";
        let found = closure.to_token_stream().to_string();

        NetabaseError::KeyGeneration(KeyGenerationError::InvalidClosureSignature {
            field_name: field_name.map(|s| s.to_string()),
            expected: expected.to_string(),
            found,
            span: closure.span(),
        })
    }

    /// Create a field error for missing field name
    pub fn missing_field_name_error(field: &Field, index: usize) -> NetabaseError {
        NetabaseError::Field(FieldError::MissingFieldName {
            field_index: index,
            span: field.ty.span(),
        })
    }

    /// Create a type validation error for missing trait implementation
    pub fn missing_trait_implementation_error(
        type_name: &str,
        trait_name: &str,
        span: Span,
    ) -> NetabaseError {
        NetabaseError::Type(TypeValidationError::MissingTraitImplementation {
            type_name: type_name.to_string(),
            trait_name: trait_name.to_string(),
            span,
        })
    }

    /// Create a schema error for no key fields
    pub fn no_key_fields_error(item_name: &Ident) -> NetabaseError {
        NetabaseError::Schema(SchemaError::NoKeyFields {
            item_name: item_name.clone(),
            span: item_name.span(),
        })
    }

    /// Create a key generation error for closure input type mismatch
    pub fn closure_input_type_mismatch_error(
        field_name: &str,
        field_type: &Type,
        closure_input_type: &Type,
        span: Span,
    ) -> NetabaseError {
        NetabaseError::KeyGeneration(KeyGenerationError::ClosureInputTypeMismatch {
            field_name: field_name.to_string(),
            field_type: field_type.to_token_stream().to_string(),
            closure_input_type: closure_input_type.to_token_stream().to_string(),
            span,
        })
    }

    /// Create a conversion error for invalid UTF-8 key
    pub fn invalid_utf8_key_error(error_message: &str, span: Span) -> NetabaseError {
        NetabaseError::Conversion(ConversionError::InvalidUtf8Key {
            error_message: error_message.to_string(),
            span,
        })
    }

    /// Create a conversion error for deserialization failure
    pub fn deserialization_error(
        type_name: &str,
        error_message: &str,
        span: Span,
    ) -> NetabaseError {
        NetabaseError::Conversion(ConversionError::DeserializationError {
            type_name: type_name.to_string(),
            error_message: error_message.to_string(),
            span,
        })
    }

    /// Create a conversion error for invalid record format
    pub fn invalid_record_format_error(expected: &str, found: &str, span: Span) -> NetabaseError {
        NetabaseError::Conversion(ConversionError::InvalidRecordFormat {
            expected: expected.to_string(),
            found: found.to_string(),
            span,
        })
    }

    /// Create a conversion error for missing data
    pub fn missing_data_error(field_name: &str, span: Span) -> NetabaseError {
        NetabaseError::Conversion(ConversionError::MissingData {
            field_name: field_name.to_string(),
            span,
        })
    }
}

/// Result type for macro operations
pub type NetabaseResult<T> = std::result::Result<T, NetabaseError>;

/// Trait for converting errors to compile errors
pub trait ToCompileError {
    fn to_compile_error(self) -> proc_macro2::TokenStream;
}

impl ToCompileError for NetabaseError {
    fn to_compile_error(self) -> proc_macro2::TokenStream {
        self.to_syn_error().to_compile_error()
    }
}

impl ToCompileError for SynError {
    fn to_compile_error(self) -> proc_macro2::TokenStream {
        syn::Error::to_compile_error(&self)
    }
}
