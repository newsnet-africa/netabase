//! Example usage demonstrating the separation between visitors and generators
//!
//! This module shows how to use the SchemaCounterVisitor to collect schema information
//! and then use the SchemaEnumGenerator to generate enums from that data.

use crate::generators::SchemaEnumGenerator;
use crate::visitors::SchemaCounterVisitor;
use syn::{parse_quote, visit::Visit};

/// Example function showing the complete workflow
pub fn example_schema_processing() {
    // Step 1: Parse some example module content
    let module: syn::ItemMod = parse_quote! {
        mod schemas {
            #[derive(NetabaseSchema)]
            pub struct User {
                #[key]
                pub id: u64,
                pub name: String,
            }

            #[derive(NetabaseSchemaKey)]
            pub struct UserKey {
                pub id: u64,
            }

            mod posts {
                #[derive(NetabaseSchema)]
                pub struct Post {
                    #[key]
                    pub id: u64,
                    pub title: String,
                    pub content: String,
                }

                #[derive(NetabaseSchemaKey)]
                pub struct PostKey {
                    pub id: u64,
                }
            }
        }
    };

    // Step 2: Use the visitor to collect schema information
    let mut visitor = SchemaCounterVisitor::new();
    visitor.visit_item_mod(&module);

    // Step 3: Get the collected schemas
    let schemas = visitor.get_schemas();
    println!("Found {} schemas", schemas.len());

    // Step 4: Use the generator to create enums
    let generator = SchemaEnumGenerator::new(schemas);

    // Generate individual enums
    let schemas_enum = generator.generate_schemas_enum("AllSchemas");
    let keys_enum = generator.generate_schema_keys_enum("AllSchemaKeys");

    // Or generate both at once
    let both_enums = generator.generate_both_enums("AllSchemas", "AllSchemaKeys");

    println!("Generated schemas enum: {}", schemas_enum);
    println!("Generated keys enum: {}", keys_enum);
}

/// Example of reusing the visitor for multiple modules
pub fn example_multiple_modules() {
    let mut visitor = SchemaCounterVisitor::new();

    // Process first module
    let module1: syn::ItemMod = parse_quote! {
        mod users {
            #[derive(NetabaseSchema)]
            pub struct User {
                #[key] pub id: u64,
            }

            #[derive(NetabaseSchemaKey)]
            pub struct UserKey {
                pub id: u64,
            }
        }
    };

    visitor.visit_item_mod(&module1);

    // Process second module
    let module2: syn::ItemMod = parse_quote! {
        mod orders {
            #[derive(NetabaseSchema)]
            pub struct Order {
                #[key] pub id: u64,
            }

            #[derive(NetabaseSchemaKey)]
            pub struct OrderKey {
                pub id: u64,
            }
        }
    };

    visitor.visit_item_mod(&module2);

    // Generate combined enums for all collected schemas
    let generator = SchemaEnumGenerator::new(visitor.get_schemas());
    let combined_enums = generator.generate_both_enums("AllSchemas", "AllKeys");

    println!("Combined enums from multiple modules: {}", combined_enums);

    // Clear the visitor for reuse
    visitor.clear();
    assert_eq!(visitor.get_schemas().len(), 0);
}

/// Example showing error handling and edge cases
pub fn example_edge_cases() {
    // Empty module
    let empty_module: syn::ItemMod = parse_quote! {
        mod empty {}
    };

    let mut visitor = SchemaCounterVisitor::new();
    visitor.visit_item_mod(&empty_module);

    let generator = SchemaEnumGenerator::new(visitor.get_schemas());
    let empty_schemas_enum = generator.generate_schemas_enum("EmptySchemas");

    // This will generate an enum with a comment indicating no schemas were found
    println!("Empty enum: {}", empty_schemas_enum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_generator_integration() {
        let module: syn::ItemMod = parse_quote! {
            mod test_schemas {
                #[derive(NetabaseSchema)]
                pub struct TestStruct {
                    #[key] pub id: u64,
                }

                #[derive(NetabaseSchemaKey)]
                pub struct TestStructKey {
                    pub id: u64,
                }
            }
        };

        let mut visitor = SchemaCounterVisitor::new();
        visitor.visit_item_mod(&module);

        assert_eq!(visitor.get_schemas().len(), 1);

        let generator = SchemaEnumGenerator::new(visitor.get_schemas());
        let schemas_enum = generator.generate_schemas_enum("TestSchemas");
        let keys_enum = generator.generate_schema_keys_enum("TestKeys");

        let schemas_str = schemas_enum.to_string();
        let keys_str = keys_enum.to_string();

        assert!(schemas_str.contains("pub enum TestSchemas"));
        assert!(keys_str.contains("pub enum TestKeys"));
    }

    #[test]
    fn test_visitor_reusability() {
        let mut visitor = SchemaCounterVisitor::new();

        // First use
        let module1: syn::ItemMod = parse_quote! {
            mod mod1 {
                #[derive(NetabaseSchema)]
                pub struct Schema1 {
                    #[key] pub id: u64,
                }

                #[derive(NetabaseSchemaKey)]
                pub struct Schema1Key {
                    pub id: u64,
                }
            }
        };

        visitor.visit_item_mod(&module1);
        assert_eq!(visitor.get_schemas().len(), 1);

        // Clear and reuse
        visitor.clear();
        assert_eq!(visitor.get_schemas().len(), 0);

        // Second use
        let module2: syn::ItemMod = parse_quote! {
            mod mod2 {
                #[derive(NetabaseSchema)]
                pub struct Schema2 {
                    #[key] pub id: u64,
                }

                #[derive(NetabaseSchemaKey)]
                pub struct Schema2Key {
                    pub id: u64,
                }
            }
        };

        visitor.visit_item_mod(&module2);
        assert_eq!(visitor.get_schemas().len(), 1);
    }
}
