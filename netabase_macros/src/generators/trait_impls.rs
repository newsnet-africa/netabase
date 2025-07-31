//! Trait implementations generator for netabase schemas
//!
//! This module generates implementations of NetabaseSchema and NetabaseSchemaKey traits
//! for user-defined schemas and their corresponding key structs.

use proc_macro2::TokenStream;
use quote::quote;

use crate::generators::key_struct::generate_key_struct_name;
use crate::visitors::{
    schema_finder::SchemaType,
    utils::{FieldKeyInfo, KeyType},
};
use std::collections::HashMap;
use syn::Variant;

/// Generate NetabaseSchema trait implementation for a schema
pub fn generate_netabase_schema_impl(schema_type: &SchemaType, key_type: &KeyType) -> TokenStream {
    let schema_name = schema_type.identity();
    let key_struct_name = generate_key_struct_name(schema_name);
    let key_extraction = generate_key_method_body(key_type, schema_type);

    quote! {
        impl crate::NetabaseSchema for #schema_name {
            type Key = #key_struct_name;

            fn key(&self) -> Self::Key {
                let key_bytes = {
                    #key_extraction
                };
                #key_struct_name::new(key_bytes)
            }
        }

    }
}

/// Generate NetabaseSchemaKey trait implementation for a key struct
pub fn generate_netabase_schema_key_impl(schema_type: &SchemaType) -> TokenStream {
    let key_struct_name = generate_key_struct_name(schema_type.identity());

    quote! {
        impl crate::NetabaseSchemaKey for #key_struct_name {
            // This trait is currently a marker trait with no required methods
        }
    }
}

/// Generate the key extraction method body for the key() function
fn generate_key_method_body(key_type: &KeyType, schema_type: &SchemaType) -> TokenStream {
    match key_type {
        KeyType::FieldKeys(field_map) => {
            generate_field_key_extraction_method(field_map, schema_type)
        }
        KeyType::SchemaKey(closure) => generate_closure_key_extraction_method(closure),
        KeyType::KeyFunction(func_name) => generate_function_key_extraction_method(func_name),
    }
}

/// Generate field-based key extraction for the key() method
fn generate_field_key_extraction_method(
    field_map: &HashMap<Option<&Variant>, FieldKeyInfo>,
    schema_type: &SchemaType,
) -> TokenStream {
    match schema_type {
        SchemaType::Struct(_) => {
            // For structs, we should have exactly one field key
            if let Some((None, field_info)) = field_map.iter().next() {
                if let Some(field_name) = field_info.field.ident.as_ref() {
                    quote! {
                        bincode::encode_to_vec(&self.#field_name, bincode::config::standard())
                            .unwrap_or_else(|_| vec![])
                    }
                } else {
                    quote! {
                        compile_error!("Field keys require named fields")
                    }
                }
            } else {
                quote! {
                    compile_error!("Struct must have exactly one key field")
                }
            }
        }
        SchemaType::Enum(_) => {
            // For enums, generate a match expression
            let match_arms: Vec<TokenStream> = field_map
                .iter()
                .filter_map(|(variant_opt, field_info)| {
                    if let Some(variant) = variant_opt {
                        let variant_name = &variant.ident;

                        // Handle both named and tuple variants
                        if let Some(field_name) = field_info.field.ident.as_ref() {
                            // Named variant: Variant { field_name, .. }
                            Some(quote! {
                                Self::#variant_name { #field_name, .. } => {
                                    bincode::encode_to_vec(#field_name, bincode::config::standard())
                                        .unwrap_or_else(|_| vec![])
                                }
                            })
                        } else if let Some(index) = field_info.index {
                            // Tuple variant: generate pattern with correct field position
                            let field_patterns: Vec<TokenStream> = (0..index + 1)
                                .map(|i| {
                                    if i == index {
                                        quote! { key_field }
                                    } else {
                                        quote! { _ }
                                    }
                                })
                                .collect();

                            Some(quote! {
                                Self::#variant_name(#(#field_patterns),*, ..) => {
                                    bincode::encode_to_vec(key_field, bincode::config::standard())
                                        .unwrap_or_else(|_| vec![])
                                }
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if match_arms.is_empty() {
                return quote! {
                    compile_error!("Enum must have at least one variant with a key field")
                };
            }

            quote! {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

/// Generate closure-based key extraction for the key() method
fn generate_closure_key_extraction_method(closure: &syn::ExprClosure) -> TokenStream {
    let closure_body = &closure.body;

    quote! {
        {
            let key_value = (#closure)(self);
            bincode::encode_to_vec(&key_value, bincode::config::standard())
                .unwrap_or_else(|_| vec![])
        }
    }
}

/// Generate function-based key extraction for the key() method
fn generate_function_key_extraction_method(func_name: &str) -> TokenStream {
    let func_ident = syn::Ident::new(func_name, proc_macro2::Span::call_site());

    quote! {
        {
            let key_value = #func_ident(self);
            bincode::encode_to_vec(&key_value, bincode::config::standard())
                .unwrap_or_else(|_| vec![])
        }
    }
}

/// Generate both trait implementations for a schema
pub fn generate_all_trait_impls(schema_type: &SchemaType, key_type: &KeyType) -> TokenStream {
    let schema_impl = generate_netabase_schema_impl(schema_type, key_type);
    let key_impl = generate_netabase_schema_key_impl(schema_type);

    quote! {
        #schema_impl
        #key_impl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visitors::utils::FieldKeyInfo;
    use std::collections::HashMap;
    use syn::parse_quote;

    #[test]
    fn test_netabase_schema_impl_generation() {
        let item: syn::Item = parse_quote! {
            struct User {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();

        // Create a simple field key type for testing
        let mut field_map = HashMap::new();
        let field: syn::Field = parse_quote!(id: u64);
        let field_info = FieldKeyInfo {
            field: &field,
            index: None,
        };
        field_map.insert(None, field_info);
        let key_type = KeyType::FieldKeys(field_map);

        let generated = generate_netabase_schema_impl(&schema_type, &key_type);

        let generated_str = generated.to_string();
        assert!(generated_str.contains("impl crate::NetabaseSchema for User"));
        assert!(generated_str.contains("type Key = UserKey"));
        assert!(generated_str.contains("fn key(&self) -> Self::Key"));
    }

    #[test]
    fn test_netabase_schema_key_impl_generation() {
        let item: syn::Item = parse_quote! {
            struct User {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let generated = generate_netabase_schema_key_impl(&schema_type);

        let generated_str = generated.to_string();
        assert!(generated_str.contains("impl crate::NetabaseSchemaKey for UserKey"));
    }

    #[test]
    fn test_closure_key_extraction() {
        let closure: syn::ExprClosure = parse_quote!(|item| item.id);
        let generated = generate_closure_key_extraction_method(&closure);

        let generated_str = generated.to_string();
        assert!(generated_str.contains("(| item | item . id)(self)"));
        assert!(generated_str.contains("bincode::encode_to_vec"));
    }

    #[test]
    fn test_function_key_extraction() {
        let generated = generate_function_key_extraction_method("get_user_key");

        let generated_str = generated.to_string();
        assert!(generated_str.contains("get_user_key(self)"));
        assert!(generated_str.contains("bincode::encode_to_vec"));
    }
}
