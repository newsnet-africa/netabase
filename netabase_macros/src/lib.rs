use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemMod, parse_macro_input, visit::Visit};

use crate::{
    generator::{generate_conversions, generate_enums, generate_variants},
    visitor::SchemaVisitor,
};

extern crate proc_macro;

mod generator;
mod visitor;

#[proc_macro_derive(NetabaseCatalog)]
pub fn derive_netabase_catalog(input: TokenStream) -> TokenStream {
    // Stub implementation - currently does nothing
    // The actual catalog functionality is handled by the netabase_schema macro
    TokenStream::new()
}

#[proc_macro_derive(NetabaseCatalogRef)]
pub fn derive_netabase_catalog_ref(input: TokenStream) -> TokenStream {
    // Stub implementation - currently does nothing
    // The actual catalog ref functionality is handled by the netabase_schema macro
    TokenStream::new()
}

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

    let ((variants, ref_variants), keys) = generate_variants(&vi);
    let ((model_list, ref_model_list), key_list) =
        generate_enums(&name_ident, variants, ref_variants, keys);

    let conversions = generate_conversions(&name_ident, &vi);

    quote! {
        #items
        #model_list
        #ref_model_list
        #key_list
        #conversions
    }
    .into()
}
