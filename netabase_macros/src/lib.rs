#![feature(extend_one)]
#![feature(box_into_inner)]

use proc_macro::{Span, TokenStream};
use proc_macro2::{Group, TokenStream as TokenStream2};
// use proc_macro::quote;
use quote::{ToTokens, quote};
use syn::{
    Generics, Ident, Item, ItemMod, Path, WhereClause,
    punctuated::Punctuated,
    token::{Gt, Lt},
    visit::Visit,
};

mod visitors;

use crate::visitors::{SchemaValidator, ValidSchemaFinder};

#[proc_macro_attribute]
pub fn schemas(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module_tokes = proc_macro2::TokenStream::from(item.clone());
    let schema_module = syn::parse_macro_input!(item as ItemMod);
    let schemas: Vec<((Ident, Path), Generics)> = {
        let mut visitor = ValidSchemaFinder::default();
        visitor.visit_item_mod(&schema_module);
        visitor
            .valid_schemas
            .iter()
            .filter_map(|(i, p)| match i {
                Item::Enum(item_enum) => Some((
                    (item_enum.ident.clone(), p.clone()),
                    item_enum.generics.clone(),
                )),
                Item::Struct(item_struct) => Some((
                    (item_struct.ident.clone(), p.clone()),
                    item_struct.generics.clone(),
                )),
                _ => None,
            })
            .collect()
    };
    let schema_variants = schemas.iter().map(|((i, p), generics)| {
        let full_struct = {
            let names = { generics.type_params().map(|tp| tp.ident.clone()) };
            quote! {#p<#(#names), *>}
        };

        syn::Variant {
            attrs: vec![],
            ident: i.clone(),
            fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
                paren_token: syn::token::Paren {
                    span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                        .delim_span(),
                },
                unnamed: {
                    let mut path_field = Punctuated::new();
                    path_field.push(syn::Field {
                        attrs: vec![],
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: syn::Type::Verbatim(full_struct),
                    });
                    path_field
                },
            }),
            discriminant: None,
        }
    });

    let full_generics = {
        Generics {
            lt_token: Some(Lt {
                spans: [Span::call_site().into()],
            }),
            params: Punctuated::from_iter(schemas.iter().flat_map(|(_, g)| g.params.clone())),
            gt_token: Some(Gt {
                spans: [Span::call_site().into()],
            }),
            where_clause: Some(WhereClause {
                where_token: syn::token::Where {
                    span: Span::call_site().into(),
                },
                predicates: Punctuated::from_iter(
                    schemas
                        .iter()
                        .filter_map(|(_, g)| {
                            g.where_clause
                                .as_ref()
                                .map(|where_clause| where_clause.predicates.clone())
                        })
                        .flatten(),
                ),
            }),
        }
    };
    let wh = full_generics.where_clause.clone().unwrap();

    quote! {
        #module_tokes
        pub enum NetabaseSchema #full_generics #wh{
            #(#schema_variants),*
        }

    }
    .into()
}

/// Derive macro for NetabaseSchema that uses the visitor pattern to analyze struct/enum fields
/// and generate a stringify!() list of key fields.
///
/// This macro:
/// 1. Uses a `SchemaValidator` visitor to traverse the input struct/enum and discover all fields marked with `#[key]`
/// 2. Extracts field names from the discovered key fields
/// 3. Creates a stringify!() list of these field names (e.g., `[stringify!(id), stringify!(name)]`)
/// 4. Prints this list to stdout during compilation for debugging/inspection
/// 5. Returns a generated constant containing the stringify!() list as a comment
///
/// ## Example
///
/// ```rust
/// use netabase_macros::NetabaseSchema;
///
/// #[derive(NetabaseSchema)]
/// struct User {
///     #[key]
///     id: String,
///     #[key]
///     email: String,
///     name: String,
/// }
/// ```
///
/// This will:
/// - Print: `Key fields stringify!() list: [stringify!(id), stringify!(email)]`
/// - Generate: `const _NETABASE_SCHEMA_KEYS: &str = "/* Key fields: [stringify!(id), stringify!(email)] */";`
///
/// ## Supported Key Field Types
///
/// - Named fields: `#[key] field_name: Type`
/// - Tuple fields: `#[key] Type` (shows as `stringify!(unnamed_field)`)
/// - Closure-based keys: `#[key = |item| transform(item)] field: Type`
///
/// ## Visitor Pattern
///
/// The macro uses `syn::visit::Visit` trait to traverse the AST:
/// - `SchemaValidator` visits items, fields, structs, and enums
/// - Collects all fields with `#[key]` attributes into `key_fields` vector
/// - Handles both struct fields and enum variant fields
#[proc_macro_derive(NetabaseSchema, attributes(key, netabase))]
pub fn netabase_schema(input: TokenStream) -> TokenStream {
    let schema = syn::parse_macro_input!(input as Item);
    let mut visitor = SchemaValidator::default();

    let ident = {
        match &schema {
            Item::Enum(item_enum) => item_enum.ident.clone(),
            Item::Struct(item_struct) => item_struct.ident.clone(),
            _ => Ident::new("Weird", Span::call_site().into()),
        }
    };

    // Visit the item to validate it
    visitor.visit_item(&schema);

    // Validate key field constraints
    match &schema {
        Item::Struct(item_struct) => {
            // Check for outer key generator (struct-level closure)
            let has_outer_key = item_struct.attrs.iter().any(|attr| match &attr.meta {
                syn::Meta::NameValue(name_value) => {
                    name_value.path.is_ident("key")
                        && matches!(&name_value.value, syn::Expr::Closure(_))
                }
                _ => false,
            });

            if !has_outer_key {
                let key_fields = count_key_fields(&item_struct.fields);
                if key_fields == 0 {
                    return quote! {
                        compile_error!(concat!(
                            "Schema '", stringify!(#ident), "' has no key fields. ",
                            "Each schema must have at least one field marked with #[key] or #[key = closure], or the struct must have a top-level #[key = closure] attribute."
                        ));
                    }.into();
                } else if key_fields > 1 {
                    return quote! {
                        compile_error!(concat!(
                            "Schema '", stringify!(#ident), "' has multiple key fields. ",
                            "Structs can have at most 1 key field unless a top-level #[key = closure] is present."
                        ));
                    }
                    .into();
                }
            }
            // If has_outer_key, skip field-level key checks
        }
        Item::Enum(item_enum) => {
            // Check for enum-level key closure
            let has_outer_key = item_enum.attrs.iter().any(|attr| match &attr.meta {
                syn::Meta::NameValue(name_value) => {
                    name_value.path.is_ident("key")
                        && matches!(&name_value.value, syn::Expr::Closure(_))
                }
                _ => false,
            });

            let enum_key_closure = item_enum.attrs.iter().find_map(|attr| {
                if let syn::Meta::NameValue(name_value) = &attr.meta
                    && name_value.path.is_ident("key")
                    && let syn::Expr::Closure(closure) = &name_value.value
                {
                    return Some(closure);
                }
                None
            });

            if let Some(closure) = enum_key_closure {
                // Store the closure for use in key extraction
                visitor.enum_key_closure = Some(closure);
            }

            if !has_outer_key {
                // Validate each variant has exactly one key field
                for variant in &item_enum.variants {
                    let key_fields = count_key_fields(&variant.fields);
                    if key_fields == 0 && !matches!(variant.fields, syn::Fields::Unit) {
                        let variant_ident = &variant.ident;
                        return quote! {
                            compile_error!(concat!(
                                "Enum variant '", stringify!(#ident), "::", stringify!(#variant_ident),
                                "' has no key fields. Each enum variant must have exactly one field marked with #[key] ",
                                "or the enum must have a top-level #[key = closure] attribute."
                            ));
                        }.into();
                    } else if key_fields > 1 {
                        let variant_ident = &variant.ident;
                        return quote! {
                            compile_error!(concat!(
                                "Enum variant '", stringify!(#ident), "::", stringify!(#variant_ident),
                                "' has multiple key fields. Enum variants can have at most 1 key field unless a top-level #[key = closure] is present."
                            ));
                        }.into();
                    }
                }
            }
            // If has_outer_key, skip variant-level key checks
        }
        _ => {
            return quote! {
                compile_error!("NetabaseSchema can only be derived for structs and enums");
            }
            .into();
        }
    }

    let keys = visitor.key_fields;

    // Generate the key type name
    let key_type_name = Ident::new(&format!("{}Key", ident), Span::call_site().into());

    // Check if serde compatibility is requested
    let use_serde = check_serde_compatibility(&schema);

    // Create key struct definition using syn
    let key_struct_attrs = if use_serde {
        vec![
            syn::parse_quote!(#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]),
            syn::parse_quote!(#[serde(transparent)]),
            syn::parse_quote!(#[repr(transparent)]),
        ]
    } else {
        vec![
            syn::parse_quote!(#[derive(Clone, Debug, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]),
            syn::parse_quote!(#[repr(transparent)]),
        ]
    };

    let key_struct = syn::ItemStruct {
        attrs: key_struct_attrs,
        vis: syn::Visibility::Public(syn::token::Pub {
            span: Span::call_site().into(),
        }),
        struct_token: syn::token::Struct {
            span: Span::call_site().into(),
        },
        ident: key_type_name.clone(),
        generics: syn::Generics::default(),
        fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
            paren_token: syn::token::Paren {
                span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                    .delim_span(),
            },
            unnamed: {
                let mut fields = Punctuated::new();
                fields.push(syn::Field {
                    attrs: vec![],
                    vis: syn::Visibility::Public(syn::token::Pub {
                        span: Span::call_site().into(),
                    }),
                    mutability: syn::FieldMutability::None,
                    ident: None,
                    colon_token: None,
                    ty: syn::parse_quote!(String),
                });
                fields
            },
        }),
        semi_token: Some(syn::token::Semi {
            spans: [Span::call_site().into()],
        }),
    };

    // Create NetabaseSchemaKey implementation using syn

    // Generate additional implementations
    let key_utility_impl = generate_key_utility_impl(&key_type_name);
    let schema_key_impl = generate_schema_key_impl(
        &ident,
        &key_type_name,
        &schema,
        &keys,
        visitor.enum_key_closure,
    );
    // Manual bincode implementations removed - now using derives
    let key_try_from_impls = generate_key_from_impls(&key_type_name, use_serde);
    // Removed sealed trait implementation - sealed module doesn't exist
    let _key_send_sync_impl = generate_key_send_sync_impl(&key_type_name);
    let _key_sync_impl = generate_key_sync_impl(&key_type_name);
    let _key_unpin_impl = generate_key_unpin_impl(&key_type_name);
    let key_additional_traits = if use_serde {
        generate_key_additional_traits_with_serde(&key_type_name)
    } else {
        generate_key_additional_traits(&key_type_name)
    };
    let key_sealed_marker_traits = generate_key_sealed_marker_traits(&key_type_name);
    // Removed conflicting TryFrom implementation to avoid blanket impl conflicts
    let schema_from_record_impl = generate_schema_from_record_impl(&ident, use_serde);
    let schema_utility_impl = generate_schema_utility_impl(&ident);
    // Removed sealed trait implementation - sealed module doesn't exist
    let _schema_send_sync_impl = generate_schema_send_sync_impl(&ident);
    let _schema_sync_impl = generate_schema_sync_impl(&ident);
    let _schema_unpin_impl = generate_schema_unpin_impl(&ident);
    let schema_additional_traits = if use_serde {
        generate_schema_additional_traits_with_serde(&ident)
    } else {
        generate_schema_additional_traits(&ident)
    };
    let _schema_sealed_marker_traits = generate_schema_sealed_marker_traits(&ident);
    let schema_bincode_impls = if use_serde {
        generate_schema_bincode_impls_with_serde(&ident)
    } else {
        // Native bincode will be handled by derives on the original struct
        None
    };
    let record_from_schema_impl = generate_record_from_schema_impl(&ident);
    let record_key_from_key_impl = generate_record_key_from_key_impl(&key_type_name);

    // Combine all generated items
    let mut output = TokenStream2::new();
    key_struct.to_tokens(&mut output);
    key_utility_impl.to_tokens(&mut output);
    // Manual bincode implementations removed - now using derives
    key_try_from_impls.to_tokens(&mut output);
    // Skip Send/Sync/Unpin implementations to avoid conflicts with derives
    for trait_impl in key_additional_traits {
        trait_impl.to_tokens(&mut output);
    }
    for trait_impl in key_sealed_marker_traits {
        trait_impl.to_tokens(&mut output);
    }
    schema_key_impl.to_tokens(&mut output);
    // Removed conflicting TryFrom implementation to avoid blanket impl conflicts
    schema_from_record_impl.to_tokens(&mut output);
    record_from_schema_impl.to_tokens(&mut output);
    record_key_from_key_impl.to_tokens(&mut output);
    schema_utility_impl.to_tokens(&mut output);
    // Skip Send/Sync/Unpin implementations to avoid conflicts with derives
    for trait_impl in schema_additional_traits {
        trait_impl.to_tokens(&mut output);
    }
    // Skip sealed marker traits to avoid conflicts with derives
    if let Some((encode_impl, decode_impl)) = schema_bincode_impls {
        encode_impl.to_tokens(&mut output);
        decode_impl.to_tokens(&mut output);
    }

    // Note: Derive macros should not re-output the original type definition
    // The user's original type should already have the necessary derives
    // If bincode traits are missing, add them to the user's derive list

    output.into()
}

/// Generate the key extraction block using syn structures
fn generate_key_extraction_block(
    schema: &Item,
    keys: &[(&syn::Field, visitors::KeyGenerator)],
    ident: &Ident,
    key_type_name: &Ident,
    enum_key_closure: Option<&syn::ExprClosure>,
) -> syn::Block {
    match schema {
        Item::Struct(_) => {
            if let Some(&(key_field, _)) = keys.first() {
                match &key_field.ident {
                    Some(field_name) => syn::parse_quote!({
                        #key_type_name(format!("{}", self.#field_name))
                    }),
                    None => syn::parse_quote!({
                        #key_type_name(format!("{:?}", self.0))
                    }),
                }
            } else {
                syn::parse_quote!({
                    #key_type_name(String::new())
                })
            }
        }
        Item::Enum(item_enum) => {
            // Check if there's a top-level enum closure
            if let Some(closure) = enum_key_closure {
                // Use the enum-level closure for key generation
                return syn::parse_quote!({
                    let closure_result = (#closure)(self);
                    #key_type_name(format!("{}", closure_result))
                });
            }

            let mut match_arms: Vec<syn::Arm> = Vec::new();

            for variant in &item_enum.variants {
                let variant_name = &variant.ident;
                let key_fields_count = count_key_fields(&variant.fields);

                if key_fields_count > 0 {
                    match &variant.fields {
                        syn::Fields::Named(fields) => {
                            if let Some(key_field) =
                                fields.named.iter().find(|f| has_key_attribute(f))
                            {
                                let field_name = key_field.ident.as_ref().unwrap();
                                match_arms.push(syn::parse_quote!(
                                    #ident::#variant_name { #field_name, .. } => {
                                        #key_type_name(format!("{}", #field_name))
                                    }
                                ));
                            } else {
                                match_arms.push(syn::parse_quote!(
                                    #ident::#variant_name { .. } => {
                                        #key_type_name(String::new())
                                    }
                                ));
                            }
                        }
                        syn::Fields::Unnamed(fields) => {
                            if fields.unnamed.iter().any(has_key_attribute) {
                                match_arms.push(syn::parse_quote!(
                                    #ident::#variant_name(key_value, ..) => {
                                        #key_type_name(format!("{}", key_value))
                                    }
                                ));
                            } else {
                                match_arms.push(syn::parse_quote!(
                                    #ident::#variant_name(..) => {
                                        #key_type_name(String::new())
                                    }
                                ));
                            }
                        }
                        syn::Fields::Unit => {
                            match_arms.push(syn::parse_quote!(
                                #ident::#variant_name => {
                                    #key_type_name(String::from(stringify!(#variant_name)))
                                }
                            ));
                        }
                    }
                } else {
                    match_arms.push(syn::parse_quote!(
                        #ident::#variant_name => {
                            #key_type_name(String::new())
                        }
                    ));
                }
            }

            syn::parse_quote!({
                match self {
                    #(#match_arms),*
                }
            })
        }
        _ => syn::parse_quote!({
            #key_type_name(String::new())
        }),
    }
}

/// Generate utility methods for the key type
fn generate_key_utility_impl(key_type_name: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: None,
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(key_type_name.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![
            syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                }),
                defaultness: None,
                sig: syn::parse_quote!(fn new(value: String) -> Self),
                block: syn::parse_quote!({ Self(value) }),
            }),
            syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                }),
                defaultness: None,
                sig: syn::parse_quote!(fn as_str(&self) -> &str),
                block: syn::parse_quote!({ &self.0 }),
            }),
            syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                }),
                defaultness: None,
                sig: syn::parse_quote!(fn into_string(self) -> String),
                block: syn::parse_quote!({ self.0 }),
            }),
            syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                }),
                defaultness: None,
                sig: syn::parse_quote!(fn generate_key() -> Self),
                block: syn::parse_quote!({
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    use std::time::{SystemTime, UNIX_EPOCH};

                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();

                    let mut hasher = DefaultHasher::new();
                    timestamp.hash(&mut hasher);
                    let hash = hasher.finish();

                    Self(format!("{}_{}", timestamp, hash))
                }),
            }),
        ],
    }
}

/// Generate key method implementation for the schema type
fn generate_schema_key_impl(
    ident: &Ident,
    key_type_name: &Ident,
    schema: &Item,
    keys: &[(&syn::Field, visitors::KeyGenerator)],
    enum_key_closure: Option<&syn::ExprClosure>,
) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: None,
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Public(syn::token::Pub {
                span: Span::call_site().into(),
            }),
            defaultness: None,
            sig: syn::parse_quote!(fn key(&self) -> #key_type_name),
            block: generate_key_extraction_block(
                schema,
                keys,
                ident,
                key_type_name,
                enum_key_closure,
            ),
        })],
    }
}

/// Check if the schema should use serde compatibility
fn check_serde_compatibility(schema: &Item) -> bool {
    let attrs = match schema {
        Item::Struct(s) => &s.attrs,
        Item::Enum(e) => &e.attrs,
        _ => return false,
    };

    attrs.iter().any(|attr| {
        if let syn::Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("netabase") {
                return attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("serde") {
                            Ok(())
                        } else {
                            Err(syn::Error::new_spanned(&meta.path, "expected 'serde'"))
                        }
                    })
                    .is_ok();
            }
        }
        false
    })
}

/// Inject bincode derives into the original schema
fn inject_bincode_derives(schema: Item) -> Item {
    match schema {
        Item::Struct(mut item_struct) => {
            // Add bincode derives to existing derives or create new derive attribute
            let bincode_derives = syn::parse_quote!(#[derive(bincode::Encode, bincode::Decode)]);

            // Check if there's already a derive attribute we can extend
            let mut found_derive = false;
            for attr in &mut item_struct.attrs {
                if let syn::Meta::List(meta_list) = &mut attr.meta {
                    if meta_list.path.is_ident("derive") {
                        // Add bincode traits to existing derive
                        let existing_tokens = &meta_list.tokens;
                        meta_list.tokens =
                            syn::parse_quote!(#existing_tokens, bincode::Encode, bincode::Decode);
                        found_derive = true;
                        break;
                    }
                }
            }

            // If no derive attribute found, add a new one
            if !found_derive {
                item_struct.attrs.push(bincode_derives);
            }

            Item::Struct(item_struct)
        }
        Item::Enum(mut item_enum) => {
            // Add bincode derives to existing derives or create new derive attribute
            let bincode_derives = syn::parse_quote!(#[derive(bincode::Encode, bincode::Decode)]);

            // Check if there's already a derive attribute we can extend
            let mut found_derive = false;
            for attr in &mut item_enum.attrs {
                if let syn::Meta::List(meta_list) = &mut attr.meta {
                    if meta_list.path.is_ident("derive") {
                        // Add bincode traits to existing derive
                        let existing_tokens = &meta_list.tokens;
                        meta_list.tokens =
                            syn::parse_quote!(#existing_tokens, bincode::Encode, bincode::Decode);
                        found_derive = true;
                        break;
                    }
                }
            }

            // If no derive attribute found, add a new one
            if !found_derive {
                item_enum.attrs.push(bincode_derives);
            }

            Item::Enum(item_enum)
        }
        other => other, // Return unchanged for other item types
    }
}

/// Generate bincode implementations for the schema type using serde compatibility
/// Only used when serde compatibility is explicitly requested
fn generate_schema_bincode_impls_with_serde(
    ident: &Ident,
) -> Option<(syn::ItemImpl, syn::ItemImpl)> {
    let encode_impl = syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(bincode::Encode),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse_quote!(fn encode<E: bincode::enc::Encoder>(
                &self,
                encoder: &mut E,
            ) -> Result<(), bincode::error::EncodeError>),
            block: syn::parse_quote!({
                use bincode::serde::Compat;
                Compat(self).encode(encoder)
            }),
        })],
    };

    let decode_impl = syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(bincode::Decode<()>),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse_quote!(fn decode<D: bincode::de::Decoder>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError>),
            block: syn::parse_quote!({
                use bincode::Decode;
                use bincode::serde::Compat;
                Compat::<Self>::decode(decoder).map(|compat| compat.0)
            }),
        })],
    };

    Some((encode_impl, decode_impl))
}

/// Generate From implementations for the key type
fn generate_key_from_impls(key_type_name: &Ident, use_serde: bool) -> syn::ItemImpl {
    let block_impl = if use_serde {
        syn::parse_quote!({
            let key_bytes = record.key.to_vec();
            let key_string = String::from_utf8(key_bytes).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Invalid UTF-8 in key for {}, using fallback: {}",
                    stringify!(#key_type_name),
                    e
                );
                String::new()
            });
            Self(key_string)
        })
    } else {
        syn::parse_quote!({
            let key_bytes = record.key.to_vec();
            let key_string = String::from_utf8(key_bytes).expect(&format!(
                "Invalid UTF-8 in key for {}: keys must be valid UTF-8 strings",
                stringify!(#key_type_name)
            ));
            Self(key_string)
        })
    };

    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(From<libp2p::kad::Record>),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(key_type_name.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse_quote!(fn from(record: libp2p::kad::Record) -> Self),
            block: block_impl,
        })],
    }
}

// Removed conflicting TryFrom implementation - users can use From implementation and .into() or explicit TryFrom via blanket impl

/// Generate From implementation for the schema type from libp2p::kad::Record
fn generate_schema_from_record_impl(ident: &Ident, use_serde: bool) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(From<libp2p::kad::Record>),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse_quote!(fn from(record: libp2p::kad::Record) -> Self),
            block: if use_serde {
                syn::parse_quote!({
                    let data = record.value;
                    use bincode::serde::Compat;
                    bincode::decode_from_slice::<Compat<Self>, _>(&data, bincode::config::standard())
                        .map(|(compat, _)| compat.0)
                        .unwrap_or_else(|e| {
                            panic!(
                                "Failed to deserialize {} from libp2p record using serde compatibility: {}. \
                                Ensure the record was created by the same schema version.",
                                stringify!(#ident),
                                e
                            )
                        })
                })
            } else {
                syn::parse_quote!({
                    let data = record.value;
                    // Use native bincode decoding
                    bincode::decode_from_slice(&data, bincode::config::standard())
                        .map(|(value, _)| value)
                        .unwrap_or_else(|e| {
                            panic!(
                                "Failed to deserialize {} from libp2p record using native bincode: {}. \
                                Ensure the record was created by a compatible schema version.",
                                stringify!(#ident),
                                e
                            )
                        })
                })
            },
        })],
    }
}

/// Generate utility methods for the schema type
fn generate_schema_utility_impl(ident: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: None,
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![
            syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                }),
                defaultness: None,
                sig: syn::parse_quote!(fn to_kad_record(&self) -> Result<libp2p::kad::Record, bincode::error::EncodeError>),
                block: syn::parse_quote!({
                    let key_bytes = self.key().as_str().as_bytes().to_vec();
                    let key = libp2p::kad::RecordKey::new(&key_bytes);

                    // Use native bincode encoding by default
                    let value = bincode::encode_to_vec(self, bincode::config::standard())?;

                    Ok(libp2p::kad::Record {
                        key,
                        value,
                        publisher: None,
                        expires: None,
                    })
                }),
            }),
            syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                }),
                defaultness: None,
                sig: syn::parse_quote!(fn kad_key(&self) -> libp2p::kad::RecordKey),
                block: syn::parse_quote!({
                    let key_bytes = self.key().as_str().as_bytes().to_vec();
                    libp2p::kad::RecordKey::new(&key_bytes)
                }),
            }),
        ],
    }
}

/// Generate From implementation for converting schema type to libp2p::kad::Record
fn generate_record_from_schema_impl(ident: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(From<#ident>),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::parse_quote!(libp2p::kad::Record),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse_quote!(fn from(schema_item: #ident) -> Self),
            block: syn::parse_quote!({
                // Extract key from the schema item
                let key_bytes = schema_item.key().as_str().as_bytes().to_vec();
                let key = libp2p::kad::RecordKey::new(&key_bytes);

                // Serialize the schema item using native bincode
                let value = bincode::encode_to_vec(&schema_item, bincode::config::standard())
                    .expect("Failed to serialize schema item with native bincode");

                libp2p::kad::Record {
                    key,
                    value,
                    publisher: None,
                    expires: None,
                }
            }),
        })],
    }
}

/// Generate From implementation for converting key type to libp2p::kad::RecordKey
fn generate_record_key_from_key_impl(key_type_name: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(From<#key_type_name>),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::parse_quote!(libp2p::kad::RecordKey),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse_quote!(fn from(key: #key_type_name) -> Self),
            block: syn::parse_quote!({
                let key_bytes = key.as_str().as_bytes().to_vec();
                libp2p::kad::RecordKey::new(&key_bytes)
            }),
        })],
    }
}

/// Count the number of fields marked with #[key] attributes
fn count_key_fields(fields: &syn::Fields) -> usize {
    match fields {
        syn::Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter(|field| has_key_attribute(field))
            .count(),
        syn::Fields::Unnamed(fields_unnamed) => fields_unnamed
            .unnamed
            .iter()
            .filter(|field| has_key_attribute(field))
            .count(),
        syn::Fields::Unit => 0,
    }
}

/// Generate Send + Sync implementation for key type
fn generate_key_send_sync_impl(key_type_name: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: Some(syn::token::Unsafe {
            span: Span::call_site().into(),
        }),
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(Send),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(key_type_name.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![],
    }
}

/// Generate Sync implementation for key type
fn generate_key_sync_impl(key_type_name: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: Some(syn::token::Unsafe {
            span: Span::call_site().into(),
        }),
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(Sync),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(key_type_name.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![],
    }
}

/// Generate Unpin implementation for key type
fn generate_key_unpin_impl(key_type_name: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(std::marker::Unpin),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(key_type_name.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![],
    }
}

// Removed sealed trait implementation - sealed module doesn't exist

/// Generate additional trait implementations for key type (without serde)
fn generate_key_additional_traits(key_type_name: &Ident) -> Vec<syn::ItemImpl> {
    vec![
        // Display implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(std::fmt::Display),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result),
                block: syn::parse_quote!({ write!(f, "{}", self.0) }),
            })],
        },
        // AsRef<str> implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(AsRef<str>),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn as_ref(&self) -> &str),
                block: syn::parse_quote!({ &self.0 }),
            })],
        },
        // From<String> implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(From<String>),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn from(value: String) -> Self),
                block: syn::parse_quote!({ Self(value) }),
            })],
        },
    ]
}

/// Generate additional trait implementations for key type (with serde)
fn generate_key_additional_traits_with_serde(key_type_name: &Ident) -> Vec<syn::ItemImpl> {
    vec![
        // Display implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(std::fmt::Display),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result),
                block: syn::parse_quote!({ write!(f, "{}", self.0) }),
            })],
        },
        // AsRef<str> implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(AsRef<str>),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn as_ref(&self) -> &str),
                block: syn::parse_quote!({ &self.0 }),
            })],
        },
        // From<String> implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(From<String>),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn from(value: String) -> Self),
                block: syn::parse_quote!({ Self(value) }),
            })],
        },
    ]
}

// Removed sealed trait implementation - sealed module doesn't exist

/// Generate additional trait implementations for schema type (without serde)
fn generate_schema_additional_traits(_ident: &Ident) -> Vec<syn::ItemImpl> {
    // For native bincode, we don't need additional trait implementations
    // The derives handle everything we need
    vec![]
}

/// Generate additional trait implementations for schema type (with serde)
fn generate_schema_additional_traits_with_serde(_ident: &Ident) -> Vec<syn::ItemImpl> {
    // When using serde compatibility, we might need additional implementations
    vec![]
}

/// Generate Send implementation for schema type
fn generate_schema_send_sync_impl(ident: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: Some(syn::token::Unsafe {
            span: Span::call_site().into(),
        }),
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(Send),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![],
    }
}

/// Generate Unpin implementation for schema type
fn generate_schema_unpin_impl(ident: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(std::marker::Unpin),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![],
    }
}

/// Generate Sync implementation for schema type
fn generate_schema_sync_impl(ident: &Ident) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: Some(syn::token::Unsafe {
            span: Span::call_site().into(),
        }),
        impl_token: syn::token::Impl {
            span: Span::call_site().into(),
        },
        generics: syn::Generics::default(),
        trait_: Some((
            None,
            syn::parse_quote!(Sync),
            syn::token::For {
                span: Span::call_site().into(),
            },
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        })),
        brace_token: syn::token::Brace {
            span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
        },
        items: vec![],
    }
}

// Removed add_sealed_bounds function - sealed traits not used

/// Generate sealed marker trait implementations for key type
fn generate_key_sealed_marker_traits(key_type_name: &Ident) -> Vec<syn::ItemImpl> {
    vec![
        // Into<Vec<u8>> implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(Into<Vec<u8>>),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn into(self) -> Vec<u8>),
                block: syn::parse_quote!({ self.0.into_bytes() }),
            })],
        },
        // From<Vec<u8>> implementation
        syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site().into(),
            },
            generics: syn::Generics::default(),
            trait_: Some((
                None,
                syn::parse_quote!(::std::convert::From<Vec<u8>>),
                syn::token::For {
                    span: Span::call_site().into(),
                },
            )),
            self_ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(key_type_name.clone()),
            })),
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: syn::parse_quote!(fn from(bytes: Vec<u8>) -> Self),
                block: syn::parse_quote!({
                    Self(String::from_utf8(bytes).unwrap_or_else(|_| String::new()))
                }),
            })],
        },
    ]
}

/// Generate sealed marker trait implementations for schema type
fn generate_schema_sealed_marker_traits(_ident: &Ident) -> Vec<syn::ItemImpl> {
    // Don't generate conflicting trait implementations
    // The derives handle PartialEq, Clone, Debug, etc.
    vec![]
}

/// Check if a field has key-related attributes
fn has_key_attribute(field: &syn::Field) -> bool {
    field.attrs.iter().any(|attr| match &attr.meta {
        syn::Meta::Path(path) => path.is_ident("key") || path.is_ident("NetabaseKey"),
        syn::Meta::List(meta_list) => {
            meta_list.path.is_ident("key") || meta_list.path.is_ident("NetabaseKey")
        }
        syn::Meta::NameValue(name_value) => {
            name_value.path.is_ident("key") || name_value.path.is_ident("NetabaseKey")
        }
    })
}
