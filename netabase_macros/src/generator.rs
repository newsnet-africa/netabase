use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    ExprAssign, ExprLet, Field, Fields, FieldsUnnamed, GenericParam, Ident, ItemEnum, ItemImpl,
    ItemStruct, LifetimeParam, Token, Type, TypePath, Variant, parse_quote, punctuated::Punctuated,
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
        attrs: vec![
            parse_quote!(#[derive(derive_more::From, derive_more::TryInto, Debug, Clone, Copy)]),
        ],
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
    let _generics = &ref_enum.generics;
    let variant_map = ref_enum.variants.iter().filter_map(|v| {
        if let Fields::Unnamed(fields_unnamed) = &v.fields
            && let Some(first) = fields_unnamed.unnamed.first()
        {
            Some(first.ty.clone())
        } else {
            None
        }
    });

    let methods: Vec<TokenStream> = variant_map
        .clone()
        .enumerate()
        .map(|(ix, ty)| {
            let method_name =
                syn::Ident::new(&format!("scan_type_{ix}"), proc_macro2::Span::call_site());
            quote! {
                pub fn #method_name(&self) -> native_db::db_type::Result<Vec<#ty>> {
                    let scan = self.r_transaction.scan();
                    let mut results = Vec::new();
                    for item in scan.primary::<#ty>()?.all()? {
                        results.push(item?);
                    }
                    Ok(results)
                }
            }
        })
        .collect();

    (
        parse_quote! {
            pub struct #name<'db> {
                r_transaction: native_db::transaction::RTransaction<'db>,
            }
        },
        parse_quote! {
            impl<'db> #name<'db> {
                pub fn new(database: &'db native_db::Database<'db>) -> native_db::db_type::Result<Self> {
                    let r_transaction = database.r_transaction()?;
                    Ok(Self {
                        r_transaction,
                    })
                }

                #(#methods)*
            }
        },
    )
}
pub fn generate_model_specific_key_enums<'ast>(schema: &SchemaVisitor<'ast>) -> Vec<syn::ItemEnum> {
    let mut enums = Vec::new();

    for nm in &schema.items {
        let variant_name = &nm.model.ident;
        let model_key_enum_name = format!("{}Keys", variant_name);
        let model_key_ident = Ident::new(&model_key_enum_name, variant_name.span());

        // Get the primary key type for this model
        let primary_key_type = nm
            .primary_key_type
            .clone()
            .unwrap_or_else(|| parse_quote!(String));

        // Create Primary variant
        let primary_variant = Variant {
            attrs: vec![],
            ident: Ident::new("Primary", variant_name.span()),
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
                        ty: primary_key_type,
                    });
                    punctuated
                },
            }),
            discriminant: None,
        };

        let mut key_variants = syn::punctuated::Punctuated::new();
        key_variants.push(primary_variant);

        // Only add Secondary variant if there are secondary keys
        if !nm.secondary_keys.is_empty() {
            let secondary_enum_name = format!("{}SecondaryKeys", variant_name);
            let secondary_enum_ident = Ident::new(&secondary_enum_name, variant_name.span());

            let secondary_variant = Variant {
                attrs: vec![],
                ident: Ident::new("Secondary", variant_name.span()),
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
                            ty: parse_quote!(#secondary_enum_ident),
                        });
                        punctuated
                    },
                }),
                discriminant: None,
            };

            key_variants.push(secondary_variant);

            // Generate the secondary keys enum
            let mut secondary_variants = syn::punctuated::Punctuated::new();
            for (field_name, field_type) in &nm.secondary_keys {
                let variant_ident = Ident::new(&field_name.to_string(), field_name.span());

                let secondary_key_variant = Variant {
                    attrs: vec![],
                    ident: variant_ident,
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
                                ty: field_type.clone(),
                            });
                            punctuated
                        },
                    }),
                    discriminant: None,
                };

                secondary_variants.push(secondary_key_variant);
            }

            let secondary_enum = ItemEnum {
                attrs: vec![
                    parse_quote!(#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]),
                ],
                vis: syn::Visibility::Public(syn::token::Pub::default()),
                enum_token: syn::token::Enum::default(),
                ident: secondary_enum_ident,
                generics: syn::Generics::default(),
                brace_token: syn::token::Brace::default(),
                variants: secondary_variants,
            };

            enums.push(secondary_enum);
        }

        let model_key_enum = ItemEnum {
            attrs: vec![parse_quote!(#[derive(Debug, Clone, bincode::Encode, bincode::Decode)])],
            vis: syn::Visibility::Public(syn::token::Pub::default()),
            enum_token: syn::token::Enum::default(),
            ident: model_key_ident,
            generics: syn::Generics::default(),
            brace_token: syn::token::Brace::default(),
            variants: key_variants,
        };

        enums.push(model_key_enum);
    }

    enums
}

pub fn generate_database_key_variants<'ast>(schema: &SchemaVisitor<'ast>) -> Vec<syn::Variant> {
    let mut variants = Vec::new();

    for nm in &schema.items {
        let variant_name = &nm.model.ident;
        let model_key_enum_name = format!("{}Keys", variant_name);
        let model_key_ident = Ident::new(&model_key_enum_name, variant_name.span());

        let variant = Variant {
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
                        ty: parse_quote!(#model_key_ident),
                    });
                    punctuated
                },
            }),
            discriminant: None,
        };

        variants.push(variant);
    }

    variants
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

    let mut get_key_arms = Vec::new();
    let mut record_conversion_impls = Vec::new();
    let mut catalog_constructor_impls = Vec::new();
    let mut from_owned_arms = Vec::new();
    let mut from_ref_arms = Vec::new();
    let mut downcast_arms = Vec::new();
    let mut variant_names = Vec::new();

    for nm in &schema.items {
        let variant_name = &nm.model.ident;
        let model_path = nm.model_path();

        variant_names.push(variant_name.clone());

        // Get the primary key field name
        let primary_key_field = if let Some(_primary_key_type) = &nm.primary_key_type {
            let mut field_name = None;
            for field in &nm.model.fields {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("primary_key"))
                {
                    if let Some(ref ident) = field.ident {
                        field_name = Some(ident.clone());
                        break;
                    }
                }
            }
            field_name.unwrap_or_else(|| Ident::new("id", variant_name.span()))
        } else {
            Ident::new("id", variant_name.span())
        };

        let model_key_enum_name = format!("{}Keys", variant_name);
        let model_key_ident = Ident::new(&model_key_enum_name, variant_name.span());

        // Generate GetKey implementation for individual models
        let key_to_bytes_impl = if !nm.secondary_keys.is_empty() {
            quote::quote! {
                match key {
                    #model_key_ident::Primary(k) => {
                        use native_db::ToKey;
                        let native_key = k.to_key();
                        ::netabase::native_db_key_to_bytes(&native_key)
                    },
                    #model_key_ident::Secondary(sk_enum) => {
                        // For now, secondary keys need custom implementation
                        todo!("Secondary key conversion not yet implemented")
                    },
                }
            }
        } else {
            quote::quote! {
                match key {
                    #model_key_ident::Primary(k) => {
                        use native_db::ToKey;
                        let native_key = k.to_key();
                        ::netabase::native_db_key_to_bytes(&native_key)
                    },
                }
            }
        };

        record_conversion_impls.push(quote::quote! {
            impl ::netabase::GetKey for #model_path {
                type KeyType = #model_key_ident;

                fn key(&self) -> Self::KeyType {
                    #model_key_ident::Primary(self.#primary_key_field.clone())
                }
            }

            impl ::netabase::RecordConversion for #model_path {
                fn calculate_expiry(&self) -> Option<std::time::Instant> {
                    // Default: no expiry. Override in specific implementations if needed.
                    None
                }

                fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
                    #key_to_bytes_impl
                }

                fn bytes_to_key(bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
                    use ::bincode::Decode;
                    let (key, _): (#model_key_ident, usize) =
                        ::bincode::decode_from_slice(bytes, ::bincode::config::standard())?;
                    Ok(key)
                }
            }
        });

        // Generate GetKey implementation for schema enum
        get_key_arms.push(quote::quote! {
            #db_name::#variant_name(data) => {
                #key_name::#variant_name(data.key())
            },
        });

        // Generate From<&Owned> for Ref
        from_owned_arms.push(quote::quote! {
            #db_name::#variant_name(data) => #ref_name::#variant_name(data),
        });

        // Generate From<Ref> for Record
        from_ref_arms.push(quote::quote! {
            #ref_name::#variant_name(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes = <#model_path as ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = #db_name::#variant_name(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            },
        });

        // Generate downcast checks for TryFrom native_db
        downcast_arms.push(quote::quote! {
            if let Some(typed_data) = any_data.downcast_ref::<#model_path>() {
                return Some(#ref_name::#variant_name(typed_data));
            }
        });

        // Generate CatalogConstructor implementations
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

    let first_variant_name = if let Some(first_variant) = variant_names.first() {
        first_variant.clone()
    } else {
        return syn::Error::new_spanned(db_name, "No models found").to_compile_error();
    };

    let first_model_key_enum_name = format!("{}Keys", first_variant_name);
    let first_model_key_ident = Ident::new(&first_model_key_enum_name, first_variant_name.span());

    let bincode_encode_arms = variant_names
        .iter()
        .enumerate()
        .map(|(idx, variant_name)| {
            let idx_u8 = idx as u8;
            quote::quote! {
                #key_name::#variant_name(key) => {
                    ::bincode::Encode::encode(&#idx_u8, encoder)?;
                    ::bincode::Encode::encode(key, encoder)
                },
            }
        })
        .collect::<Vec<_>>();

    let bincode_decode_arms = variant_names
        .iter()
        .enumerate()
        .map(|(idx, variant_name)| {
            let idx_u8 = idx as u8;
            quote::quote! {
                #idx_u8 => {
                    let key = ::bincode::Decode::decode(decoder)?;
                    Ok(#key_name::#variant_name(key))
                },
            }
        })
        .collect::<Vec<_>>();

    quote::quote! {
        // Individual model implementations
        #(#record_conversion_impls)*

        // GetKey implementation for schema enum
        impl ::netabase::GetKey for #db_name {
            type KeyType = #key_name;

            fn key(&self) -> Self::KeyType {
                match self {
                    #(#get_key_arms)*
                }
            }
        }

        // RecordConversion implementation for schema enum
        impl ::netabase::RecordConversion for #db_name {
            fn calculate_expiry(&self) -> Option<std::time::Instant> {
                match self {
                    #(#db_name::#variant_names(data) => data.calculate_expiry(),)*
                }
            }

            fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
                match key {
                    #(#key_name::#variant_names(k) => {
                        // Delegate to the individual model's implementation
                        // For now, use a simple encoding
                        use ::bincode::Encode;
                        ::bincode::encode_to_vec(k, ::bincode::config::standard()).unwrap_or_default()
                    },)*
                }
            }

            fn bytes_to_key(bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
                // For now, default to the first variant - this needs improvement for production
                use ::bincode::Decode;
                let (key, _): (#first_model_key_ident, usize) =
                    ::bincode::decode_from_slice(bytes, ::bincode::config::standard())?;
                Ok(#key_name::#first_variant_name(key))
            }
        }

        // ThreadSafe is automatically implemented via blanket impl

        // Manual implementation of bincode traits for the key enum
        impl ::bincode::Encode for #key_name {
            fn encode<__E: ::bincode::enc::Encoder>(
                &self,
                encoder: &mut __E,
            ) -> core::result::Result<(), ::bincode::error::EncodeError> {
                match self {
                    #(#bincode_encode_arms)*
                }
            }
        }

        impl<Context> ::bincode::Decode<Context> for #key_name {
            fn decode<__D: ::bincode::de::Decoder<Context = Context>>(
                decoder: &mut __D,
            ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
                let discriminant: u8 = ::bincode::Decode::decode(decoder)?;
                match discriminant {
                    #(#bincode_decode_arms)*
                    _ => Err(::bincode::error::DecodeError::UnexpectedEnd { additional: 0 }),
                }
            }
        }


        // From owned enum to ref enum
        impl<'a> From<&'a #db_name> for #ref_name<'a> {
            fn from(owned: &'a #db_name) -> Self {
                match owned {
                    #(#from_owned_arms)*
                }
            }
        }

        // From ref enum to Record
        impl<'a> From<#ref_name<'a>> for ::netabase::Record<#db_name> {
            fn from(ref_enum: #ref_name<'a>) -> Self {
                match ref_enum {
                    #(#from_ref_arms)*
                }
            }
        }

        // FromNativeDb implementation for ref enum
        impl<'a> ::netabase::FromNativeDb<'a> for #ref_name<'a> {
            fn try_from_native_db<T: ::native_db::ToInput + 'a>(data: &'a T) -> Option<Self>
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
