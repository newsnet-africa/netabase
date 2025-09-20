use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemMod, parse_macro_input, visit::Visit};

use crate::{
    generator::{
        generate_conversions, generate_database_key_variants, generate_enums,
        generate_model_specific_key_enums, generate_ref_iter, generate_variants,
    },
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

    let ((variants, ref_variants), _old_keys) = generate_variants(&vi);
    let ((model_list, ref_model_list), _old_key_list) =
        generate_enums(&name_ident, variants, ref_variants, vec![]);

    // Generate model-specific key enums
    let model_key_enums = generate_model_specific_key_enums(&vi);
    let (iter_struct, iter_impl) = generate_ref_iter(&model_list);

    // Generate new database key enum with model-specific variants
    let db_key_variants = generate_database_key_variants(&vi);
    let keys_name = {
        let mut temp_name = name_ident.to_string();
        temp_name.push_str("Key");
        syn::Ident::new(&temp_name, proc_macro2::Span::call_site())
    };

    let key_list = syn::ItemEnum {
        attrs: vec![syn::parse_quote!(#[derive(Debug, Clone)])],
        vis: syn::Visibility::Public(syn::token::Pub::default()),
        enum_token: syn::token::Enum::default(),
        ident: keys_name,
        generics: syn::Generics::default(),
        brace_token: syn::token::Brace::default(),
        variants: {
            let mut variants = syn::punctuated::Punctuated::new();
            for variant in db_key_variants {
                variants.push(variant);
            }
            variants
        },
    };

    let conversions = generate_conversions(&name_ident, &vi);

    quote! {
        #items
        #model_list
        #ref_model_list
        #iter_struct
        #iter_impl
        #(#model_key_enums)*
        #key_list
        #conversions
    }
    .into()
}
