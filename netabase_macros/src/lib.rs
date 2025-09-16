use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DataEnum, DeriveInput, Ident, ItemMod, parse_macro_input, visit::Visit, visit_mut::VisitMut,
};

use crate::{
    generator::{generate_enums, generate_variants},
    visitor::{CatalogVisitor, SchemaVisitor},
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

    let ((variants, ref_variants), keys) = generate_variants(&vi);
    let ((model_list, ref_model_list), key_list) =
        generate_enums(&name_ident, variants, ref_variants, keys);

    quote! {
        #items
        #model_list
        #ref_model_list
        #key_list
    }
    .into()
}

#[proc_macro_derive(NetabaseCatalog, attributes(netabase_ref_catalog))]
pub fn derive_catalog(input: TokenStream) -> TokenStream {
    let mut enum_cat = parse_macro_input!(input as DeriveInput);
    let mut mutable_visitor = CatalogVisitor;
    mutable_visitor.visit_derive_input_mut(&mut enum_cat);
    let name = &enum_cat.ident;
    let ref_name = {
        let old = &mut enum_cat.ident.to_string();
        old.push_str("Ref");
        Ident::new(old, proc_macro2::Span::call_site())
    };

    let generics = &enum_cat.generics;

    quote! {
        impl #generics NetabaseCatalog for #name #generics {
            type RefCatalog = #ref_name;
        }
    }
    .into()
}
