#![feature(extend_one)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemMod, ItemStruct, Path, PathSegment, parse_macro_input, visit::Visit};

use crate::visitors::{schema_finder::SchemaFinder, schema_validator::SchemaValidator};

mod generators;
mod visitors;

#[proc_macro_attribute]
pub fn schema(_: TokenStream, module1: TokenStream) -> TokenStream {
    let module1 = parse_macro_input!(module1 as ItemMod);
    let id = module1.ident.clone();
    let mut finder = SchemaFinder::default();
    finder.visit_item_mod(&module1);
    let schemas = finder
        .schemas
        .iter()
        .map(|sc| sc.path.last().unwrap().clone())
        .collect::<Vec<PathSegment>>();
    quote! {
        fn list(){
            #(#schemas);*;
        }
    }
    .into()
}
