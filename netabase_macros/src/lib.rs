use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, visit::Visit, visit_mut::VisitMut};

use crate::{
    generators::key::generate_key,
    visitors::netabase_schema_derive::{DeriveVisitor, NetabaseSchemaVisitor},
};

mod generators;
mod visitors;

#[proc_macro_derive(NetabaseSchema, attributes(key))]
pub fn netabase_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let mut visitor = NetabaseSchemaVisitor::default();
    visitor.visit_derive_input(&derive_input);
    let out = generate_key(visitor);
    quote! {#out}.into()
}

#[proc_macro_attribute]
pub fn key_derive(_derives: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let mut der_visitor = DeriveVisitor::new();
    der_visitor.visit_derive_input_mut(&mut input);
    quote::quote!(#input).into()
}
