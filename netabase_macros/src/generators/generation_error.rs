use quote::quote;
use thiserror::Error;

/// Generation errors that occur during macro expansion and code generation
/// These are compile-time errors that help diagnose issues with the generated code
#[derive(Error, Debug)]
pub enum GenerationError {
    #[error("Code generation failed: {message}")]
    CodeGeneration { message: String },

    #[error("Bincode conversion error: {0}")]
    BincodeConversion(#[from] BincodeGenerationError),

    #[error("TryFrom conversion error: {0}")]
    TryFromConversion(#[from] TryFromGenerationError),

    #[error("Template expansion failed for {template}: {reason}")]
    TemplateExpansion { template: String, reason: String },

    #[error("Type inference failed for {type_name}: {reason}")]
    TypeInference { type_name: String, reason: String },

    #[error("Generic parameter resolution failed for {param}: {constraint}")]
    GenericResolution { param: String, constraint: String },

    #[error("Trait bound validation failed: {bound} on {type_name}")]
    TraitBoundError { bound: String, type_name: String },

    #[error("Implementation generation failed for {impl_type}: {reason}")]
    ImplGeneration { impl_type: String, reason: String },

    #[error("Key generation failed: {key_type}")]
    KeyGeneration { key_type: String },

    #[error("Schema enum generation failed: {enum_name}")]
    SchemaEnumGeneration { enum_name: String },

    #[error("Function signature generation failed: {function_name}")]
    FunctionGeneration { function_name: String },

    #[error("Derive macro expansion failed: {macro_name}")]
    DeriveMacroError { macro_name: String },

    #[error("Token stream processing error: {context}")]
    TokenStreamError { context: String },
}

/// Bincode-specific generation errors for macro-generated conversion code
#[derive(Error, Debug)]
pub enum BincodeGenerationError {
    #[error("Bincode encoding generation failed: {type_name}")]
    EncodingGeneration { type_name: String },

    #[error("Bincode decoding generation failed: {type_name}")]
    DecodingGeneration { type_name: String },

    #[error("Bincode configuration generation error: {config_type}")]
    ConfigurationGeneration { config_type: String },

    #[error("Buffer handling generation failed for {operation}")]
    BufferHandlingGeneration { operation: String },

    #[error("Serialization format generation incompatible with type: {type_name}")]
    SerializationIncompatibility { type_name: String },

    #[error("Bincode feature requirements not met: {required_features:?}")]
    MissingFeatures { required_features: Vec<String> },

    #[error("Bincode version compatibility issue: expected {expected}, generating for {actual}")]
    VersionCompatibility { expected: String, actual: String },

    #[error("Complex type serialization not supported: {type_info}")]
    UnsupportedComplexType { type_info: String },
}

/// TryFrom conversion generation errors for macro-generated conversion code
#[derive(Error, Debug)]
pub enum TryFromGenerationError {
    #[error("TryFrom implementation generation failed: {from} -> {to}")]
    ImplementationGeneration { from: String, to: String },

    #[error("Record key conversion generation failed: {key_type}")]
    RecordKeyGeneration { key_type: String },

    #[error("Record value conversion generation failed: {value_type}")]
    RecordValueGeneration { value_type: String },

    #[error("Error type generation failed for conversion: {conversion_name}")]
    ErrorTypeGeneration { conversion_name: String },

    #[error("Pattern matching generation failed in conversion: {pattern}")]
    PatternMatchingGeneration { pattern: String },

    #[error("Field access generation failed: {field_path}")]
    FieldAccessGeneration { field_path: String },

    #[error("Method call generation failed: {method_name}")]
    MethodCallGeneration { method_name: String },

    #[error("Generic constraint generation failed: {constraint}")]
    GenericConstraintGeneration { constraint: String },

    #[error("Lifetime parameter generation failed: {lifetime}")]
    LifetimeGeneration { lifetime: String },

    #[error("Associated type generation failed: {assoc_type}")]
    AssociatedTypeGeneration { assoc_type: String },
}

/// Helper trait for converting generation errors into compile-time errors
pub trait IntoCompileError {
    fn into_compile_error(self) -> proc_macro2::TokenStream;
    fn into_compile_error_with_span(self, span: proc_macro2::Span) -> proc_macro2::TokenStream;
}

impl IntoCompileError for GenerationError {
    fn into_compile_error(self) -> proc_macro2::TokenStream {
        let message = format!("Netabase Generation Error: {}", self);
        quote! {
            compile_error!(#message);
        }
    }

    fn into_compile_error_with_span(self, span: proc_macro2::Span) -> proc_macro2::TokenStream {
        let message = format!("Netabase Generation Error: {}", self);
        quote::quote_spanned! {span=>
            compile_error!(#message);
        }
    }
}

impl IntoCompileError for BincodeGenerationError {
    fn into_compile_error(self) -> proc_macro2::TokenStream {
        let message = format!("Bincode Generation Error: {}", self);
        quote! {
            compile_error!(#message);
        }
    }

    fn into_compile_error_with_span(self, span: proc_macro2::Span) -> proc_macro2::TokenStream {
        let message = format!("Bincode Generation Error: {}", self);
        quote::quote_spanned! {span=>
            compile_error!(#message);
        }
    }
}

impl IntoCompileError for TryFromGenerationError {
    fn into_compile_error(self) -> proc_macro2::TokenStream {
        let message = format!("TryFrom Generation Error: {}", self);
        quote! {
            compile_error!(#message);
        }
    }

    fn into_compile_error_with_span(self, span: proc_macro2::Span) -> proc_macro2::TokenStream {
        let message = format!("TryFrom Generation Error: {}", self);
        quote::quote_spanned! {span=>
            compile_error!(#message);
        }
    }
}

/// Extension trait for Results to add generation-specific error context
pub trait GenerationContext<T> {
    fn with_generation_context(self, message: &str) -> Result<T, GenerationError>;
    fn with_bincode_context(self, type_name: &str) -> Result<T, GenerationError>;
    fn with_tryfrom_context(self, source: &str, target: &str) -> Result<T, GenerationError>;
    fn with_template_context(self, template: &str) -> Result<T, GenerationError>;
}

impl<T, E> GenerationContext<T> for Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn with_generation_context(self, message: &str) -> Result<T, GenerationError> {
        self.map_err(|e| GenerationError::CodeGeneration {
            message: format!("{}: {}", message, e),
        })
    }

    fn with_bincode_context(self, type_name: &str) -> Result<T, GenerationError> {
        self.map_err(|e| {
            GenerationError::BincodeConversion(BincodeGenerationError::EncodingGeneration {
                type_name: format!("{}: {}", type_name, e),
            })
        })
    }

    fn with_tryfrom_context(self, source: &str, target: &str) -> Result<T, GenerationError> {
        self.map_err(|e| {
            GenerationError::TryFromConversion(TryFromGenerationError::ImplementationGeneration {
                from: source.to_string(),
                to: format!("{}: {}", target, e),
            })
        })
    }

    fn with_template_context(self, template: &str) -> Result<T, GenerationError> {
        self.map_err(|e| GenerationError::TemplateExpansion {
            template: template.to_string(),
            reason: e.to_string(),
        })
    }
}

/// Builder for constructing detailed generation errors
pub struct GenerationErrorBuilder {
    error_type: String,
    context: Vec<String>,
    suggestions: Vec<String>,
}

impl GenerationErrorBuilder {
    pub fn new(error_type: &str) -> Self {
        Self {
            error_type: error_type.to_string(),
            context: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.context.push(context.to_string());
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestions.push(suggestion.to_string());
        self
    }

    pub fn build(self) -> GenerationError {
        let mut message = self.error_type;

        if !self.context.is_empty() {
            message.push_str("\nContext: ");
            message.push_str(&self.context.join(", "));
        }

        if !self.suggestions.is_empty() {
            message.push_str("\nSuggestions: ");
            message.push_str(&self.suggestions.join(", "));
        }

        GenerationError::CodeGeneration { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_error_display() {
        let err = GenerationError::CodeGeneration {
            message: "test error".to_string(),
        };
        assert_eq!(err.to_string(), "Code generation failed: test error");
    }

    #[test]
    fn test_bincode_error_display() {
        let err = BincodeGenerationError::EncodingGeneration {
            type_name: "MyStruct".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Bincode encoding generation failed: MyStruct"
        );
    }

    #[test]
    fn test_tryfrom_error_display() {
        let err = TryFromGenerationError::ImplementationGeneration {
            from: "String".to_string(),
            to: "i32".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "TryFrom implementation generation failed: String -> i32"
        );
    }

    #[test]
    fn test_error_builder() {
        let err = GenerationErrorBuilder::new("Test error")
            .with_context("in function foo")
            .with_suggestion("try using bar instead")
            .build();

        let message = err.to_string();
        assert!(message.contains("Test error"));
        assert!(message.contains("in function foo"));
        assert!(message.contains("try using bar instead"));
    }

    #[test]
    fn test_compile_error_generation() {
        let err = GenerationError::CodeGeneration {
            message: "test".to_string(),
        };
        let compile_error = err.into_compile_error();
        let tokens_str = compile_error.to_string();
        assert!(tokens_str.contains("compile_error"));
        assert!(tokens_str.contains("Netabase Generation Error"));
    }
}
