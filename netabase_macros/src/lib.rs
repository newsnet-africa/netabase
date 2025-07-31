#![feature(extend_one)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemMod, ItemStruct, Path, PathSegment, parse_macro_input, visit::Visit};

<<<<<<< HEAD
use crate::visitors::{schema_finder::SchemaFinder, schema_validator::SchemaValidator};
=======
use crate::visitors::{schema_finder::SchemaFinder, utils::KeyType};
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724

mod generators;
mod visitors;

#[proc_macro_attribute]
<<<<<<< HEAD
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
=======
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
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
        }
    }
    .into()
}
