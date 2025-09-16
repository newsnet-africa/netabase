use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Field, Ident, ItemEnum, ItemStruct, PathSegment, Token, Variant, parse_quote,
    punctuated::Punctuated,
};

use crate::SchemaVisitor;

pub fn generate_variants<'ast>(
    schema: &SchemaVisitor<'ast>,
) -> (Vec<syn::Variant>, Vec<syn::Variant>) {
    let (mut model_variants, mut key_variants): (Vec<Variant>, Vec<Variant>) = (vec![], vec![]);
    schema.items.iter().map(|nm| {
        let variant_name = &nm.model.ident;
        let variant_type = &nm.model_path();
        let key_name = &nm.key.ident;
        let key_type = &nm.key_path();
        model_variants.push(parse_quote! {
            #variant_name(#variant_type),
        });
        key_variants.push(parse_quote! {
            #key_name(#key_type),
        });
    });

    (model_variants, key_variants)
}

pub fn generate_enums<'ast>(
    db_name: &Ident,
    model_variants: Vec<Variant>,
    key_variants: Vec<Variant>,
) -> (syn::ItemEnum, syn::ItemEnum) {
    let keys_name = {
        let mut temp_name = db_name.to_string();
        temp_name.push_str("Key");
        Ident::new(&temp_name, proc_macro2::Span::call_site())
    };
    let model_enum: ItemEnum = parse_quote! {
        pub enum #db_name {
            #(#model_variants),*
        }
    };
    let key_enum: ItemEnum = parse_quote! {

        pub enum #keys_name {
            #(#key_variants),*
        }
    };

    (model_enum, key_enum)
}
