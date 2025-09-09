# Netabase Error Handling System

This document describes the comprehensive error handling functionality implemented for the Netabase macro system using `thiserror`.

## Overview

The Netabase macro system includes two primary categories of errors:

1. **Visitor Errors** - For invalid API usage that should be propagated to users
2. **Generation Errors** - For compile-time issues in generated code, especially bincode and TryFrom conversions

## Error Categories

### 1. Visitor Errors (`VisitError`)

These errors occur during the validation phase when users provide invalid macro usage. They help users understand and fix their code.

```rust
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
    
    // ... more variants
}
```

#### Key Errors (`KeyError`)

Specific errors related to key validation:

```rust
#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Too many keys found in schema definition")]
    TooManyKeys,
    
    #[error("No key found in schema definition")]
    KeyNotFound,
    
    #[error("Inner key validation failed: {0}")]
    InnerKeyError(#[from] InnerKeyError),
    
    #[error("Outer key validation failed: {0}")]
    OuterKeyError(#[from] OuterKeyError),
    
    // ... more variants
}
```

### 2. Generation Errors (`GenerationError`)

These errors occur during macro expansion and code generation, providing detailed diagnostics for compile-time issues.

```rust
#[derive(Error, Debug)]
pub enum GenerationError {
    #[error("Code generation failed: {message}")]
    CodeGeneration { message: String },
    
    #[error("Bincode conversion error: {0}")]
    BincodeConversion(#[from] BincodeGenerationError),
    
    #[error("TryFrom conversion error: {0}")]
    TryFromConversion(#[from] TryFromGenerationError),
    
    #[error("Key generation failed: {key_type}")]
    KeyGeneration { key_type: String },
    
    // ... more variants
}
```

#### Bincode Generation Errors (`BincodeGenerationError`)

Specific to bincode serialization/deserialization issues in generated code:

```rust
#[derive(Error, Debug)]
pub enum BincodeGenerationError {
    #[error("Bincode encoding generation failed: {type_name}")]
    EncodingGeneration { type_name: String },
    
    #[error("Bincode decoding generation failed: {type_name}")]
    DecodingGeneration { type_name: String },
    
    #[error("Serialization format incompatible with type: {type_name}")]
    SerializationIncompatibility { type_name: String },
    
    // ... more variants
}
```

#### TryFrom Generation Errors (`TryFromGenerationError`)

Specific to TryFrom conversion issues in generated code:

```rust
#[derive(Error, Debug)]
pub enum TryFromGenerationError {
    #[error("TryFrom implementation generation failed: {from} -> {to}")]
    ImplementationGeneration { from: String, to: String },
    
    #[error("Record key conversion generation failed: {key_type}")]
    RecordKeyGeneration { key_type: String },
    
    #[error("Record value conversion generation failed: {value_type}")]
    RecordValueGeneration { value_type: String },
    
    // ... more variants
}
```

## Usage Examples

### Basic Error Handling in Macros

```rust
#[proc_macro_derive(NetabaseSchema)]
pub fn netabase_schema_derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let mut vi = SchemaValidator::default();
    vi.visit_derive_input(&inp);

    match generate_netabase_macro(vi) {
        Ok(net_impl) => quote! {
            #net_impl
        }.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
```

### Error Context and Chaining

```rust
// Error chaining for better context
let inner_error = InnerKeyError::InnerKeyNotFound;
let key_error = KeyError::InnerKeyError(inner_error);
let visit_error = VisitError::KeyError(key_error);

// The full error chain is preserved:
// VisitError::KeyError(KeyError::InnerKeyError(InnerKeyError::InnerKeyNotFound))
```

### Converting Errors to Compile Errors

```rust
impl IntoCompileError for GenerationError {
    fn into_compile_error(self) -> proc_macro2::TokenStream {
        let message = format!("Netabase Generation Error: {}", self);
        quote! {
            compile_error!(#message);
        }
    }
}
```

## Error Context Extensions

The system provides extension traits for adding context to errors:

### For Visitor Errors

```rust
pub trait ErrorContext<T> {
    fn with_visitor_context(self, message: &str) -> Result<T, VisitError>;
    fn with_key_context(self, key_info: &str) -> Result<T, VisitError>;
    fn with_schema_context(self, schema_name: &str) -> Result<T, VisitError>;
}

// Usage:
result.with_schema_context("UserSchema")?;
```

### For Generation Errors

```rust
pub trait GenerationContext<T> {
    fn with_generation_context(self, message: &str) -> Result<T, GenerationError>;
    fn with_bincode_context(self, type_name: &str) -> Result<T, GenerationError>;
    fn with_tryfrom_context(self, source: &str, target: &str) -> Result<T, GenerationError>;
}

// Usage:
result.with_bincode_context("MyStruct")?;
```

## Error Builder Pattern

For complex error construction:

```rust
let error = GenerationErrorBuilder::new("Complex operation failed")
    .with_context("in function generate_key_impl")
    .with_suggestion("ensure the key field has a valid type")
    .build();
```

## Best Practices

### 1. Error Propagation

Always use `?` operator or proper error handling:

```rust
fn generate_something() -> Result<TokenStream, GenerationError> {
    let key_item = generate_key()?; // Proper error propagation
    let impl_item = generate_impl()?;
    
    Ok(quote! {
        #key_item
        #impl_item
    })
}
```

### 2. Meaningful Error Messages

Provide specific, actionable error messages:

```rust
// Good
GenerationError::KeyGeneration {
    key_type: format!("Outer key for '{}' missing return type annotation", schema_name),
}

// Bad
GenerationError::KeyGeneration {
    key_type: "Error".to_string(),
}
```

### 3. Error Context

Add context when converting between error types:

```rust
let ident = input.ident().map_err(|e| GenerationError::ImplGeneration {
    impl_type: "NetabaseSchema".to_string(),
    reason: format!("Failed to get identifier: {}", e),
})?;
```

### 4. Compile Error Generation

Always provide helpful compile errors:

```rust
match result {
    Ok(tokens) => tokens.into(),
    Err(err) => err.into_compile_error().into(),
}
```

## Testing Error Handling

```rust
#[test]
fn test_error_chaining() {
    let inner_error = InnerKeyError::InnerKeyNotFound;
    let key_error = KeyError::InnerKeyError(inner_error);
    let visit_error = VisitError::KeyError(key_error);
    
    // Test error message contains expected text
    assert!(visit_error.to_string().contains("Inner key field not found"));
    
    // Test error source chain
    assert!(visit_error.source().is_some());
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
```

## Integration with Existing Code

The error system is fully integrated into the macro system:

1. **Visitor Phase**: Uses `VisitError` and its variants
2. **Generation Phase**: Uses `GenerationError` and its variants
3. **Macro Output**: Converts errors to `compile_error!` tokens

All existing panic scenarios have been replaced with proper error handling that provides actionable feedback to users.

## File Structure

```
netabase_macros/src/
├── visitors/
│   └── validation_error.rs    # Visitor error types
├── generators/
│   └── generation_error.rs    # Generation error types
└── lib.rs                     # Error handling integration
```

## Future Extensibility

The error system is designed to be extensible:

1. Add new error variants to existing enums
2. Create new error categories as needed
3. Extend context traits for additional functionality
4. Add new compile error generation strategies

The use of `thiserror` ensures consistent error handling patterns and automatic `std::error::Error` trait implementations.