use quote::quote;
use thiserror::Error;

/// Visitor errors for invalid API usage that should be propagated to the user
#[derive(Error, Debug)]
pub enum VisitError {
    #[error("Registry is not a valid schema type")]
    RegistryNotSchema,

    #[error("Key validation failed: {0}")]
    KeyError(#[from] KeyError),

    #[error("Syntax parsing failed: {0}")]
    ParseError(#[from] syn::Error),

    #[error("Invalid schema type. Only structs and enums are allowed")]
    InvalidSchemaType,

    #[error("Schema validation failed: {message}")]
    SchemaValidation { message: String },

    #[error("Attribute parsing failed: {attribute}")]
    AttributeError { attribute: String },

    #[error("Multiple conflicting attributes found")]
    ConflictingAttributes,

    #[error("Required attribute missing: {attribute}")]
    MissingAttribute { attribute: String },
}

/// Key-specific validation errors
#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Too many keys found in schema definition")]
    TooManyKeys,

    #[error("No key found in schema definition")]
    KeyNotFound,

    #[error("Invalid schema structure for key generation")]
    InvalidSchema,

    #[error("Inner key validation failed: {0}")]
    InnerKeyError(#[from] InnerKeyError),

    #[error("Outer key validation failed: {0}")]
    OuterKeyError(#[from] OuterKeyError),

    #[error("Key type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Key field access error: {field}")]
    FieldAccessError { field: String },
}

/// Outer key specific errors
#[derive(Error, Debug)]
pub enum OuterKeyError {
    #[error("Outer key function not found")]
    OuterKeyNotFound,

    #[error("Return type annotation missing for key function")]
    ReturnTypeNotFound,

    #[error("Function receiver (self parameter) not found")]
    ArgumentReceiverNotFound,

    #[error("Invalid function signature for key generation")]
    InvalidSignature,

    #[error("Key function must be public")]
    NonPublicKeyFunction,
}

/// Inner key specific errors
#[derive(Error, Debug)]
pub enum InnerKeyError {
    #[error("Inner key field not found")]
    InnerKeyNotFound,

    #[error("Key must be the first item in tuple struct")]
    KeyNotFirstTupleItem,

    #[error("Named field key must have valid identifier")]
    InvalidFieldIdentifier,

    #[error("Tuple field key access failed")]
    TupleFieldAccessError,

    #[error("Key field type is not supported")]
    UnsupportedKeyFieldType,
}

/// Helper trait for converting visitor errors into TokenStream compilation errors
pub trait IntoCompileError {
    fn into_compile_error(self) -> proc_macro2::TokenStream;
}

impl IntoCompileError for VisitError {
    fn into_compile_error(self) -> proc_macro2::TokenStream {
        let message = format!("Netabase Visitor Error: {}", self);
        quote! {
            compile_error!(#message);
        }
    }
}

/// Extension trait for Result types to provide better error context for visitor errors
pub trait ErrorContext<T> {
    fn with_visitor_context(self, message: &str) -> Result<T, VisitError>;
    fn with_key_context(self, key_info: &str) -> Result<T, VisitError>;
    fn with_schema_context(self, schema_name: &str) -> Result<T, VisitError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn with_visitor_context(self, message: &str) -> Result<T, VisitError> {
        self.map_err(|e| VisitError::SchemaValidation {
            message: format!("{}: {}", message, e),
        })
    }

    fn with_key_context(self, key_info: &str) -> Result<T, VisitError> {
        self.map_err(|_| {
            VisitError::KeyError(KeyError::FieldAccessError {
                field: key_info.to_string(),
            })
        })
    }

    fn with_schema_context(self, schema_name: &str) -> Result<T, VisitError> {
        self.map_err(|e| VisitError::SchemaValidation {
            message: format!("Schema '{}': {}", schema_name, e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let visit_err = VisitError::InvalidSchemaType;
        assert_eq!(
            visit_err.to_string(),
            "Invalid schema type. Only structs and enums are allowed"
        );

        let key_err = KeyError::TooManyKeys;
        assert_eq!(
            key_err.to_string(),
            "Too many keys found in schema definition"
        );

        let gen_err = GenerationError::CodeGeneration {
            message: "test".to_string(),
        };
        assert_eq!(gen_err.to_string(), "Code generation failed: test");
    }

    #[test]
    fn test_error_chaining() {
        let inner_err = InnerKeyError::InnerKeyNotFound;
        let key_err = KeyError::InnerKeyError(inner_err);
        let visit_err = VisitError::KeyError(key_err);

        assert!(visit_err.to_string().contains("Inner key field not found"));
    }
}
