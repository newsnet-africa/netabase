use quote::quote;
use std::collections::HashMap;

use quote::ToTokens;
use syn::{ExprClosure, Field, Fields, PathSegment, Token, Variant, punctuated::Punctuated};

use crate::visitors::schema_finder::SchemaType;

#[derive(Clone, Default)]
pub(crate) struct SchemaInfo<'ast> {
    pub schema_type: Option<SchemaType<'ast>>,
    pub path: Punctuated<PathSegment, Token![::]>,
    pub schema_key: Option<KeyInfo<'ast>>,
}

#[derive(Clone)]
pub(crate) enum KeyType<'schema> {
    FieldKeys(HashMap<Option<&'schema Variant>, &'schema Fields>),
    SchemaKey(&'schema ExprClosure),
}

impl ToTokens for KeyType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            KeyType::FieldKeys(hash_map) => {
                let stream = hash_map.iter().filter_map(|(o, f)| {
                    o.as_ref().map(|op| {
                        quote! {
                            #op: #f,
                        }
                    })
                });
                tokens.extend_one(quote! {
                    #(#stream)*;
                });
            }
            KeyType::SchemaKey(expr_closure) => expr_closure.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct KeyInfo<'schema> {
    pub generation_type: Option<KeyType<'schema>>,
}
