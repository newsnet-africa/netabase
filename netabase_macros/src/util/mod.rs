use std::{collections::HashMap, str::FromStr};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Ident, Item, ItemStruct, MetaList, Type, parse_quote};

use crate::SchemaModuleVisitor;

pub trait NetabaseItemStruct<'ast> {
    fn is_schema(&'ast self) -> Option<Ident>;
    fn get_schema_name(&'ast self) -> Option<Ident>;
    fn is_key(&'ast self) -> Option<&'ast MetaList>;
}

impl<'ast> NetabaseItemStruct<'ast> for ItemStruct {
    fn is_schema(&'ast self) -> Option<syn::Ident> {
        if self.attrs.iter().any(|att| {
            if let syn::Meta::List(metalist) = &att.meta
                && att.path().is_ident("derive")
                && metalist.tokens.to_string().contains("NetabaseSchema")
            {
                true
            } else {
                false
            }
        }) {
            self.attrs.iter().find_map(|att| {
                if att.path().is_ident("key_name") {
                    let tok = &att.meta.require_list().ok()?.tokens;
                    Some(syn::parse_quote! {
                        #tok
                    })
                } else {
                    // syn::Error::new(proc_macro2::Span::call_site(), "Struct is not Key")
                    None
                }
            })
        } else {
            None
        }
    }

    fn is_key(&'ast self) -> Option<&'ast MetaList> {
        self.attrs.iter().find_map(|att| {
            if att.path().is_ident("key_schema") {
                att.meta.require_list().ok()
            } else {
                // syn::Error::new(proc_macro2::Span::call_site(), "Struct is not Key")
                None
            }
        })
    }

    fn get_schema_name(&'ast self) -> Option<Ident> {
        self.is_key().map(|list| {
            let tok = &list.tokens;
            parse_quote!(#tok)
        })
    }
}

pub trait NetabaseItem<'ast> {
    fn get_struct(&'ast self) -> Option<&'ast ItemStruct>;
}

impl<'ast> NetabaseItem<'ast> for Item {
    fn get_struct(&'ast self) -> Option<&'ast syn::ItemStruct> {
        if let syn::Item::Struct(item_struct) = &self {
            Some(item_struct)
        } else {
            None
        }
    }
}

impl SchemaModuleVisitor {
    pub fn format_paths(&self) -> Vec<((TokenStream, TokenStream), (TokenStream, TokenStream))> {
        self.k_v_pairs
            .iter()
            .map(|(k, v)| {
                eprintln!(
                    "Formatting Paths: {:?}, {:?}",
                    k.to_token_stream().to_string(),
                    v.to_token_stream().to_string()
                );
                (
                    (
                        TokenStream::from_str(
                            &k.iter()
                                .map(|p| p.to_token_stream().to_string())
                                .collect::<Vec<String>>()
                                .join("::"),
                        )
                        .unwrap(),
                        k.last().to_token_stream(),
                    ),
                    (
                        TokenStream::from_str(
                            &v.iter()
                                .map(|p| p.to_token_stream().to_string())
                                .collect::<Vec<String>>()
                                .join("::"),
                        )
                        .unwrap(),
                        v.last().to_token_stream(),
                    ),
                )
            })
            .collect()
    }
}
