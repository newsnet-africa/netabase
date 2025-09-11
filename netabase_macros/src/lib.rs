#![feature(proc_macro_diagnostic)]
mod generators;
mod visitors;

extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, Ident, ItemMod, parse_macro_input, visit::Visit};

use crate::{
    generators::{
        IntoCompileError, SchemaEnumGenerator,
        generate_netabase_impl::{
            generate_netabase_macro,
            netabase_schema_key::{generate_from_to_key_record, generate_key_impl},
        },
    },
    visitors::{RegistryVisitor, SchemaCounterVisitor, SchemaValidator},
};

#[proc_macro_attribute]
pub fn schema_module(name: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemMod);
    let mut visit = SchemaCounterVisitor::default();
    visit.visit_item_mod(&input);
    let generator = SchemaEnumGenerator::new(visit.get_schemas());

    let ident = parse_macro_input!(name as Ident);

    match generator.generate_both_enums(&ident) {
        Ok(tok) => quote! {
            #input
            #tok
        }
        .into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(NetabaseSchema, attributes(key, __netabase_registery))]
pub fn netabase_schema_derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let mut vi = SchemaValidator::default();
    vi.visit_derive_input(&inp);

    match generate_netabase_macro(vi) {
        Ok(net_impl) => quote! {
            #net_impl
        }
        .into(),
        Err(err) => err.into_compile_error().into(),
    }
}
#[proc_macro_derive(NetabaseRegistry)]
pub fn netabase_registry_derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let mut vi = RegistryVisitor::default();
    vi.visit_derive_input(&inp);

    match vi.generate_keys_registry() {
        Ok((net_impl, impl_item)) => quote! {
            #net_impl
            #impl_item
        }
        .into(),
        Err(err) => err.into_compile_error().into(),
    }
}
#[proc_macro_derive(NetabaseSchemaKey)]
pub fn netabase_schema_key_derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let name = inp.ident;

    match (|| -> Result<_, crate::generators::GenerationError> {
        let net_impl = generate_key_impl(&name);
        let conversions = generate_from_to_key_record(&name);
        Ok(quote! {
            #net_impl
            #conversions
        })
    })() {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
