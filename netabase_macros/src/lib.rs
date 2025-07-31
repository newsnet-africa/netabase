#![feature(extend_one)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemMod, ItemStruct, Path, parse_macro_input, visit::Visit};

use crate::visitors::{schema_finder::SchemaFinder, utils::KeyType};

mod generators;
mod visitors;

#[proc_macro_attribute]
pub fn schema(_: TokenStream, module: TokenStream) -> TokenStream {
    let module = parse_macro_input!(module as ItemMod);
    let id = module.ident.clone();
    let mut finder = SchemaFinder::default();
    finder.visit_item_mod(&module);
    let schemas = finder
        .schemas
        .iter()
        .map(|i| i.schema_key.clone().unwrap().generation_type.unwrap())
        .collect::<Vec<KeyType<'_>>>();
    quote! {
        fn foo() {
            stringify!(#(#schemas)*);
        }
    }
    .into()
}
