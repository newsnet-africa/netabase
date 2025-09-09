//! # Error Handling Demonstration
//!
//! This example demonstrates the comprehensive error handling functionality
//! created for the Netabase macro system using `thiserror`.

use netabase_macros::{
    generators::{
        BincodeGenerationError, GenerationError, IntoCompileError, TryFromGenerationError,
    },
    visitors::validation_error::{InnerKeyError, KeyError, OuterKeyError, VisitError},
};

fn main() {
    println!("=== Netabase Error Handling Demo ===\n");

    // Demonstrate visitor errors (for invalid API usage)
    demonstrate_visitor_errors();

    // Demonstrate generation errors (for compile-time issues)
    demonstrate_generation_errors();

    // Demonstrate error chaining
    demonstrate_error_chaining();

    // Demonstrate compile error generation
    demonstrate_compile_error_generation();
}

fn demonstrate_visitor_errors() {
    println!("1. Visitor Errors (Invalid API Usage):");
    println!("   These errors help users fix their macro usage\n");

    // Key validation errors
    let key_error = KeyError::TooManyKeys;
    println!("   KeyError: {}", key_error);

    let outer_error = OuterKeyError::ReturnTypeNotFound;
    println!("   OuterKeyError: {}", outer_error);

    let inner_error = InnerKeyError::InnerKeyNotFound;
    println!("   InnerKeyError: {}", inner_error);

    let visit_error = VisitError::InvalidSchemaType;
    println!("   VisitError: {}", visit_error);

    println!();
}

fn demonstrate_generation_errors() {
    println!("2. Generation Errors (Compile-time Issues):");
    println!("   These errors help diagnose generated code problems\n");

    // Code generation error
    let gen_error = GenerationError::CodeGeneration {
        message: "Failed to generate NetabaseSchema implementation".to_string(),
    };
    println!("   GenerationError: {}", gen_error);

    // Bincode conversion error
    let bincode_error = BincodeGenerationError::EncodingGeneration {
        type_name: "ComplexStruct".to_string(),
    };
    println!("   BincodeGenerationError: {}", bincode_error);

    // TryFrom conversion error
    let tryfrom_error = TryFromGenerationError::RecordValueGeneration {
        value_type: "UserDefinedType".to_string(),
    };
    println!("   TryFromGenerationError: {}", tryfrom_error);

    println!();
}

fn demonstrate_error_chaining() {
    println!("3. Error Chaining:");
    println!("   Errors can be chained for better context\n");

    // Chain inner key error -> key error -> visit error
    let inner_error = InnerKeyError::InnerKeyNotFound;
    let key_error = KeyError::InnerKeyError(inner_error);
    let visit_error = VisitError::KeyError(key_error);

    println!("   Chained error: {}", visit_error);

    // Demonstrate source chain
    let mut source = visit_error.source();
    let mut level = 1;
    while let Some(err) = source {
        println!("   └─ Level {}: {}", level, err);
        source = err.source();
        level += 1;
    }

    println!();
}

fn demonstrate_compile_error_generation() {
    println!("4. Compile Error Generation:");
    println!("   Errors can be converted to compile-time errors\n");

    let generation_error = GenerationError::KeyGeneration {
        key_type: "Invalid key configuration detected".to_string(),
    };

    // Convert to compile error tokens (this would be used in macro context)
    let compile_tokens = generation_error.into_compile_error();
    println!("   Generated compile error tokens:");
    println!("   {}", compile_tokens);

    println!();
}

// Example of how errors would be used in actual macro code
#[allow(dead_code)]
fn example_macro_usage() -> Result<(), GenerationError> {
    // Simulate bincode conversion error
    simulate_bincode_operation().map_err(|_| {
        GenerationError::BincodeConversion(BincodeGenerationError::EncodingGeneration {
            type_name: "ExampleStruct".to_string(),
        })
    })?;

    // Simulate TryFrom conversion error
    simulate_tryfrom_operation().map_err(|_| {
        GenerationError::TryFromConversion(TryFromGenerationError::ImplementationGeneration {
            from: "Record".to_string(),
            to: "ExampleStruct".to_string(),
        })
    })?;

    Ok(())
}

#[allow(dead_code)]
fn simulate_bincode_operation() -> Result<(), &'static str> {
    Err("Bincode encoding failed")
}

#[allow(dead_code)]
fn simulate_tryfrom_operation() -> Result<(), &'static str> {
    Err("TryFrom conversion failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let visit_error = VisitError::InvalidSchemaType;
        assert!(visit_error.to_string().contains("Invalid schema type"));

        let gen_error = GenerationError::CodeGeneration {
            message: "test".to_string(),
        };
        assert!(gen_error.to_string().contains("Code generation failed"));
    }

    #[test]
    fn test_error_source_chain() {
        let inner_error = InnerKeyError::InnerKeyNotFound;
        let key_error = KeyError::InnerKeyError(inner_error);
        let visit_error = VisitError::KeyError(key_error);

        // Test that the error chain is maintained
        assert!(visit_error.source().is_some());

        let source = visit_error.source().unwrap();
        assert!(source.to_string().contains("Inner key validation failed"));
    }

    #[test]
    fn test_compile_error_generation() {
        let error = GenerationError::KeyGeneration {
            key_type: "test".to_string(),
        };

        let tokens = error.into_compile_error();
        let tokens_str = tokens.to_string();

        assert!(tokens_str.contains("compile_error"));
        assert!(tokens_str.contains("Netabase Generation Error"));
    }
}
