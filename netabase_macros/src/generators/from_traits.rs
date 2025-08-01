//! From trait generation for netabase schemas
//!
//! This module generates From/Into trait implementations for converting between
//! user schemas and libp2p kad records.

use proc_macro2::TokenStream;
use quote::quote;

use crate::generators::key_struct::generate_key_struct_name;
use crate::visitors::{
    utils::schema_finder::SchemaType,
    utils::{FieldKeyInfo, KeyType},
};
use std::collections::HashMap;
use syn::Variant;

/// Generate From<libp2p::kad::Record> for Schema
pub fn generate_record_to_schema(schema_type: &SchemaType) -> TokenStream {
    let schema_name = schema_type.identity();

    quote! {
        impl From<libp2p::kad::Record> for #schema_name {
            fn from(record: libp2p::kad::Record) -> Self {
                // Deserialize the value bytes back into our schema
                bincode::decode_from_slice(&record.value, bincode::config::standard())
                    .map(|(decoded, _)| decoded)
                    .unwrap_or_else(|e| {
                        panic!("Failed to deserialize {} from record: {}", stringify!(#schema_name), e)
                    })
            }
        }
    }
}

/// Generate From<Schema> for libp2p::kad::Record
pub fn generate_schema_to_record(schema_type: &SchemaType, key_type: &KeyType) -> TokenStream {
    let schema_name = schema_type.identity();
    let key_extraction = generate_key_extraction_for_record(key_type, schema_type);

    quote! {
        impl From<#schema_name> for libp2p::kad::Record {
            fn from(schema: #schema_name) -> Self {
                // Serialize the schema to bytes
                let value = bincode::encode_to_vec(&schema, bincode::config::standard())
                    .unwrap_or_else(|e| {
                        panic!("Failed to serialize {} to bytes: {}", stringify!(#schema_name), e)
                    });

                // Extract the key
                let key_bytes = {
                    #key_extraction
                };

                // Create the record
                libp2p::kad::Record {
                    key: libp2p::kad::RecordKey::new(&key_bytes),
                    value,
                    publisher: None,
                    expires: None,
                }
            }
        }
    }
}

/// Generate From<&Schema> for libp2p::kad::Record (reference version)
pub fn generate_schema_ref_to_record(schema_type: &SchemaType, key_type: &KeyType) -> TokenStream {
    let schema_name = schema_type.identity();
    let key_extraction = generate_key_extraction_for_record_ref(key_type, schema_type);

    quote! {
        impl From<&#schema_name> for libp2p::kad::Record {
            fn from(schema: &#schema_name) -> Self {
                // Serialize the schema to bytes
                let value = bincode::encode_to_vec(schema, bincode::config::standard())
                    .unwrap_or_else(|e| {
                        panic!("Failed to serialize {} to bytes: {}", stringify!(#schema_name), e)
                    });

                // Extract the key
                let key_bytes = {
                    #key_extraction
                };

                // Create the record
                libp2p::kad::Record {
                    key: libp2p::kad::RecordKey::new(&key_bytes),
                    value,
                    publisher: None,
                    expires: None,
                }
            }
        }
    }
}

/// Generate key extraction code for owned schema -> record conversion
fn generate_key_extraction_for_record(key_type: &KeyType, schema_type: &SchemaType) -> TokenStream {
    match key_type {
        KeyType::FieldKeys(field_map) => {
            generate_field_key_extraction_for_record(field_map, schema_type, false)
        }
        KeyType::SchemaKey(closure) => generate_closure_key_extraction_for_record(closure, false),
        KeyType::KeyFunction(func_name) => {
            generate_function_key_extraction_for_record(func_name, false)
        }
    }
}

/// Generate key extraction code for reference schema -> record conversion
fn generate_key_extraction_for_record_ref(
    key_type: &KeyType,
    schema_type: &SchemaType,
) -> TokenStream {
    match key_type {
        KeyType::FieldKeys(field_map) => {
            generate_field_key_extraction_for_record(field_map, schema_type, true)
        }
        KeyType::SchemaKey(closure) => generate_closure_key_extraction_for_record(closure, true),
        KeyType::KeyFunction(func_name) => {
            generate_function_key_extraction_for_record(func_name, true)
        }
    }
}

/// Generate field-based key extraction for record conversion
fn generate_field_key_extraction_for_record(
    field_map: &HashMap<Option<&Variant>, FieldKeyInfo>,
    schema_type: &SchemaType,
    is_reference: bool,
) -> TokenStream {
    let schema_ref = if is_reference {
        quote!(schema)
    } else {
        quote!(&schema)
    };

    match schema_type {
        SchemaType::Struct(_) => {
            // For structs, we should have exactly one field key
            if let Some((None, field_info)) = field_map.iter().next() {
                if let Some(field_name) = field_info.field.ident.as_ref() {
                    quote! {
                        bincode::encode_to_vec(&#schema_ref.#field_name, bincode::config::standard())
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
                                #schema_ref::#variant_name { #field_name, .. } => {
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
                                #schema_ref::#variant_name(#(#field_patterns),*, ..) => {
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
                match #schema_ref {
                    #(#match_arms)*
                }
            }
        }
    }
}

/// Generate closure-based key extraction for record conversion
fn generate_closure_key_extraction_for_record(
    closure: &syn::ExprClosure,
    is_reference: bool,
) -> TokenStream {
    let closure_body = &closure.body;
    let schema_arg = if is_reference {
        quote!(schema)
    } else {
        quote!(&schema)
    };

    quote! {
        {
            let key_value = (#closure)(#schema_arg);
            bincode::encode_to_vec(&key_value, bincode::config::standard())
                .unwrap_or_else(|_| vec![])
        }
    }
}

/// Generate function-based key extraction for record conversion
fn generate_function_key_extraction_for_record(func_name: &str, is_reference: bool) -> TokenStream {
    let func_ident = syn::Ident::new(func_name, proc_macro2::Span::call_site());
    let schema_arg = if is_reference {
        quote!(schema)
    } else {
        quote!(&schema)
    };

    quote! {
        {
            let key_value = #func_ident(#schema_arg);
            bincode::encode_to_vec(&key_value, bincode::config::standard())
                .unwrap_or_else(|_| vec![])
        }
    }
}

/// Generate all From trait implementations for a schema
pub fn generate_all_record_conversions(
    schema_type: &SchemaType,
    key_type: &KeyType,
) -> TokenStream {
    let record_to_schema = generate_record_to_schema(schema_type);
    let schema_to_record = generate_schema_to_record(schema_type, key_type);
    let schema_ref_to_record = generate_schema_ref_to_record(schema_type, key_type);

    quote! {
        #record_to_schema
        #schema_to_record
        #schema_ref_to_record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visitors::utils::FieldKeyInfo;
    use std::collections::HashMap;
    use syn::parse_quote;

    #[test]
    fn test_record_to_schema_generation() {
        let item: syn::Item = parse_quote! {
            struct User {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let generated = generate_record_to_schema(&schema_type);

        let generated_str = generated.to_string();
        assert!(generated_str.contains("impl From<libp2p::kad::Record> for User"));
        assert!(generated_str.contains("bincode::decode_from_slice"));
    }

    #[test]
    fn test_schema_to_record_generation() {
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

        let generated = generate_schema_to_record(&schema_type, &key_type);

        let generated_str = generated.to_string();
        assert!(generated_str.contains("impl From<User> for libp2p::kad::Record"));
        assert!(generated_str.contains("bincode::encode_to_vec"));
        assert!(generated_str.contains("libp2p::kad::Record"));
    }
}
