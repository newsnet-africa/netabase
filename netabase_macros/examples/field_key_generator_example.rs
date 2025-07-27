//! Example demonstrating the FieldKeyGenerator struct and its ToTokens implementation
//!
//! This example shows how to create FieldKeyGenerator instances with different key generation
//! strategies and how they convert to properly parameterized key generator functions.

use netabase_macros::visitors::{FieldKeyGenerator, KeyGenerator};
use quote::ToTokens;
use syn::parse_quote;

fn main() {
    println!("=== FieldKeyGenerator Examples ===\n");

    // Example 1: Direct field access
    direct_field_access_example();

    // Example 2: Field closure
    field_closure_example();

    // Example 3: Item closure
    item_closure_example();

    // Example 4: Handling empty fields
    empty_fields_example();
}

/// Example 1: Direct field access - uses the field value directly as the key
fn direct_field_access_example() {
    println!("1. Direct Field Access Example:");
    println!("   Input: struct User {{ #[key] id: String, name: String }}");

    // Create a struct with fields
    let item_struct: syn::ItemStruct = parse_quote! {
        struct ExampleUserForField {
            #[key]
            id: String,
            name: String,
        }
    };

    // Get the key field
    let key_field: syn::Field = parse_quote! { id: String };

    // Create FieldKeyGenerator with direct field access
    let key_gen = KeyGenerator::Field(&key_field);
    let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);

    // Generate the key function
    let mut tokens = proc_macro2::TokenStream::new();
    field_key_gen.to_tokens(&mut tokens);

    println!("   Generated function:");
    println!("   {}", tokens);
    println!("   → Creates: pub fn key(item: &String) -> impl From<Vec<u8>> + Into<Vec<u8>>");
    println!();
}

/// Example 2: Field closure - uses a closure that transforms the field value
fn field_closure_example() {
    println!("2. Field Closure Example:");
    println!("   Input: #[key = |id| format!(\"user_{{}}\", id)] id: u64");

    // Create a struct with fields
    let item_struct: syn::ItemStruct = parse_quote! {
        struct ExampleUserForClosure {
            #[key]
            id: u64,
            name: String,
        }
    };

    // Create a closure that transforms the field
    let closure: syn::ExprClosure = parse_quote! { |id| format!("user_{}", id) };
    let key_gen = KeyGenerator::FieldClosure(&closure);
    let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);

    // Generate the key function
    let mut tokens = proc_macro2::TokenStream::new();
    field_key_gen.to_tokens(&mut tokens);

    println!("   Generated function:");
    println!("   {}", tokens);
    println!("   → Creates: pub fn key(item: &u64) -> impl From<Vec<u8>> + Into<Vec<u8>>");
    println!("   → Applies closure to transform the field value");
    println!();
}

/// Example 3: Item closure - uses a closure that takes the entire item
fn item_closure_example() {
    println!("3. Item Closure Example:");
    println!("   Input: #[key = |product| product.category + &product.id] on Product struct");

    // Create a struct with fields
    let item_struct: syn::ItemStruct = parse_quote! {
        struct ExampleProductForItem {
            id: String,
            category: String,
            price: f64,
        }
    };

    // Create an item-level closure
    let closure: syn::ExprClosure = parse_quote! {
        |product| format!("{}_{}", product.category, product.id)
    };
    let item_type: syn::Type = parse_quote! { ExampleProductForItem };

    let key_gen = KeyGenerator::ItemClosure(&closure);
    let field_key_gen = FieldKeyGenerator::with_item_type(&item_struct.fields, key_gen, &item_type);

    // Generate the key function
    let mut tokens = proc_macro2::TokenStream::new();
    field_key_gen.to_tokens(&mut tokens);

    println!("   Generated function:");
    println!("   {}", tokens);
    println!(
        "   → Creates: pub fn key(item: &ExampleProductForItem) -> impl From<Vec<u8>> + Into<Vec<u8>>"
    );
    println!("   → Takes entire item and applies closure to extract key");
    println!();
}

/// Example 4: Handling empty fields - generates fallback function
fn empty_fields_example() {
    println!("4. Empty Fields Example:");
    println!("   Input: Unit struct or struct with no accessible fields");

    // Create empty fields
    let empty_fields = syn::Fields::Unit;
    let dummy_field: syn::Field = parse_quote! { id: String };
    let key_gen = KeyGenerator::Field(&dummy_field);
    let field_key_gen = FieldKeyGenerator::new(&empty_fields, key_gen);

    // Generate the key function
    let mut tokens = proc_macro2::TokenStream::new();
    field_key_gen.to_tokens(&mut tokens);

    println!("   Generated function:");
    println!("   {}", tokens);
    println!(
        "   → Creates fallback: pub fn key<T>(item: &T) -> impl From<Vec<u8>> + Into<Vec<u8>>"
    );
    println!("   → Returns empty Vec when no fields are available");
    println!();
}

/// Utility function to show the complete workflow
#[allow(dead_code)]
fn complete_workflow_example() {
    println!("=== Complete Workflow ===");

    // 1. Parse a struct from code
    let struct_code = r#"
        struct ExampleBlogPost {
            #[key]
            slug: String,
            title: String,
            content: String,
        }
    "#;

    let item_struct: syn::ItemStruct = syn::parse_str(struct_code).unwrap();

    // 2. Extract the key field
    let key_field = item_struct
        .fields
        .iter()
        .find(|field| field.attrs.iter().any(|attr| attr.path().is_ident("key")))
        .unwrap();

    // 3. Create FieldKeyGenerator
    let key_gen = KeyGenerator::Field(key_field);
    let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);

    // 4. Generate the function
    let mut tokens = proc_macro2::TokenStream::new();
    field_key_gen.to_tokens(&mut tokens);

    println!("Complete workflow result:");
    println!("{}", tokens);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_field_access() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestUser { id: String, name: String }
        };
        let key_field: syn::Field = parse_quote! { id: String };
        let key_gen = KeyGenerator::Field(&key_field);
        let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);

        let mut tokens = proc_macro2::TokenStream::new();
        field_key_gen.to_tokens(&mut tokens);

        // Should generate a function with proper signature
        assert!(!tokens.is_empty());
        let token_string = tokens.to_string();
        assert!(token_string.contains("pub fn"));
        assert!(token_string.contains("impl From < Vec < u8 > > + Into < Vec < u8 > >"));
    }

    #[test]
    fn test_field_closure() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestUserClosure { id: u64, name: String }
        };
        let closure: syn::ExprClosure = parse_quote! { |id| format!("user_{}", id) };
        let key_gen = KeyGenerator::FieldClosure(&closure);
        let field_key_gen = FieldKeyGenerator::new(&item_struct.fields, key_gen);

        let mut tokens = proc_macro2::TokenStream::new();
        field_key_gen.to_tokens(&mut tokens);

        // Should generate a function that applies the closure
        assert!(!tokens.is_empty());
        let token_string = tokens.to_string();
        assert!(token_string.contains("pub fn"));
    }

    #[test]
    fn test_item_closure() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestProduct { id: String, category: String }
        };
        let closure: syn::ExprClosure = parse_quote! { |product| product.id.clone() };
        let item_type: syn::Type = parse_quote! { TestProduct };
        let key_gen = KeyGenerator::ItemClosure(&closure);
        let field_key_gen =
            FieldKeyGenerator::with_item_type(&item_struct.fields, key_gen, &item_type);

        let mut tokens = proc_macro2::TokenStream::new();
        field_key_gen.to_tokens(&mut tokens);

        // Should generate a function that takes the item type
        assert!(!tokens.is_empty());
        let token_string = tokens.to_string();
        assert!(token_string.contains("pub fn"));
        assert!(token_string.contains("TestProduct"));
    }
}
