use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Ident, ItemMod, Token,
    parse::Parser,
    parse_macro_input,
    punctuated::Punctuated,
    visit::Visit,
};

use crate::visitors::{definitions_visitor::DefinitionsVisitor, model_visitor::ModelVisitor};

mod errors;
mod generators;
mod item_info;
mod util;
mod visitors;

#[proc_macro_derive(NetabaseModel, attributes(primary_key, secondary_key, link))]
pub fn netabase_model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut visitor = ModelVisitor::default();
    visitor.visit_derive_input(&input);
    let (p, sl, s, k) = visitor.generate_keys();
    let trait_impl = visitor.generate_model_trait_impl();

    quote! {
        #p
        #(#sl)*
        #s
        #k
        #trait_impl
    }
    .into()
}

#[proc_macro_derive(NetabaseModelKey)]
pub fn netabase_model_key_derive(_input: TokenStream) -> TokenStream {
    quote! {}.into()
}

#[proc_macro_attribute]
pub fn netabase_definition_module(name: TokenStream, input: TokenStream) -> TokenStream {
    let mut def_module = parse_macro_input!(input as ItemMod);
    let mut visitor = DefinitionsVisitor::default();
    visitor.visit_item_mod(&def_module);
    let list = match Punctuated::<Ident, Token![,]>::parse_terminated.parse(name) {
        Ok(l) => l,
        Err(e) => panic!("Error parsing Definitions module: {e}"),
    };
    let definition = list.first().unwrap();
    let definition_key = list.last().unwrap();
    let (defin, def_key) = visitor.generate_definitions(definition, definition_key);
    let trait_impls = visitor.generate_definition_trait_impls(definition, definition_key);

    if let Some((_, c)) = &mut def_module.content {
        c.push(syn::Item::Enum(defin));
        c.push(syn::Item::Enum(def_key));
    };

    quote! {
        #def_module
        #trait_impls
    }.into()
}
