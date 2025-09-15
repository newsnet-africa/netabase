use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, ItemMod, parse_macro_input, visit::Visit};

use crate::{
    generator::{build_items_from_generated, generate_record_entity},
    visitor::SchemaVisitor,
};

extern crate proc_macro;

mod generator;
mod visitor;

#[proc_macro_attribute]
pub fn netabase_schema(input: TokenStream, item: TokenStream) -> TokenStream {
    let name_ident = parse_macro_input!(input as Ident);
    let items = parse_macro_input!(item as ItemMod);
    let mut vi = SchemaVisitor::default();
    vi.visit_item_mod(&items);
    let (variants, keys) = generate_record_entity(&name_ident, vi.items);
    let (new_types, main_enum, keys_enum) = build_items_from_generated(&name_ident, variants, keys);
    quote! {
        #items
        #(#new_types)*
        #main_enum
        #keys_enum
    }
    .into()
}
