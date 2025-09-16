use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemMod, parse_macro_input, visit::Visit};

use crate::{
    generator::{generate_enums, generate_variants},
    visitor::SchemaVisitor,
};

extern crate proc_macro;

mod generator;
mod visitor;

#[proc_macro_attribute]
pub fn netabase_schema(input: TokenStream, item: TokenStream) -> TokenStream {
    let name_ident = parse_macro_input!(input as Ident);
    let items = parse_macro_input!(item as ItemMod);

    eprintln!(
        "netabase_schema: module has inline content = {:?}",
        items.content.is_some()
    );

    let mut vi = SchemaVisitor::default();
    vi.visit_item_mod(&items);

    eprintln!("netabase_schema: found {} native models", vi.items.len());

    let (variants, keys) = generate_variants(&vi);
    let (model_list, key_list) = generate_enums(&name_ident, variants, keys);

    quote! {
        #items
        #model_list
        #key_list
    }
    .into()
}
