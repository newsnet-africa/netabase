use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Field, Fields, FieldsUnnamed, GenericParam, Ident, ItemEnum, ItemImpl, ItemStruct,
    LifetimeParam, Token, Type, TypePath, Variant, parse_quote, punctuated::Punctuated,
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

        // For keys, we'll use a variant that stores the actual primary key type
        let key_variant_name = {
            let mut name = variant_name.to_string();
            name.push_str("Key");
            Ident::new(&name, variant_name.span())
        };

        // Use the actual primary key type if available, fallback to String
        let key_type = nm
            .primary_key_type
            .clone()
            .unwrap_or_else(|| parse_quote!(String));

        let key_variant = Variant {
            attrs: vec![],
            ident: key_variant_name,
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
                        ty: key_type,
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
        attrs: vec![parse_quote!(#[derive(Debug, Clone, bincode::Encode, bincode::Decode)])],
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
        attrs: vec![parse_quote!(#[derive(Debug, Clone, Copy)])],
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
        attrs: vec![parse_quote!(#[derive(Debug, Clone)])],
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

pub fn generate_ref_iter(ref_enum: &ItemEnum) -> (ItemStruct, ItemImpl) {
    let mut name = ref_enum.ident.to_string();
    name.push_str("DBIter");
    let name = Ident::new(&name, proc_macro2::Span::call_site());

    // Extract the base enum name (remove "Ref" suffix)
    let base_enum_name = {
        let name_str = ref_enum.ident.to_string();
        if name_str.ends_with("Ref") {
            let base_str = &name_str[..name_str.len() - 3];
            Ident::new(base_str, proc_macro2::Span::call_site())
        } else {
            ref_enum.ident.clone()
        }
    };

    // Extract variant types from the reference enum
    let variant_info: Vec<(Ident, Type)> = ref_enum
        .variants
        .iter()
        .filter_map(|v| {
            if let Fields::Unnamed(fields_unnamed) = &v.fields {
                if let Some(first) = fields_unnamed.unnamed.first() {
                    // Extract the inner type from &'a Type
                    if let Type::Reference(type_ref) = &first.ty {
                        Some((v.ident.clone(), (*type_ref.elem).clone()))
                    } else {
                        Some((v.ident.clone(), first.ty.clone()))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Generate iterator methods that return collected results to avoid lifetime issues
    let iter_methods: Vec<TokenStream> = variant_info
        .iter()
        .map(|(variant_name, ty)| {
            let method_name = syn::Ident::new(
                &format!("scan_{}", variant_name.to_string().to_lowercase()),
                proc_macro2::Span::call_site(),
            );
            quote! {
                pub fn #method_name(&self) -> native_db::db_type::Result<Vec<#ty>> {
                    let r_transaction = self.database.r_transaction()?;
                    let scan = r_transaction.scan().primary::<#ty>()?;
                    let mut items = Vec::new();
                    for item_result in scan.all()? {
                        items.push(item_result?);
                    }
                    Ok(items)
                }
            }
        })
        .collect();

    // Generate collection calls for all types
    let collect_calls: Vec<TokenStream> = variant_info
        .iter()
        .map(|(variant_name, _ty)| {
            let method_name = syn::Ident::new(
                &format!("scan_{}", variant_name.to_string().to_lowercase()),
                proc_macro2::Span::call_site(),
            );
            quote! {
                for item in self.#method_name()? {
                    all_items.push(#base_enum_name::#variant_name(item));
                }
            }
        })
        .collect();

    // The struct with simplified lifetime
    let iter_struct = parse_quote! {
        pub struct #name<'db> {
            database: &'db native_db::Database<'db>,
        }
    };

    // Implementation with working iterator methods
    let iter_impl = parse_quote! {
        impl<'db> #name<'db> {
            pub fn new(database: &'db native_db::Database<'db>) -> Self {
                Self { database }
            }

            #(#iter_methods)*

            /// Collects all items from all types into a Vec as enum variants
            /// This resolves lifetime issues by owning the data
            pub fn scan_all_types(&self) -> native_db::db_type::Result<Vec<#base_enum_name>> {
                let mut all_items = Vec::new();

                #(#collect_calls)*

                Ok(all_items)
            }
        }
    };

    (iter_struct, iter_impl)
}

pub fn generate_model_specific_key_enums<'ast>(schema: &SchemaVisitor<'ast>) -> Vec<ItemEnum> {
    schema
        .items
        .iter()
        .map(|nm| {
            let key_enum_name = {
                let mut name = nm.model.ident.to_string();
                name.push_str("Key");
                Ident::new(&name, nm.model.ident.span())
            };

            let mut variants = vec![];

            // Primary key variant
            if let Some(primary_key_type) = &nm.primary_key_type {
                variants.push(Variant {
                    attrs: vec![],
                    ident: Ident::new("Primary", proc_macro2::Span::call_site()),
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
                                ty: primary_key_type.clone(),
                            });
                            punctuated
                        },
                    }),
                    discriminant: None,
                });
            }

            // Secondary key variants
            for (secondary_key_name, secondary_key_type) in &nm.secondary_keys {
                let variant_name = {
                    let name_str = secondary_key_name.to_string();
                    // Convert snake_case to PascalCase
                    let pascal_case = name_str
                        .split('_')
                        .map(|s| {
                            let mut chars = s.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => {
                                    first.to_uppercase().collect::<String>()
                                        + &chars.as_str().to_lowercase()
                                }
                            }
                        })
                        .collect::<String>();
                    Ident::new(&pascal_case, secondary_key_name.span())
                };

                variants.push(Variant {
                    attrs: vec![],
                    ident: variant_name,
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
                                ty: secondary_key_type.clone(),
                            });
                            punctuated
                        },
                    }),
                    discriminant: None,
                });
            }

            ItemEnum {
                attrs: vec![parse_quote!(#[derive(Debug, Clone)])],
                vis: syn::Visibility::Public(syn::token::Pub::default()),
                enum_token: syn::token::Enum::default(),
                ident: key_enum_name,
                generics: syn::Generics::default(),
                brace_token: syn::token::Brace::default(),
                variants: Punctuated::from_iter(variants),
            }
        })
        .collect()
}

pub fn generate_database_key_variants<'ast>(schema: &SchemaVisitor<'ast>) -> Vec<Variant> {
    schema
        .items
        .iter()
        .map(|nm| {
            let variant_name = &nm.model.ident;
            let key_enum_name = {
                let mut name = nm.model.ident.to_string();
                name.push_str("Key");
                Ident::new(&name, nm.model.ident.span())
            };

            Variant {
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
                            ty: parse_quote!(#key_enum_name),
                        });
                        punctuated
                    },
                }),
                discriminant: None,
            }
        })
        .collect()
}

pub fn generate_conversions<'ast>(db_name: &Ident, schema: &SchemaVisitor<'ast>) -> TokenStream {
    let base_enum_name = db_name;
    let ref_enum_name = {
        let mut name = db_name.to_string();
        name.push_str("Ref");
        Ident::new(&name, proc_macro2::Span::call_site())
    };

    // Generate From implementations for each model type to the base enum
    let from_impls: Vec<TokenStream> = schema
        .items
        .iter()
        .map(|nm| {
            let variant_name = &nm.model.ident;
            let model_path = nm.model_path();
            let model_type = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: model_path,
                },
            });

            quote! {
                impl From<#model_type> for #base_enum_name {
                    fn from(value: #model_type) -> Self {
                        #base_enum_name::#variant_name(value)
                    }
                }
            }
        })
        .collect();

    // Generate From implementations for references to the ref enum
    let ref_from_impls: Vec<TokenStream> = schema
        .items
        .iter()
        .map(|nm| {
            let variant_name = &nm.model.ident;
            let model_path = nm.model_path();
            let model_type = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: model_path,
                },
            });

            quote! {
                impl<'a> From<&'a #model_type> for #ref_enum_name<'a> {
                    fn from(value: &'a #model_type) -> Self {
                        #ref_enum_name::#variant_name(value)
                    }
                }
            }
        })
        .collect();

    // Generate conversion from base enum to ref enum
    let base_to_ref_arms: Vec<TokenStream> = schema
        .items
        .iter()
        .map(|nm| {
            let variant_name = &nm.model.ident;
            quote! {
                #base_enum_name::#variant_name(item) => #ref_enum_name::#variant_name(item),
            }
        })
        .collect();

    let base_to_ref_impl = quote! {
        impl<'a> From<&'a #base_enum_name> for #ref_enum_name<'a> {
            fn from(value: &'a #base_enum_name) -> Self {
                match value {
                    #(#base_to_ref_arms)*
                }
            }
        }
    };

    // Generate TryFrom implementations for extracting specific types
    let try_from_impls: Vec<TokenStream> = schema
        .items
        .iter()
        .map(|nm| {
            let variant_name = &nm.model.ident;
            let model_path = nm.model_path();
            let model_type = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: model_path,
                },
            });

            quote! {
                impl TryFrom<#base_enum_name> for #model_type {
                    type Error = #base_enum_name;

                    fn try_from(value: #base_enum_name) -> Result<Self, Self::Error> {
                        match value {
                            #base_enum_name::#variant_name(item) => Ok(item),
                            other => Err(other),
                        }
                    }
                }

                impl<'a> TryFrom<#ref_enum_name<'a>> for &'a #model_type {
                    type Error = #ref_enum_name<'a>;

                    fn try_from(value: #ref_enum_name<'a>) -> Result<Self, Self::Error> {
                        match value {
                            #ref_enum_name::#variant_name(item) => Ok(item),
                            other => Err(other),
                        }
                    }
                }
            }
        })
        .collect();

    quote! {
        #(#from_impls)*
        #(#ref_from_impls)*
        #base_to_ref_impl
        #(#try_from_impls)*
    }
}
