use quote::ToTokens;
use syn::{Ident, ItemEnum, ItemStruct, parse_quote};

use crate::{
    append_ident,
    util::{
        netabase_error_type_path, netabase_model_key_trait_path,
        netabase_relational_keys_trait_path, netabase_secondary_keys_trait_path,
    },
    visitors::netabase_schema_derive::NetabaseModelVisitor,
};

impl<'ast> NetabaseModelVisitor<'ast> {
    pub fn generate_key(&self) -> (ItemEnum, ItemEnum, ItemStruct, ItemEnum) {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let key_name = match &self.key_name {
            Some(sp) => sp,
            None => &append_ident(name, "Key"),
        };

        let secondary_keys_name = append_ident(name, "SecondaryKeys");
        let primary_key_name = append_ident(name, "PrimaryKey");
        let relations_name = append_ident(name, "Relations");

        let primary_key_type = if let Some(keys) = self.key_field {
            &keys.ty
        } else {
            panic!("Primary key type not found")
        };

        // Generate primary key struct
        let mut primary_key_derive = quote::quote!(
            ::netabase_deps::__private::bincode::Encode,
            ::netabase_deps::__private::bincode::Decode,
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash,
            ::netabase_deps::__private::derive_more::From,
            ::netabase_deps::__private::derive_more::Into,
            ::netabase_deps::__private::serde::Serialize,
            ::netabase_deps::__private::serde::Deserialize
        );
        if let Some(keys) = self.key_derive {
            keys.tokens.clone().to_tokens(&mut primary_key_derive);
        }

        let primary_key_struct: ItemStruct = parse_quote! {
            #[derive(#primary_key_derive)]
            pub struct #primary_key_name(pub #primary_key_type);
        };

        // Generate secondary key variants
        let secondary_key_variants: Vec<syn::Variant> = self
            .secondary_key_fields
            .iter()
            .filter_map(|field| {
                field.ident.as_ref().map(|ident| {
                    let ident_string = ident.to_string();
                    let variant_name = format!(
                        "{}Key",
                        ident_string
                            .chars()
                            .enumerate()
                            .map(|(i, c)| if i == 0 {
                                c.to_uppercase().collect::<String>()
                            } else {
                                c.to_string()
                            })
                            .collect::<String>()
                    );
                    let variant_ident = Ident::new(&variant_name, proc_macro2::Span::call_site());
                    let field_type = &field.ty;

                    parse_quote! {
                        #variant_ident(#field_type)
                    }
                })
            })
            .collect();

        // Generate secondary keys enum
        let secondary_keys_enum: ItemEnum = if secondary_key_variants.is_empty() {
            // Create a placeholder enum if no secondary keys exist
            parse_quote! {
                #[derive(::netabase_deps::__private::bincode::Encode, ::netabase_deps::__private::bincode::Decode, Debug, Clone, Hash, PartialEq, Eq, ::netabase_deps::__private::serde::Serialize, ::netabase_deps::__private::serde::Deserialize)]
                pub enum #secondary_keys_name {
                    #[doc = "Placeholder for models with no secondary keys"]
                    _NoSecondaryKeys,
                }
            }
        } else {
            parse_quote! {
                #[derive(::netabase_deps::__private::bincode::Encode, ::netabase_deps::__private::bincode::Decode, Debug, Clone, Hash, PartialEq, Eq, ::netabase_deps::__private::strum::EnumIter, ::netabase_deps::__private::strum::EnumDiscriminants, ::netabase_deps::__private::serde::Serialize, ::netabase_deps::__private::serde::Deserialize)]
                #[strum_discriminants(derive(::netabase_deps::__private::strum::EnumIter, ::netabase_deps::__private::strum::AsRefStr, Hash))]
                pub enum #secondary_keys_name {
                    #(#secondary_key_variants),*
                }
            }
        };

        // Generate relations enum (always empty placeholder now)
        let relations_enum: ItemEnum = parse_quote! {
            #[derive(::netabase_deps::__private::bincode::Encode, ::netabase_deps::__private::bincode::Decode, Debug, Clone, Hash, PartialEq, Eq, ::netabase_deps::__private::serde::Serialize, ::netabase_deps::__private::serde::Deserialize)]
            pub enum #relations_name {
                #[doc = "Placeholder for models with no relations"]
                _NoRelations,
            }
        };

        // Generate main key enum with Primary and Secondary variants
        let main_key_enum: ItemEnum = parse_quote! {
            #[derive(::netabase_deps::__private::bincode::Encode, ::netabase_deps::__private::bincode::Decode, Debug, Clone, PartialEq, Eq, Hash, ::netabase_deps::__private::derive_more::From, ::netabase_deps::__private::derive_more::TryInto, ::netabase_deps::__private::serde::Serialize, ::netabase_deps::__private::serde::Deserialize)]
            pub enum #key_name {
                Primary(#primary_key_name),
                Secondary(#secondary_keys_name),
            }
        };

        (
            main_key_enum,
            secondary_keys_enum,
            primary_key_struct,
            relations_enum,
        )
    }

    pub fn generate_main_key_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let key_name = match &self.key_name {
            Some(sp) => sp,
            None => &append_ident(name, "Key"),
        };

        let secondary_keys_name = append_ident(name, "SecondaryKeys");

        // Generate discriminant type name for secondary keys
        let secondary_keys_discriminants = if self.secondary_key_fields.is_empty() {
            // For empty secondary keys, use the enum itself as discriminant
            secondary_keys_name.clone()
        } else {
            // For enums with variants, strum generates a Discriminants type
            append_ident(&secondary_keys_name, "Discriminants")
        };

        let primary_key_name = append_ident(name, "PrimaryKey");

        let trait_path = netabase_model_key_trait_path();
        parse_quote! {
            impl #trait_path for #key_name {
                type PrimaryKey = #primary_key_name;
                type SecondaryKeys = #secondary_keys_name;
                type SecondaryKeysDiscriminants = #secondary_keys_discriminants;

                fn secondary_key_discriminants() -> Vec<Self::SecondaryKeysDiscriminants> {
                    <Self::SecondaryKeysDiscriminants as ::netabase_deps::__private::strum::IntoEnumIterator>::iter().collect()
                }

                fn primary_keys(&self) -> Option<&Self::PrimaryKey> {
                    match self {
                        #key_name::Primary(pk) => Some(pk),
                        _ => None,
                    }
                }

                fn secondary_keys(&self) -> Option<&Self::SecondaryKeys> {
                    match self {
                        #key_name::Secondary(sk) => Some(sk),
                        _ => None,
                    }
                }
            }
        }
    }

    pub fn generate_primary_key_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let primary_key_name = append_ident(name, "PrimaryKey");

        parse_quote! {
            impl #primary_key_name {
                // Primary key implementation - no traits needed
            }
        }
    }

    pub fn generate_primary_key_from_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let primary_key_name = append_ident(name, "PrimaryKey");
        let key_name = match &self.key_name {
            Some(sp) => sp,
            None => &append_ident(name, "Key"),
        };

        let primary_key_type = if let Some(keys) = self.key_field {
            &keys.ty
        } else {
            panic!("Primary key type not found")
        };

        parse_quote! {
            impl From<#primary_key_type> for #key_name {
                fn from(value: #primary_key_type) -> Self {
                    Self::Primary(#primary_key_name(value))
                }
            }
        }
    }

    pub fn generate_secondary_keys_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let secondary_keys_name = append_ident(name, "SecondaryKeys");

        let trait_path = netabase_secondary_keys_trait_path();
        parse_quote! {
            impl #trait_path for #secondary_keys_name {}
        }
    }

    pub fn generate_secondary_keys_try_from_ivec_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let secondary_keys_name = append_ident(name, "SecondaryKeys");

        let error_path = netabase_error_type_path();
        parse_quote! {
            impl TryFrom<::netabase_deps::__private::sled::IVec> for #secondary_keys_name {
                type Error = #error_path;
                fn try_from(ivec: ::netabase_deps::__private::sled::IVec) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::bincode::decode_from_slice::<Self, _>(&ivec, ::netabase_deps::__private::bincode::config::standard())?.0)
                }
            }
        }
    }

    pub fn generate_secondary_keys_try_into_ivec_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let secondary_keys_name = append_ident(name, "SecondaryKeys");

        let error_path = netabase_error_type_path();
        parse_quote! {
            impl TryFrom<#secondary_keys_name> for ::netabase_deps::__private::sled::IVec {
                type Error = #error_path;
                fn try_from(value: #secondary_keys_name) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::sled::IVec::from(::netabase_deps::__private::bincode::encode_to_vec(&value, ::netabase_deps::__private::bincode::config::standard())?))
                }
            }
        }
    }

    pub fn generate_relations_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let relations_name = append_ident(name, "Relations");

        let trait_path = netabase_relational_keys_trait_path();
        parse_quote! {
            impl #trait_path for #relations_name {}
        }
    }

    pub fn generate_relations_try_from_ivec_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let relations_name = append_ident(name, "Relations");

        let error_path = netabase_error_type_path();
        parse_quote! {
            impl TryFrom<::netabase_deps::__private::sled::IVec> for #relations_name {
                type Error = #error_path;
                fn try_from(ivec: ::netabase_deps::__private::sled::IVec) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::bincode::decode_from_slice::<Self, _>(&ivec, ::netabase_deps::__private::bincode::config::standard())?.0)
                }
            }
        }
    }

    pub fn generate_relations_try_into_ivec_impl(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let relations_name = append_ident(name, "Relations");

        let error_path = netabase_error_type_path();
        parse_quote! {
            impl TryFrom<#relations_name> for ::netabase_deps::__private::sled::IVec {
                type Error = #error_path;
                fn try_from(value: #relations_name) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::sled::IVec::from(::netabase_deps::__private::bincode::encode_to_vec(&value, ::netabase_deps::__private::bincode::config::standard())?))
                }
            }
        }
    }

    pub fn generate_secondary_keys_fn(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let secondary_key_names: Vec<proc_macro2::TokenStream> = self
            .secondary_key_fields
            .iter()
            .filter_map(|field| {
                field.ident.as_ref().map(|ident| {
                    let ident_str = ident.to_string();
                    quote::quote! { #ident_str }
                })
            })
            .collect();

        parse_quote! {
            impl #name {
                pub fn secondary_keys() -> Vec<&'static str> {
                    vec![#(#secondary_key_names),*]
                }
            }
        }
    }

    pub fn generate_relations_fn(&self) -> syn::ItemImpl {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        parse_quote! {
            impl #name {
                pub fn relations() -> Vec<&'static str> {
                    vec![]
                }
            }
        }
    }

    pub fn generate_secondary_keys_placeholder_impls(&self) -> proc_macro2::TokenStream {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let secondary_keys_name = append_ident(name, "SecondaryKeys");

        if self.secondary_key_fields.is_empty() {
            // Generate trait implementations for placeholder enum
            quote::quote! {
                impl ::netabase_deps::__private::strum::IntoEnumIterator for #secondary_keys_name {
                    type Iterator = ::netabase_deps::__private::std::iter::Once<Self>;
                    fn iter() -> Self::Iterator {
                        ::netabase_deps::__private::std::iter::once(#secondary_keys_name::_NoSecondaryKeys)
                    }
                }

                impl AsRef<str> for #secondary_keys_name {
                    fn as_ref(&self) -> &str {
                        "_no_secondary_keys"
                    }
                }
            }
        } else {
            // No additional implementations needed for enums with variants
            quote::quote! {}
        }
    }

    pub fn generate_relations_placeholder_impls(&self) -> proc_macro2::TokenStream {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let relations_name = append_ident(name, "Relations");

        // Always generate trait implementations for placeholder enum
        quote::quote! {
            impl ::netabase_deps::__private::strum::IntoEnumIterator for #relations_name {
                type Iterator = ::netabase_deps::__private::std::iter::Once<Self>;
                fn iter() -> Self::Iterator {
                    ::netabase_deps::__private::std::iter::once(#relations_name::_NoRelations)
                }
            }

            impl AsRef<str> for #relations_name {
                fn as_ref(&self) -> &str {
                    "_no_relations"
                }
            }
        }
    }

    pub fn generate_main_key_ivec_impls(&self) -> proc_macro2::TokenStream {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let key_name = match &self.key_name {
            Some(sp) => sp,
            None => &append_ident(name, "Key"),
        };

        let error_path = netabase_error_type_path();

        quote::quote! {
            impl TryFrom<::netabase_deps::__private::sled::IVec> for #key_name {
                type Error = #error_path;
                fn try_from(ivec: ::netabase_deps::__private::sled::IVec) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::bincode::decode_from_slice::<Self, _>(&ivec, ::netabase_deps::__private::bincode::config::standard())?.0)
                }
            }

            impl TryFrom<#key_name> for ::netabase_deps::__private::sled::IVec {
                type Error = #error_path;
                fn try_from(value: #key_name) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::sled::IVec::from(::netabase_deps::__private::bincode::encode_to_vec(&value, ::netabase_deps::__private::bincode::config::standard())?))
                }
            }
        }
    }

    pub fn generate_model_ivec_impls(&self) -> proc_macro2::TokenStream {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };

        let error_path = netabase_error_type_path();

        quote::quote! {
            impl TryFrom<::netabase_deps::__private::sled::IVec> for #name {
                type Error = #error_path;
                fn try_from(ivec: ::netabase_deps::__private::sled::IVec) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::bincode::decode_from_slice::<Self, _>(&ivec, ::netabase_deps::__private::bincode::config::standard())?.0)
                }
            }

            impl TryFrom<#name> for ::netabase_deps::__private::sled::IVec {
                type Error = #error_path;
                fn try_from(value: #name) -> Result<Self, Self::Error> {
                    Ok(::netabase_deps::__private::sled::IVec::from(::netabase_deps::__private::bincode::encode_to_vec(&value, ::netabase_deps::__private::bincode::config::standard())?))
                }
            }
        }
    }
}
