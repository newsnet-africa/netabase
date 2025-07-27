//! Key generation functionality for netabase schemas
//!
//! This module handles the generation of key functions for netabase schemas,
//! supporting both field-level and item-level key generation strategies.

use proc_macro::Span;
use proc_macro2::{Group, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{
    ExprClosure, Field, FnArg, Ident, ItemFn, Pat, ReturnType, Type, TypeParamBound,
    punctuated::Punctuated, spanned::Spanned, token::Paren,
};

use crate::visitors::errors::{NetabaseError, NetabaseResult, ToCompileError};

/// Represents different strategies for generating keys from schema data
#[derive(Debug, Clone)]
pub enum KeyGenerator<'a> {
    /// Use a closure that takes a field value and returns a key
    FieldClosure(&'a ExprClosure),
    /// Use a closure that takes the entire item (struct/enum) and returns a key
    ItemClosure(&'a ExprClosure),
    /// Simply use the field value directly as the key
    Field(&'a Field),
    /// No key generation strategy specified
    None,
}

impl<'a> Default for KeyGenerator<'a> {
    fn default() -> Self {
        KeyGenerator::None
    }
}

impl<'a> ToTokens for KeyGenerator<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            KeyGenerator::FieldClosure(expr_closure) => {
                expr_closure.to_tokens(tokens);
            }
            KeyGenerator::ItemClosure(expr_closure) => {
                expr_closure.to_tokens(tokens);
            }
            KeyGenerator::Field(field) => {
                if let Some(ref ident) = field.ident {
                    ident.to_tokens(tokens);
                }
            }
            KeyGenerator::None => {}
        }
    }
}

/// Labels for key generation, containing field and key generator information
#[derive(Debug)]
pub struct KeyGeneratorLabels<'a> {
    pub field: &'a Field,
    pub generator: KeyGenerator<'a>,
    pub item_type: Option<&'a Type>,
}

impl<'a> KeyGeneratorLabels<'a> {
    /// Create new key generator labels
    pub fn new(field: &'a Field, generator: KeyGenerator<'a>) -> Self {
        Self {
            field,
            generator,
            item_type: None,
        }
    }

    /// Create new key generator labels with item type for item-level closures
    pub fn with_item_type(
        field: &'a Field,
        generator: KeyGenerator<'a>,
        item_type: &'a Type,
    ) -> Self {
        Self {
            field,
            generator,
            item_type: Some(item_type),
        }
    }
}

impl<'a> From<KeyGeneratorLabels<'a>> for ItemFn {
    fn from(value: KeyGeneratorLabels<'a>) -> Self {
        match generate_key_function(value) {
            Ok(item_fn) => item_fn,
            Err(error) => {
                // Create a function that produces a compile error
                create_compile_error_function(error)
            }
        }
    }
}

/// Create a function that will cause a compile error with the given error message
fn create_compile_error_function(error: NetabaseError) -> ItemFn {
    let error_tokens = error.to_compile_error();

    ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Public(syn::token::Pub {
            span: Span::call_site().into(),
        }),
        sig: syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: syn::token::Fn {
                span: Span::call_site().into(),
            },
            ident: Ident::new("key", Span::call_site().into()),
            generics: syn::Generics::default(),
            paren_token: Paren {
                span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                    .delim_span(),
            },
            inputs: Punctuated::new(),
            variadic: None,
            output: ReturnType::Default,
        },
        block: Box::new(syn::Block {
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            stmts: vec![syn::Stmt::Item(syn::Item::Verbatim(error_tokens))],
        }),
    }
}

/// Generate a key function based on the key generator labels
pub fn generate_key_function(labels: KeyGeneratorLabels) -> NetabaseResult<ItemFn> {
    let fn_name = Ident::new("key", Span::call_site().into());

    match labels.generator {
        KeyGenerator::FieldClosure(closure) => {
            generate_field_closure_function(fn_name, closure, labels.field)
        }
        KeyGenerator::ItemClosure(closure) => {
            if let Some(item_type) = labels.item_type {
                generate_item_closure_function(fn_name, closure, item_type)
            } else {
                Err(NetabaseError::KeyGeneration(
                    crate::visitors::errors::KeyGenerationError::MissingClosureInput {
                        field_name: labels.field.ident.as_ref().map(|i| i.to_string()),
                        span: closure.span(),
                    },
                ))
            }
        }
        KeyGenerator::Field(field) => {
            generate_field_access_function(fn_name, field, labels.item_type)
        }
        KeyGenerator::None => Err(NetabaseError::Schema(
            crate::visitors::errors::SchemaError::NoKeyFields {
                item_name: Ident::new("Unknown", Span::call_site().into()),
                span: Span::call_site().into(),
            },
        )),
    }
}

/// Generate a key function that uses a field-level closure
fn generate_field_closure_function(
    fn_name: Ident,
    closure: &ExprClosure,
    field: &Field,
) -> NetabaseResult<ItemFn> {
    // Validate closure signature
    validate_field_closure_signature(closure, field)?;

    let field_name = field.ident.as_ref().ok_or_else(|| {
        NetabaseError::Field(crate::visitors::errors::FieldError::MissingFieldName {
            field_index: 0,
            span: field.span(),
        })
    })?;

    let field_type = &field.ty;

    Ok(ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Public(syn::token::Pub {
            span: Span::call_site().into(),
        }),
        sig: syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: syn::token::Fn {
                span: Span::call_site().into(),
            },
            ident: fn_name,
            generics: syn::Generics::default(),
            paren_token: Paren {
                span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                    .delim_span(),
            },
            inputs: {
                let mut inputs = Punctuated::new();
                inputs.push(FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: Ident::new("item", Span::call_site().into()),
                        subpat: None,
                    })),
                    colon_token: syn::token::Colon {
                        spans: [Span::call_site().into()],
                    },
                    ty: Box::new(syn::Type::Reference(syn::TypeReference {
                        and_token: syn::token::And {
                            spans: [Span::call_site().into()],
                        },
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(field_type.clone()),
                    })),
                }));
                inputs
            },
            variadic: None,
            output: ReturnType::Type(
                syn::token::RArrow {
                    spans: [Span::call_site().into(), Span::call_site().into()],
                },
                Box::new(syn::Type::ImplTrait(syn::TypeImplTrait {
                    impl_token: syn::token::Impl {
                        span: Span::call_site().into(),
                    },
                    bounds: {
                        let mut bounds = Punctuated::new();
                        bounds.push(TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse_quote!(From<Vec<u8>>),
                        }));
                        bounds.push(TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse_quote!(Into<Vec<u8>>),
                        }));
                        bounds
                    },
                })),
            ),
        },
        block: Box::new(syn::Block {
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            stmts: vec![syn::Stmt::Expr(
                syn::Expr::Call(syn::ExprCall {
                    attrs: vec![],
                    func: Box::new(syn::Expr::Paren(syn::ExprParen {
                        attrs: vec![],
                        paren_token: syn::token::Paren {
                            span: Group::new(
                                proc_macro2::Delimiter::Parenthesis,
                                TokenStream2::new(),
                            )
                            .delim_span(),
                        },
                        expr: Box::new(syn::Expr::Closure(closure.clone())),
                    })),
                    paren_token: syn::token::Paren {
                        span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                            .delim_span(),
                    },
                    args: {
                        let mut args = Punctuated::new();
                        args.push(syn::Expr::Field(syn::ExprField {
                            attrs: vec![],
                            base: Box::new(syn::Expr::Path(syn::ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: syn::parse_quote!(item),
                            })),
                            dot_token: syn::token::Dot {
                                spans: [Span::call_site().into()],
                            },
                            member: syn::Member::Named(field_name.clone()),
                        }));
                        args
                    },
                }),
                None,
            )],
        }),
    })
}

/// Generate a key function that uses an item-level closure
fn generate_item_closure_function(
    fn_name: Ident,
    closure: &ExprClosure,
    item_type: &Type,
) -> NetabaseResult<ItemFn> {
    // Validate closure signature
    validate_item_closure_signature(closure, item_type)?;

    Ok(ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Public(syn::token::Pub {
            span: Span::call_site().into(),
        }),
        sig: syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: syn::token::Fn {
                span: Span::call_site().into(),
            },
            ident: fn_name,
            generics: syn::Generics::default(),
            paren_token: Paren {
                span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                    .delim_span(),
            },
            inputs: {
                let mut inputs = Punctuated::new();
                inputs.push(FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: Ident::new("item", Span::call_site().into()),
                        subpat: None,
                    })),
                    colon_token: syn::token::Colon {
                        spans: [Span::call_site().into()],
                    },
                    ty: Box::new(syn::Type::Reference(syn::TypeReference {
                        and_token: syn::token::And {
                            spans: [Span::call_site().into()],
                        },
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(item_type.clone()),
                    })),
                }));
                inputs
            },
            variadic: None,
            output: ReturnType::Type(
                syn::token::RArrow {
                    spans: [Span::call_site().into(), Span::call_site().into()],
                },
                Box::new(syn::Type::ImplTrait(syn::TypeImplTrait {
                    impl_token: syn::token::Impl {
                        span: Span::call_site().into(),
                    },
                    bounds: {
                        let mut bounds = Punctuated::new();
                        bounds.push(TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse_quote!(From<Vec<u8>>),
                        }));
                        bounds.push(TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse_quote!(Into<Vec<u8>>),
                        }));
                        bounds
                    },
                })),
            ),
        },
        block: Box::new(syn::Block {
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            stmts: vec![syn::Stmt::Expr(
                syn::Expr::Call(syn::ExprCall {
                    attrs: vec![],
                    func: Box::new(syn::Expr::Paren(syn::ExprParen {
                        attrs: vec![],
                        paren_token: syn::token::Paren {
                            span: Group::new(
                                proc_macro2::Delimiter::Parenthesis,
                                TokenStream2::new(),
                            )
                            .delim_span(),
                        },
                        expr: Box::new(syn::Expr::Closure(closure.clone())),
                    })),
                    paren_token: syn::token::Paren {
                        span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                            .delim_span(),
                    },
                    args: {
                        let mut args = Punctuated::new();
                        args.push(syn::Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::parse_quote!(item),
                        }));
                        args
                    },
                }),
                None,
            )],
        }),
    })
}

/// Generate a key function that directly accesses a field
fn generate_field_access_function(
    fn_name: Ident,
    field: &Field,
    item_type: Option<&Type>,
) -> NetabaseResult<ItemFn> {
    let field_name = field.ident.as_ref().ok_or_else(|| {
        NetabaseError::Field(crate::visitors::errors::FieldError::MissingFieldName {
            field_index: 0,
            span: field.span(),
        })
    })?;

    // Use the provided item type or infer from context
    let input_type = item_type.cloned().unwrap_or_else(|| {
        // Default to a generic Self type if no item type provided
        syn::parse_quote!(Self)
    });

    Ok(ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Public(syn::token::Pub {
            span: Span::call_site().into(),
        }),
        sig: syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: syn::token::Fn {
                span: Span::call_site().into(),
            },
            ident: fn_name,
            generics: syn::Generics::default(),
            paren_token: Paren {
                span: Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream2::new())
                    .delim_span(),
            },
            inputs: {
                let mut inputs = Punctuated::new();
                inputs.push(FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: Ident::new("item", Span::call_site().into()),
                        subpat: None,
                    })),
                    colon_token: syn::token::Colon {
                        spans: [Span::call_site().into()],
                    },
                    ty: Box::new(syn::Type::Reference(syn::TypeReference {
                        and_token: syn::token::And {
                            spans: [Span::call_site().into()],
                        },
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(input_type),
                    })),
                }));
                inputs
            },
            variadic: None,
            output: ReturnType::Type(
                syn::token::RArrow {
                    spans: [Span::call_site().into(), Span::call_site().into()],
                },
                Box::new(field.ty.clone()),
            ),
        },
        block: Box::new(syn::Block {
            brace_token: syn::token::Brace {
                span: Group::new(proc_macro2::Delimiter::Brace, TokenStream2::new()).delim_span(),
            },
            stmts: vec![syn::Stmt::Expr(
                syn::Expr::Field(syn::ExprField {
                    attrs: vec![],
                    base: Box::new(syn::Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::parse_quote!(item),
                    })),
                    dot_token: syn::token::Dot {
                        spans: [Span::call_site().into()],
                    },
                    member: syn::Member::Named(field_name.clone()),
                }),
                None,
            )],
        }),
    })
}

/// Validate that a field-level closure has the correct signature
fn validate_field_closure_signature(closure: &ExprClosure, field: &Field) -> NetabaseResult<()> {
    // Check that closure has exactly one input
    if closure.inputs.len() != 1 {
        return Err(NetabaseError::KeyGeneration(
            crate::visitors::errors::KeyGenerationError::MissingClosureInput {
                field_name: field.ident.as_ref().map(|i| i.to_string()),
                span: closure.span(),
            },
        ));
    }

    // Check that the input type matches the field type
    if let Some(Pat::Type(pat_type)) = closure.inputs.first() {
        let closure_input_type = &pat_type.ty;
        if !types_match(&field.ty, closure_input_type) {
            return Err(NetabaseError::KeyGeneration(
                crate::visitors::errors::KeyGenerationError::ClosureInputTypeMismatch {
                    field_name: field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_default(),
                    field_type: field.ty.to_token_stream().to_string(),
                    closure_input_type: closure_input_type.to_token_stream().to_string(),
                    span: closure.span(),
                },
            ));
        }
    }

    // Validate return type bounds
    if let ReturnType::Type(_, return_type) = &closure.output {
        validate_key_return_type(return_type, field.ident.as_ref().map(|i| i.to_string()))?;
    }

    Ok(())
}

/// Validate that an item-level closure has the correct signature
fn validate_item_closure_signature(closure: &ExprClosure, item_type: &Type) -> NetabaseResult<()> {
    // Check that closure has exactly one input
    if closure.inputs.len() != 1 {
        return Err(NetabaseError::KeyGeneration(
            crate::visitors::errors::KeyGenerationError::MissingClosureInput {
                field_name: None,
                span: closure.span(),
            },
        ));
    }

    // Check that the input type matches the item type
    if let Some(Pat::Type(pat_type)) = closure.inputs.first() {
        let closure_input_type = &pat_type.ty;
        if !types_match(item_type, closure_input_type) {
            return Err(NetabaseError::KeyGeneration(
                crate::visitors::errors::KeyGenerationError::ClosureInputTypeMismatch {
                    field_name: "item".to_string(),
                    field_type: item_type.to_token_stream().to_string(),
                    closure_input_type: closure_input_type.to_token_stream().to_string(),
                    span: closure.span(),
                },
            ));
        }
    }

    // Validate return type bounds
    if let ReturnType::Type(_, return_type) = &closure.output {
        validate_key_return_type(return_type, None)?;
    }

    Ok(())
}

/// Validate that a return type implements the required key traits
fn validate_key_return_type(return_type: &Type, field_name: Option<String>) -> NetabaseResult<()> {
    match return_type {
        Type::ImplTrait(impl_trait) => {
            let has_from_bytes = impl_trait.bounds.iter().any(|bound| {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    trait_bound.path.is_ident("From")
                        || trait_bound
                            .path
                            .segments
                            .last()
                            .map_or(false, |seg| seg.ident == "From")
                } else {
                    false
                }
            });

            let has_into_bytes = impl_trait.bounds.iter().any(|bound| {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    trait_bound.path.is_ident("Into")
                        || trait_bound
                            .path
                            .segments
                            .last()
                            .map_or(false, |seg| seg.ident == "Into")
                } else {
                    false
                }
            });

            if !has_from_bytes || !has_into_bytes {
                return Err(NetabaseError::KeyGeneration(
                    crate::visitors::errors::KeyGenerationError::InvalidClosureReturnType {
                        field_name,
                        expected: "impl From<Vec<u8>> + Into<Vec<u8>>".to_string(),
                        found: return_type.to_token_stream().to_string(),
                        span: return_type.span(),
                    },
                ));
            }
        }
        _ => {
            // For non-impl trait types, we assume they're valid
            // This could be enhanced with more sophisticated type checking
        }
    }

    Ok(())
}

/// Simple type matching - could be enhanced with more sophisticated comparison
fn types_match(type1: &Type, type2: &Type) -> bool {
    // For now, do a simple token stream comparison
    // This could be made more sophisticated to handle equivalent types
    type1.to_token_stream().to_string() == type2.to_token_stream().to_string()
}

/// Helper trait for generating key functions
pub trait KeyFunctionGenerator {
    /// Generate a key function for the given schema
    fn generate_key_function(&self) -> NetabaseResult<ItemFn>;
}

impl<'a> KeyFunctionGenerator for KeyGeneratorLabels<'a> {
    fn generate_key_function(&self) -> NetabaseResult<ItemFn> {
        generate_key_function(KeyGeneratorLabels {
            field: self.field,
            generator: self.generator.clone(),
            item_type: self.item_type,
        })
    }
}
