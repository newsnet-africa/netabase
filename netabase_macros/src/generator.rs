use syn::{
    Field, Fields, FieldsUnnamed, GenericParam, Ident, ItemEnum, LifetimeParam, Token, Type,
    TypePath, Variant, parse_quote, punctuated::Punctuated,
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

        // For keys, we'll use a variant that stores the catalog variant type
        // since each catalog item has its own key based on the data
        let key_variant_name = {
            let mut name = variant_name.to_string();
            name.push_str("Key");
            Ident::new(&name, variant_name.span())
        };
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
                        ty: parse_quote!(::netabase::SerializableKey),
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
        attrs: vec![
            parse_quote!(#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]),
            parse_quote!(#[derive(derive_more::From, derive_more::TryInto)]),
        ],
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
        attrs: vec![parse_quote!(#[derive(Debug, Clone, bincode::Encode, bincode::Decode)])],
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

pub fn generate_conversions<'ast>(
    db_name: &Ident,
    schema: &SchemaVisitor<'ast>,
) -> proc_macro2::TokenStream {
    let ref_name = {
        let mut old_name = db_name.to_string();
        old_name.push_str("Ref");
        Ident::new(&old_name, proc_macro2::Span::call_site())
    };

    let key_name = {
        let mut temp_name = db_name.to_string();
        temp_name.push_str("Key");
        Ident::new(&temp_name, proc_macro2::Span::call_site())
    };

    let mut from_owned_arms = Vec::new();
    let mut from_ref_arms = Vec::new();
    let mut catalog_key_arms = Vec::new();
    let mut key_to_native_db_arms = Vec::new();
    let mut key_to_serializable_arms = Vec::new();
    let mut downcast_arms = Vec::new();
    let mut catalog_constructor_impls = Vec::new();
    let mut key_variant_names = Vec::new();

    for nm in &schema.items {
        let variant_name = &nm.model.ident;
        let model_path = nm.model_path();
        let key_variant_name = {
            let mut name = variant_name.to_string();
            name.push_str("Key");
            Ident::new(&name, variant_name.span())
        };

        // Store key variant name for later use
        key_variant_names.push(key_variant_name.clone());

        // Generate From<&Owned> for Ref
        from_owned_arms.push(quote::quote! {
            #db_name::#variant_name(data) => #ref_name::#variant_name(data),
        });

        // Generate CatalogKey implementation using native_db's primary key
        catalog_key_arms.push(quote::quote! {
            #db_name::#variant_name(data) => {
                let native_key = data.native_db_primary_key();
                let type_hint = format!("{}::{}", stringify!(#db_name), stringify!(#variant_name));
                let serializable_key = ::netabase::SerializableKey::from_native_db_key_with_hint(&native_key, type_hint);
                #key_name::#key_variant_name(serializable_key)
            },
        });

        // Generate key to bytes conversion (legacy)
        key_to_native_db_arms.push(quote::quote! {
            #key_name::#key_variant_name(serializable_key) => serializable_key.as_bytes().to_vec(),
        });

        // Generate key to SerializableKey conversion
        key_to_serializable_arms.push(quote::quote! {
            #key_name::#key_variant_name(serializable_key) => serializable_key.clone(),
        });

        // Generate From<Ref> for Record using bincode and SerializableKey
        from_ref_arms.push(quote::quote! {
            #ref_name::#variant_name(data) => {
                let native_key = data.native_db_primary_key();
                let type_hint = format!("{}::{}", stringify!(#db_name), stringify!(#variant_name));
                let serializable_key = ::netabase::SerializableKey::from_native_db_key_with_hint(&native_key, type_hint);
                let catalog_data = #db_name::#variant_name(data.clone());
                ::netabase::Record::from_serializable_key(serializable_key, catalog_data)
            },
        });

        // Generate downcast checks for TryFrom native_db
        downcast_arms.push(quote::quote! {
            if let Some(typed_data) = any_data.downcast_ref::<#model_path>() {
                return Some(#ref_name::#variant_name(typed_data));
            }
        });

        // Generate CatalogConstructor implementations for each variant
        catalog_constructor_impls.push(quote::quote! {
            impl ::netabase::CatalogConstructor<#model_path> for #db_name {
                fn from_native_db(data: #model_path) -> Self {
                    #db_name::#variant_name(data)
                }

                fn to_native_db(self) -> #model_path {
                    match self {
                        #db_name::#variant_name(data) => data,
                        _ => panic!("Cannot convert {} variant to {}", stringify!(#db_name), stringify!(#model_path)),
                    }
                }
            }
        });
    }

    // Get the first key variant for the bytes_to_key fallback
    let first_key_variant = if let Some(first_variant) = key_variant_names.first() {
        first_variant.clone()
    } else {
        return quote::quote! {
            compile_error!("No key variants found for schema");
        };
    };

    quote::quote! {
        // Import required traits
        use ::native_db::db_type::ToInput;

        // NetabaseCatalog trait implementation
        impl ::netabase::NetabaseCatalog for #db_name {
            type RefCatalog<'a> = #ref_name<'a>;
        }

        // NetabaseRefCatalog trait implementation
        impl<'a> ::netabase::NetabaseRefCatalog<'a> for #ref_name<'a> {}

        // From owned enum to ref enum
        impl<'a> From<&'a #db_name> for #ref_name<'a> {
            fn from(owned: &'a #db_name) -> Self {
                match owned {
                    #(#from_owned_arms)*
                }
            }
        }

        // CatalogKey implementation using native_db keys
        impl ::netabase::CatalogKey for #db_name {
            type KeyType = #key_name;

            fn catalog_key(&self) -> Self::KeyType {
                match self {
                    #(#catalog_key_arms)*
                }
            }

            fn key_to_serializable(key: &Self::KeyType) -> ::netabase::SerializableKey {
                match key {
                    #(#key_to_serializable_arms)*
                }
            }

            fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
                Self::key_to_serializable(key).as_bytes().to_vec()
            }

            fn bytes_to_key(bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
                let serializable_key = ::netabase::SerializableKey {
                    key_bytes: bytes.to_vec(),
                    type_hint: None,
                };
                // Use first variant as default - in a more sophisticated implementation,
                // you would need type discrimination to determine the correct variant
                Ok(#key_name::#first_key_variant(serializable_key))
            }

            fn serializable_to_key(key: &::netabase::SerializableKey) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
                // For now, use the first variant as default - in production you'd want to use the type_hint
                Ok(#key_name::#first_key_variant(key.clone()))
            }
        }

        // NetabaseRecordExt implementation for the main enum
        impl ::netabase::NetabaseRecordExt for #db_name {}

        // From ref enum to Record using bincode and native_db keys
        impl<'a> From<#ref_name<'a>> for ::netabase::Record<#db_name> {
            fn from(ref_enum: #ref_name<'a>) -> Self {
                match ref_enum {
                    #(#from_ref_arms)*
                }
            }
        }

        // AsKadRecord implementation for ref enum
        impl<'a> ::netabase::AsKadRecord for #ref_name<'a> {
            fn as_kad_record(&self) -> ::std::borrow::Cow<'_, ::netabase::KadRecord> {
                let record: ::netabase::Record<#db_name> = (*self).into();
                ::std::borrow::Cow::Owned(record.into())
            }
        }



        // TryFrom native_db types for ref enum
        impl<'a> #ref_name<'a> {
            pub fn try_from_native_db<T: ::native_db::ToInput + 'a>(data: &'a T) -> Option<Self>
            where
                T: ::std::any::Any,
            {
                let any_data = data as &dyn ::std::any::Any;
                #(#downcast_arms)*
                None
            }
        }

        // CatalogConstructor implementations for each variant
        #(#catalog_constructor_impls)*
    }
}
