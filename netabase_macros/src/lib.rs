#![feature(extend_one)]

use proc_macro::{Span, TokenStream};
use proc_macro2::{Group, TokenStream as TokenStream2};
// use proc_macro::quote;
use quote::{ToTokens, quote};
use syn::{
    AttrStyle, GenericParam, Generics, Ident, Item, ItemMod, Meta, MetaList, Path, Token,
    TraitBound, TypeParamBound, WhereClause, WherePredicate,
    punctuated::Punctuated,
    token::{Comma, Gt, Lt},
    visit::Visit,
};

use crate::visitors::ValidSchemaFinder;

mod visitors;

#[proc_macro_attribute]
pub fn schemas(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module_tokes = proc_macro2::TokenStream::from(item.clone());
    let schema_module = syn::parse_macro_input!(item as ItemMod);
    let schemas: Vec<((Ident, Path), Generics)> = {
        let mut visitor = ValidSchemaFinder::default();
        visitor.visit_item_mod(&schema_module);
        visitor
            .valid_schemas
            .iter()
            .filter_map(|(i, p)| match i {
                Item::Enum(item_enum) => {
                    Some((
                        (item_enum.ident.clone(), p.clone()),
                        item_enum.generics.clone(),
                    )) // TODO: Generics
                }
                Item::Struct(item_struct) => {
                    Some((
                        (item_struct.ident.clone(), p.clone()),
                        item_struct.generics.clone(),
                    )) // TODO: Generics
                }
                _ => None,
            })
            .collect()
    };
    let schema_variants = schemas.iter().map(|((i, p), generics)| {
        let full_struct = {
            let names = { generics.type_params().map(|tp| tp.ident.clone()) };
            quote! {#p<#(#names), *>}
        };

        syn::Variant {
            attrs: vec![],
            ident: i.clone(),
            fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
                paren_token: syn::token::Paren {
                    span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                        .delim_span(),
                },
                unnamed: {
                    let mut path_field = Punctuated::new();
                    path_field.push(syn::Field {
                        attrs: vec![],
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: syn::Type::Verbatim(full_struct),
                    });
                    path_field
                },
            }),
            discriminant: None,
        }
    });

    let full_generics = {
        Generics {
            lt_token: Some(Lt {
                spans: [Span::call_site().into()],
            }),
            params: Punctuated::from_iter(schemas.iter().map(|(_, g)| g.params.clone()).flatten()),
            gt_token: Some(Gt {
                spans: [Span::call_site().into()],
            }),
            where_clause: Some(WhereClause {
                where_token: syn::token::Where {
                    span: Span::call_site().into(),
                },
                predicates: Punctuated::from_iter(
                    schemas
                        .iter()
                        .filter_map(|(_, g)| {
                            g.where_clause
                                .as_ref()
                                .map(|where_clause| where_clause.predicates.clone())
                        })
                        .flatten(),
                ),
            }),
        }
    };
    let wh = full_generics.where_clause.clone().unwrap();

    quote! {
        #module_tokes
        pub enum NetabaseSchema #full_generics #wh{
            #(#schema_variants),*
        }

    }
    .into()
}

#[proc_macro_derive(NetabaseSchema, attributes(key))]
pub fn netabase_schema(input: TokenStream) -> TokenStream {
    let schema = syn::parse_macro_input!(input as Item);
    quote! {}.into()
}
