use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItemFn, ItemImpl, parse_quote};

use crate::{
    SchemaValidator,
    generators::{GenerationError, generation_error::TryFromGenerationError},
    visitors::Key,
};

use self::netabase_schema_key::{generate_key_getter, generate_netabase_key};

pub fn generate_netabase_macro(input: SchemaValidator) -> Result<TokenStream, GenerationError> {
    let key_item = generate_netabase_key(
        input.key().map_err(|e| GenerationError::KeyGeneration {
            key_type: format!("Failed to get key from validator: {}", e),
        })?,
        input.ident().map_err(|e| GenerationError::KeyGeneration {
            key_type: format!("Failed to get identifier from validator: {}", e),
        })?,
    )?;

    let key = input.key().map_err(|e| GenerationError::KeyGeneration {
        key_type: format!("Failed to get key reference: {}", e),
    })?;

    let k_fun = generate_key_getter(&key_item, key)?;
    let impl_item = generate_netabase_impl(&input, k_fun)?;
    let from_to_record = generate_from_to_record(&input)?;
    match key {
        Key::Registry(attribute) => Ok(quote! {
            #impl_item
            #from_to_record
        }),
        _ => Ok(quote! {
            #key_item
            #impl_item
            #from_to_record
        }),
    }
}

pub fn generate_netabase_impl(
    input: &SchemaValidator,
    key: ImplItemFn,
) -> Result<ItemImpl, GenerationError> {
    let ident = input.ident().map_err(|e| GenerationError::ImplGeneration {
        impl_type: "NetabaseSchema".to_string(),
        reason: format!("Failed to get identifier: {}", e),
    })?;

    let key_ident = Key::ident(ident);
    Ok(parse_quote! {
        impl<R: netabase::netabase_trait::NetabaseRegistery> netabase::netabase_trait::NetabaseSchema<R> for #ident {
            type Key = #key_ident;
            #key
        }
    })
}

pub fn generate_from_to_record(
    input: &SchemaValidator,
) -> Result<proc_macro2::TokenStream, GenerationError> {
    let ident = input.ident().map_err(|e| {
        GenerationError::TryFromConversion(TryFromGenerationError::RecordValueGeneration {
            value_type: format!("Failed to get identifier: {}", e),
        })
    })?;

    Ok(quote! {
        impl TryFrom<::macro_exports::__netabase_libp2p_kad::Record> for #ident {
            type Error = ::macro_exports::__netabase_anyhow::Error;
            fn try_from(value: ::macro_exports::__netabase_libp2p_kad::Record) -> Result<Self, ::macro_exports::__netabase_anyhow::Error> {
                let bytes = value.value;
                Ok(::macro_exports::__netabase_bincode::decode_from_slice(&bytes, ::macro_exports::__netabase_bincode_config::standard())?.0)
            }
        }

        impl TryFrom<#ident> for ::macro_exports::__netabase_libp2p_kad::Record {
            type Error = ::macro_exports::__netabase_anyhow::Error;
            fn try_from(value: #ident) -> Result<Self, ::macro_exports::__netabase_anyhow::Error> {
                let key = ::macro_exports::__netabase_libp2p_kad::RecordKey::try_from(value.key())?;
                let bytes = ::macro_exports::__netabase_bincode::encode_to_vec(value, ::macro_exports::__netabase_bincode_config::standard())?;
                Ok(::macro_exports::__netabase_libp2p_kad::Record {
                    key,
                    value: bytes,
                    publisher: None,
                    expires: None
                })
            }
        }

        impl TryFrom<&#ident> for ::macro_exports::__netabase_libp2p_kad::Record {
            type Error = ::macro_exports::__netabase_anyhow::Error;
            fn try_from(value: &#ident) -> Result<Self, ::macro_exports::__netabase_anyhow::Error> {
                let key = ::macro_exports::__netabase_libp2p_kad::RecordKey::try_from(value.key())?;
                let bytes = ::macro_exports::__netabase_bincode::encode_to_vec(value, ::macro_exports::__netabase_bincode_config::standard())?;
                Ok(::macro_exports::__netabase_libp2p_kad::Record {
                    key,
                    value: bytes,
                    publisher: None,
                    expires: None
                })
            }
        }
    })
}

pub mod netabase_schema_key {

    use proc_macro2::Span;

    use quote::ToTokens;
    use quote::quote;

    use syn::Meta;
    use syn::{
        Arm, Fields, Ident, ImplItemFn, ItemEnum, ItemImpl, ItemStruct, ReturnType, Variant,
        parse_quote,
    };

    use crate::generators::{GenerationError, schema_enum_generator::SchemaEnumGenerator};
    use crate::visitors::Key;

    #[derive(Debug)]
    pub enum KeyItemType {
        StructKey(ItemStruct),
        EnumKey(ItemEnum),
    }

    impl ToTokens for KeyItemType {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                KeyItemType::StructKey(item_struct) => {
                    item_struct.to_tokens(tokens);
                }
                KeyItemType::EnumKey(item_enum) => {
                    item_enum.to_tokens(tokens);
                }
            }
        }
    }

    pub fn generate_netabase_key(
        key: &Key,
        schema_name: &Ident,
    ) -> Result<KeyItemType, GenerationError> {
        let name = Key::ident(schema_name);
        match key {
            Key::Outer { sig } => {
                if let ReturnType::Type(_, boxed_type) = &sig.output {
                    let type_name = *boxed_type.clone();
                    Ok(KeyItemType::StructKey(parse_quote!(
                        #[derive(NetabaseSchemaKey, Debug, ::macro_exports::__netabase_bincode::Encode, ::macro_exports::__netabase_bincode::Decode, Clone)]
                        #[schema_name(#schema_name)]
                        pub struct #name(#type_name);
                    )))
                } else {
                    Err(GenerationError::KeyGeneration {
                        key_type: format!("Outer key for '{}' missing return type", schema_name),
                    })
                }
            }
            Key::StructInner { field, .. } => {
                let type_name = &field.ty;
                Ok(KeyItemType::StructKey(parse_quote!(
                    #[derive(NetabaseSchemaKey, Debug, ::macro_exports::__netabase_bincode::Encode, ::macro_exports::__netabase_bincode::Decode, Clone)]
                    #[schema_name(#schema_name)]
                    pub struct #name(#type_name);
                )))
            }
            Key::EnumInner { variant_fields } => {
                let variant_iter = variant_fields.iter().map(|(v, f)| -> Variant {
                    let variant_ident = &v.ident;
                    let type_name = &f.ty;
                    parse_quote!(
                        #variant_ident(#type_name)
                    )
                });

                Ok(KeyItemType::EnumKey(parse_quote!(
                    #[derive(NetabaseSchemaKey, Debug, ::macro_exports::__netabase_bincode::Encode, ::macro_exports::__netabase_bincode::Decode, Clone)]
                    #[schema_name(#schema_name)]
                    pub enum #name {
                        #(#variant_iter),*
                    }
                )))
            }
            Key::Registry(boxed_enum) => {
                let item =
                    SchemaEnumGenerator::generate_schema_keys_enum_from_attr(boxed_enum, &name)
                        .map_err(|e| GenerationError::KeyGeneration {
                            key_type: format!("Failed to generate registry key enum: {}", e),
                        })?;
                Ok(KeyItemType::EnumKey(parse_quote!(#item)))
            }
        }
    }
    pub fn generate_key_impl(key_ident: &Ident, schema_name: &Ident) -> ItemImpl {
        parse_quote!(
            impl<K: netabase::netabase_trait::NetabaseRegistryKey> netabase::netabase_trait::NetabaseSchemaKey<K> for #key_ident {
            type Schema = #schema_name;
            }
        )
    }

    pub fn generate_from_to_key_record(input: &Ident) -> proc_macro2::TokenStream {
        quote! {
            impl TryFrom<::macro_exports::__netabase_libp2p_kad::RecordKey> for #input {
                type Error = ::macro_exports::__netabase_anyhow::Error;
                fn try_from(value: ::macro_exports::__netabase_libp2p_kad::RecordKey) -> Result<Self, ::macro_exports::__netabase_anyhow::Error> {
                    let bytes = value.to_vec();
                    Ok(::macro_exports::__netabase_bincode::decode_from_slice(&bytes, ::macro_exports::__netabase_bincode_config::standard())?.0)
                }
            }
            impl TryFrom<#input> for ::macro_exports::__netabase_libp2p_kad::RecordKey {
                type Error = ::macro_exports::__netabase_anyhow::Error;
                fn try_from(value: #input) -> Result<Self, ::macro_exports::__netabase_anyhow::Error> {
                    let bytes = ::macro_exports::__netabase_bincode::encode_to_vec(value, ::macro_exports::__netabase_bincode_config::standard())?;
                    Ok(::macro_exports::__netabase_libp2p_kad::RecordKey::new(&bytes))
                }
            }
            impl TryFrom<&#input> for ::macro_exports::__netabase_libp2p_kad::RecordKey {
                type Error = ::macro_exports::__netabase_anyhow::Error;
                fn try_from(value: &#input) -> Result<Self, ::macro_exports::__netabase_anyhow::Error> {
                    let bytes = ::macro_exports::__netabase_bincode::encode_to_vec(value, ::macro_exports::__netabase_bincode_config::standard())?;
                    Ok(::macro_exports::__netabase_libp2p_kad::RecordKey::new(&bytes))
                }
            }
        }
    }
    pub fn generate_key_getter(
        key_item: &KeyItemType,
        key: &Key,
    ) -> Result<ImplItemFn, GenerationError> {
        match (key_item, key) {
            (KeyItemType::StructKey(item_struct), Key::Outer { sig }) => {
                let name = &item_struct.ident;
                let fn_call = &sig.ident;
                Ok(parse_quote!(
                    fn key(&self) -> Self::Key {
                        #name(self.#fn_call())
                    }
                ))
            }
            (KeyItemType::StructKey(item_struct), Key::StructInner { field }) => {
                let name = &item_struct.ident;
                let field_name = {
                    match &field.ident {
                        Some(ident) => ident,
                        None => &Ident::new("0", Span::call_site()),
                    }
                };
                Ok(parse_quote! {
                    fn key(&self) -> Self::Key {
                        #name(self.#field_name.clone())
                    }
                })
            }
            (KeyItemType::EnumKey(item_enum), Key::EnumInner { variant_fields }) => {
                //TODO: Match against itself
                let name = &item_enum.ident;
                let enumkey_variantfields: Result<Vec<Arm>, GenerationError> =
                            item_enum.variants.iter().zip(variant_fields.iter()).map(
                                |(key_variants, (v, f))| {
                                    let variant_name = &v.ident;
                                    let res_variant_name = &key_variants.ident;
                                    match (&v.fields, &f.ident) {
                                        (Fields::Named(_fields_named), Some(field_name)) => {
                                            Ok(parse_quote! {
                                                Self::#variant_name { #field_name, .. } => #name::#res_variant_name(#field_name.clone())
                                            })
                                        },
                                        (Fields::Unnamed(_fields_unnamed), None) => {
                                            Ok(parse_quote! {
                                                Self::#variant_name(first, .. ) => #name::#res_variant_name(first.clone())
                                            })
                                        },
                                        _ => {
                                            Err(GenerationError::KeyGeneration {
                                                key_type: "Invalid enum variant field combination for key generation".to_string(),
                                            })
                                        }
                                    }
                                },
                            ).collect();
                let enumkey_variantfields = enumkey_variantfields?;
                Ok(parse_quote!(
                    fn key(&self) -> Self::Key {
                        match self {
                            #(#enumkey_variantfields),*
                        }
                    }
                ))
            }
            (KeyItemType::EnumKey(_reg_item_enum), Key::Registry(item_enum)) => {
                let arms: Vec<Arm> = {
                    if let Meta::List(list_of_var) = &item_enum.meta {
                        let list = list_of_var.tokens.to_string();
                        let split = list.split(",");
                        split
                            .map(|v| {
                                let v_trimmed = v.trim();
                                // Extract the key variant name (e.g., "MeKey" from "MeKey(schemas :: Me)")
                                let key_name = if let Some(paren_pos) = v_trimmed.find('(') {
                                    v_trimmed[..paren_pos].trim()
                                } else {
                                    v_trimmed
                                };
                                // Extract the schema name (e.g., "Me" from "MeKey")
                                let self_name = if let Some(key_pos) = key_name.find("Key") {
                                    &key_name[..key_pos]
                                } else {
                                    key_name
                                };
                                let self_ident =
                                    syn::Ident::new(self_name, proc_macro2::Span::call_site());
                                let key_ident =
                                    syn::Ident::new(key_name, proc_macro2::Span::call_site());
                                parse_quote!(
                                    Self::#self_ident(__v) => Self::Key::#key_ident(__v.key())
                                )
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                };
                Ok(parse_quote!(
                    fn key(&self) -> Self::Key {
                        match self {
                            #(#arms),*
                        }
                    }
                ))
            }
            (KeyItemType::StructKey(_item_struct), Key::EnumInner { variant_fields: _ }) => {
                Err(GenerationError::KeyGeneration {
                    key_type: "Cannot generate struct key for enum inner key".to_string(),
                })
            }
            (KeyItemType::EnumKey(_item_enum), Key::Outer { sig: _ }) => {
                Err(GenerationError::KeyGeneration {
                    key_type: "Cannot generate enum key for outer key".to_string(),
                })
            }
            (KeyItemType::EnumKey(_item_enum), Key::StructInner { field: _ }) => {
                Err(GenerationError::KeyGeneration {
                    key_type: "Cannot generate enum key for struct inner key".to_string(),
                })
            }
            (KeyItemType::StructKey(_item_struct), Key::Registry(_item_enum)) => {
                Err(GenerationError::KeyGeneration {
                    key_type: format!("Unhandled key generation case: struct key with registry"),
                })
            }
        }
    }
}
