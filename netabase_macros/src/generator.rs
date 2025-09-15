use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Field, Ident, ItemEnum, ItemStruct, PathSegment, Token, Variant, parse_quote,
    punctuated::Punctuated,
};

pub fn generate_record_entity<'ast>(
    _name: &'ast Ident,
    items: Vec<(
        &'ast ItemStruct,
        Punctuated<PathSegment, Token![::]>,
        (&'ast Field, Option<&'ast Field>),
    )>,
) -> (Vec<Variant>, Vec<(ItemStruct, Option<ItemStruct>)>) {
    let mut variants: Vec<Variant> = Vec::with_capacity(items.len());
    let mut keys: Vec<(ItemStruct, Option<ItemStruct>)> = Vec::with_capacity(items.len());

    for (i_struct, path, (primary_field, secondary_field)) in items.into_iter() {
        let var_name = &i_struct.ident;

        let mut primary_key_name = var_name.to_string();
        primary_key_name.push_str("PrimaryKey");
        let primary_key_ident = Ident::new(&primary_key_name, Span::call_site());

        let mut secondary_key_name = var_name.to_string();
        secondary_key_name.push_str("SecondaryKey");
        let secondary_key_ident = Ident::new(&secondary_key_name, Span::call_site());

        let primary_key_type = &primary_field.ty;
        let secondary_key_type = secondary_field.map(|f| &f.ty);

        let var_type = path.to_token_stream();

        let variant: Variant = parse_quote! {
            #var_name(#var_type)
        };

        let primary_key_struct: ItemStruct = parse_quote! {
            pub struct #primary_key_ident(pub #primary_key_type);
        };

        let secondary_key_struct: Option<ItemStruct> = secondary_key_type.map(|t| {
            parse_quote! {
                pub struct #secondary_key_ident(pub #t);
            }
        });

        variants.push(variant);
        keys.push((primary_key_struct, secondary_key_struct));
    }

    (variants, keys)
}

pub fn build_items_from_generated(
    name: &Ident,
    variants: Vec<Variant>,
    keys: Vec<(ItemStruct, Option<ItemStruct>)>,
) -> (Vec<ItemStruct>, ItemEnum, ItemEnum) {
    // Collect all newtype ItemStructs into a flat Vec<ItemStruct>
    let mut newtypes: Vec<ItemStruct> = Vec::new();
    for (primary, secondary_opt) in &keys {
        newtypes.push(primary.clone());
        if let Some(sec) = secondary_opt {
            newtypes.push(sec.clone());
        }
    }

    // Build main enum ItemEnum from variants
    let enum_ident = name.clone();
    let main_enum: ItemEnum = parse_quote! {
        pub enum #enum_ident {
            #(#variants),*
        }
    };

    // Build Keys enum (name = <Name>Keys) with one variant per main enum variant
    let keys_enum_ident = Ident::new(&format!("{}Keys", enum_ident), enum_ident.span());

    // Prepare keys enum variants tokens by zipping original variants with keys
    let mut keys_variant_tokens = Vec::with_capacity(variants.len());
    for (variant, (primary_struct, secondary_struct_opt)) in
        variants.into_iter().zip(keys.into_iter())
    {
        let var_ident = variant.ident.clone();

        let primary_ident = primary_struct.ident.clone();
        if let Some(secondary_struct) = secondary_struct_opt {
            let secondary_ident = secondary_struct.ident.clone();

            // variant with both primary and optional secondary
            let tokens = quote::quote! {
                #var_ident {
                    primary: #primary_ident,
                    secondary: Option<#secondary_ident>,
                }
            };
            keys_variant_tokens.push(tokens);
        } else {
            // variant with primary and secondary as Option<()> (no secondary newtype)
            let tokens = quote::quote! {
                #var_ident {
                    primary: #primary_ident,
                    secondary: Option<()>,
                }
            };
            keys_variant_tokens.push(tokens);
        }
    }

    let keys_enum: ItemEnum = parse_quote! {
        pub enum #keys_enum_ident {
            #(#keys_variant_tokens),*
        }
    };

    (newtypes, main_enum, keys_enum)
}
