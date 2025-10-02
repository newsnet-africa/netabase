use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, ItemMod, parse_macro_input, visit::Visit, visit_mut::VisitMut};

use crate::{
    generators::{
        append_ident, models::impls::key_impl::generate_netabase_model_key_trait,
        schema::model_enum::generate_module_schema,
    },
    visitors::{
        netabase_schema_derive::{DeriveVisitor, NetabaseModelVisitor},
        schema_module::SchemaModuleVisitor,
    },
};

mod generators;
mod util;
mod visitors;

#[proc_macro_derive(NetabaseModel, attributes(key, secondary_key, key_name))]
pub fn netabase_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    // Analyze the input without transformation
    let mut visitor = NetabaseModelVisitor::default();
    visitor.visit_derive_input(&derive_input);

    // Check if we found the key field
    if visitor.key_field.is_none() {
        return quote! {
            compile_error!("NetabaseModel requires a field marked with #[key]");
        }
        .into();
    }

    // Generate each component with error handling
    let (main_key_enum, secondary_keys_enum, primary_key_struct, relations_enum) =
        visitor.generate_key();

    let netabase_impl = visitor.generate_netabase_model_trait();

    let main_key_impl = visitor.generate_main_key_impl();

    let primary_key_impl = visitor.generate_primary_key_impl();

    let primary_key_from_impl = visitor.generate_primary_key_from_impl();

    let secondary_keys_impl = visitor.generate_secondary_keys_impl();

    let secondary_keys_try_from_ivec_impl = visitor.generate_secondary_keys_try_from_ivec_impl();

    let secondary_keys_try_into_ivec_impl = visitor.generate_secondary_keys_try_into_ivec_impl();

    let secondary_keys_fn = visitor.generate_secondary_keys_fn();

    let relations_impl = visitor.generate_relations_impl();

    let relations_try_from_ivec_impl = visitor.generate_relations_try_from_ivec_impl();

    let relations_try_into_ivec_impl = visitor.generate_relations_try_into_ivec_impl();

    let relations_fn = visitor.generate_relations_fn();

    let secondary_keys_placeholder_impls = visitor.generate_secondary_keys_placeholder_impls();
    let relations_placeholder_impls = visitor.generate_relations_placeholder_impls();

    let type_alias = visitor.generate_type_alias();

    let final_tokens = quote! {
        #primary_key_struct
        #primary_key_impl
        #primary_key_from_impl
        #secondary_keys_enum
        #secondary_keys_impl
        #secondary_keys_try_from_ivec_impl
        #secondary_keys_try_into_ivec_impl
        #secondary_keys_placeholder_impls
        #relations_enum
        #relations_impl
        #relations_try_from_ivec_impl
        #relations_try_into_ivec_impl
        #relations_placeholder_impls
        #main_key_enum
        #main_key_impl
        #netabase_impl
        #secondary_keys_fn
        #relations_fn
        #type_alias
    };

    final_tokens.into()
}

#[proc_macro_derive(NetabaseModelKey)]
pub fn netabase_key_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let netabase_impl = generate_netabase_model_key_trait(&derive_input);
    quote! {
        #netabase_impl
    }
    .into()
}

#[proc_macro_attribute]
pub fn key_derive(_derives: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let mut der_visitor = DeriveVisitor::new();
    der_visitor.visit_derive_input_mut(&mut input);
    quote::quote!(#input).into()
}

#[proc_macro_attribute]
pub fn netabase_schema_module(name: TokenStream, input: TokenStream) -> TokenStream {
    // let name = parse_macro_input!(name as Ident);
    let binding = name.to_string();
    let mut split = binding.split(",");
    let schema_ident = match split.next() {
        Some(sp) => Ident::new(sp.trim(), proc_macro2::Span::call_site()),
        None => panic!("Schema needs a name"),
    };
    let key_ident = match split.next() {
        Some(sp) => Ident::new(sp.trim(), proc_macro2::Span::call_site()),
        None => append_ident(&schema_ident, "Key"),
    };
    let mut input = parse_macro_input!(input as ItemMod);
    let mut visitor = SchemaModuleVisitor::new(schema_ident, key_ident);
    visitor.visit_item_mod(&input);

    let (schema, key, impls) = generate_module_schema(visitor);
    let temp_cont = input.content.unwrap();
    let mut new_vec = temp_cont.1;
    new_vec.push(syn::Item::Enum(schema));
    new_vec.push(syn::Item::Enum(key));

    // Add all the generated implementations
    for impl_item in impls {
        new_vec.push(syn::Item::Impl(impl_item));
    }

    input.content = Some((temp_cont.0, new_vec));
    quote! {
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn key_schema(_item: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn key_name(_item: TokenStream, input: TokenStream) -> TokenStream {
    input
}
