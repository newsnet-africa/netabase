use syn::{
    Field, Fields, FieldsUnnamed, GenericParam, Ident, ItemEnum, Lifetime, LifetimeParam, Token,
    Type, TypePath, Variant, parse_quote, punctuated::Punctuated,
};

use crate::SchemaVisitor;

pub fn generate_variants<'ast>(
    schema: &SchemaVisitor<'ast>,
) -> ((Vec<syn::Variant>, Vec<syn::Variant>), Vec<syn::Variant>) {
    let ((mut model_variants, mut reference_model_variants), mut key_variants): (
        (Vec<Variant>, Vec<Variant>),
        Vec<Variant>,
    ) = ((vec![], vec![]), vec![]);
    schema.items.iter().for_each(|nm| {
        let variant_name = &nm.model.ident;
        let variant_type = &nm.model_path();
        let key_name = &nm.key_name();
        let key_type = &nm.key_path();

        // Build model variant directly
        let model_path = syn::Path {
            leading_colon: None,
            segments: variant_type.clone(),
        };
        let model_type = Type::Path(TypePath {
            qself: None,
            path: model_path,
        });
        let ref_model_type: Type = parse_quote!(
            &'a #model_type
        );
        let model_variant = Variant {
            attrs: vec![],
            ident: variant_name.clone(),
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: syn::token::Paren::default(),
                unnamed: {
                    let mut punctuated = syn::punctuated::Punctuated::new();
                    punctuated.push(Field {
                        attrs: vec![],
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: model_type,
                    });
                    punctuated
                },
            }),
            discriminant: None,
        };
        let ref_model_variant = Variant {
            attrs: vec![],
            ident: variant_name.clone(),
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: syn::token::Paren::default(),
                unnamed: {
                    let mut punctuated = syn::punctuated::Punctuated::new();
                    punctuated.push(Field {
                        attrs: vec![],
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: ref_model_type,
                    });
                    punctuated
                },
            }),
            discriminant: None,
        };

        // Build key variant directly
        let key_path = syn::Path {
            leading_colon: None,
            segments: key_type.clone(),
        };
        let key_type_path = Type::Path(TypePath {
            qself: None,
            path: key_path,
        });
        let key_variant = Variant {
            attrs: vec![],
            ident: key_name.clone(),
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: syn::token::Paren::default(),
                unnamed: {
                    let mut punctuated = syn::punctuated::Punctuated::new();
                    punctuated.push(Field {
                        attrs: vec![],
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: key_type_path,
                    });
                    punctuated
                },
            }),
            discriminant: None,
        };

        model_variants.push(model_variant);
        reference_model_variants.push(ref_model_variant);
        key_variants.push(key_variant);
    });

    ((model_variants, reference_model_variants), key_variants)
}

pub fn generate_enums<'ast>(
    db_name: &Ident,
    model_variants: Vec<Variant>,
    ref_model_variants: Vec<Variant>,
    key_variants: Vec<Variant>,
) -> ((syn::ItemEnum, syn::ItemEnum), syn::ItemEnum) {
    let keys_name = {
        let mut temp_name = db_name.to_string();
        temp_name.push_str("Key");
        Ident::new(&temp_name, proc_macro2::Span::call_site())
    };

    let model_enum = ItemEnum {
        attrs: vec![parse_quote!(#[derive(NetabaseCatalog)])],
        vis: syn::Visibility::Public(syn::token::Pub::default()),
        enum_token: syn::token::Enum::default(),
        ident: db_name.clone(),
        generics: syn::Generics::default(),
        brace_token: syn::token::Brace::default(),
        variants: {
            let mut variants = syn::punctuated::Punctuated::new();
            for variant in model_variants {
                variants.push(variant);
            }
            variants
        },
    };
    let ref_model_enum = ItemEnum {
        attrs: vec![parse_quote!(#[derive(NetabaseCatalog)])],
        vis: syn::Visibility::Public(syn::token::Pub::default()),
        enum_token: syn::token::Enum::default(),
        ident: {
            let mut old_name = db_name.to_string();
            old_name.push_str("Ref");
            Ident::new(&old_name, proc_macro2::Span::call_site())
        },
        generics: syn::Generics {
            lt_token: Some(Token![<](proc_macro2::Span::call_site())),
            params: {
                let param = vec![GenericParam::Lifetime(LifetimeParam::new(
                    syn::Lifetime::new("'a", proc_macro2::Span::call_site()),
                ))];
                Punctuated::<GenericParam, Token![,]>::from_iter(param)
            },
            gt_token: Some(Token![>](proc_macro2::Span::call_site())),
            where_clause: None,
        },
        brace_token: syn::token::Brace::default(),
        variants: {
            let mut variants = syn::punctuated::Punctuated::new();
            for variant in ref_model_variants {
                variants.push(variant);
            }
            variants
        },
    };

    let key_enum = ItemEnum {
        attrs: vec![parse_quote!(#[derive(NetabaseCatalog)])],
        vis: syn::Visibility::Public(syn::token::Pub::default()),
        enum_token: syn::token::Enum::default(),
        ident: keys_name,
        generics: syn::Generics::default(),
        brace_token: syn::token::Brace::default(),
        variants: {
            let mut variants = syn::punctuated::Punctuated::new();
            for variant in key_variants {
                variants.push(variant);
            }
            variants
        },
    };

    ((model_enum, ref_model_enum), key_enum)
}
