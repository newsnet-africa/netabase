pub mod schema {
    use proc_macro2::{Group, Span};
    use quote::ToTokens;
    use syn::{
        Block, FnArg, GenericArgument, Generics, Ident, ImplItem, ItemImpl, Path, PathSegment,
        Stmt, Token, parse_quote,
        punctuated::{Pair, Punctuated},
        token::{Brace, Comma, Lt},
    };

    use crate::SchemaType;

    fn try_from_impl(from: &Ident, to: &Ident, block: Block) -> ItemImpl {
        let to_type = syn::Type::Path(syn::TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: {
                    let mut punct = Punctuated::new();
                    punct.push(PathSegment {
                        ident: to.clone(),
                        arguments: syn::PathArguments::None,
                    });
                    punct
                },
            },
        });
        let from_type = syn::Type::Path(syn::TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: {
                    let mut punct = Punctuated::new();
                    punct.push(PathSegment {
                        ident: from.clone(),
                        arguments: syn::PathArguments::None,
                    });
                    punct
                },
            },
        });
        let fn_arg = FnArg::Typed(syn::PatType {
            attrs: vec![],
            pat: Box::new(syn::Pat::Type(syn::PatType {
                attrs: vec![],
                pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: Ident::new("value", Span::call_site()),
                    subpat: None,
                })),
                colon_token: Token![:](Span::call_site()),
                ty: Box::new(from_type.clone()),
            })),
            colon_token: Token![:](Span::call_site()),
            ty: Box::new(from_type.clone()),
        });
        ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: syn::token::Impl {
                span: Span::call_site(),
            },
            generics: syn::Generics {
                lt_token: None,
                params: Punctuated::new(),
                gt_token: None,
                where_clause: None,
            },
            trait_: Some((
                None,
                Path {
                    leading_colon: None,
                    segments: {
                        let mut punct = Punctuated::new();
                        punct.push(PathSegment {
                            ident: proc_macro2::Ident::new("TryFrom", Span::call_site()),
                            arguments: syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token![<](Span::call_site()),
                                    args: {
                                        let mut punct = Punctuated::new();
                                        punct.push(GenericArgument::Type(from_type.clone()));
                                        punct
                                    },
                                    gt_token: Token![>](Span::call_site()),
                                },
                            ),
                        });
                        punct
                    },
                },
                Token![for](Span::call_site()),
            )),
            self_ty: Box::new(to_type.clone()),
            brace_token: Brace::default(),
            items: vec![
                ImplItem::Type(syn::ImplItemType {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    defaultness: None,
                    type_token: Token![type](Span::call_site()),
                    ident: Ident::new("Error", Span::call_site()),
                    generics: Generics {
                        lt_token: None,
                        params: Punctuated::new(),
                        gt_token: None,
                        where_clause: None,
                    },
                    eq_token: Token![=](Span::call_site()),
                    ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: {
                                Punctuated::from_iter(
                                    vec![
                                        Pair::Punctuated(
                                            PathSegment {
                                                ident: Ident::new("anyhow", Span::call_site()),
                                                arguments: syn::PathArguments::None,
                                            },
                                            Token![::](Span::call_site()),
                                        ),
                                        Pair::End(PathSegment {
                                            ident: Ident::new("Error", Span::call_site()),
                                            arguments: syn::PathArguments::None,
                                        }),
                                    ]
                                    .into_iter(),
                                )
                            },
                        },
                    }),
                    semi_token: Token![;](Span::call_site()),
                }),
                ImplItem::Fn(syn::ImplItemFn {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    defaultness: None,
                    sig: {
                        let args = {
                            let mut punct = Punctuated::<FnArg, Comma>::new();
                            punct.extend_one(Pair::End(fn_arg));
                            punct
                        };
                        syn::Signature {
                            constness: None,
                            asyncness: None,
                            unsafety: None,
                            abi: None,
                            fn_token: Token![fn](Span::call_site()),
                            ident: Ident::new("try_from", Span::call_site()),
                            generics: Generics::default(),
                            paren_token: syn::token::Paren {
                                span: Group::new(
                                    proc_macro2::Delimiter::Parenthesis,
                                    from_type.to_token_stream(),
                                )
                                .delim_span(),
                            },
                            inputs: args,
                            variadic: None,
                            output: syn::ReturnType::Type(
                                Token![->](Span::call_site()),
                                Box::new(syn::Type::Path(syn::TypePath {
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: {
                                            let res = vec![
                                                PathSegment {
                                                    ident: Ident::new("anyhow", Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                },
                                                PathSegment {
                                                    ident: Ident::new("Result", Span::call_site()),
                                                    arguments: syn::PathArguments::AngleBracketed(
                                                        syn::AngleBracketedGenericArguments {
                                                            colon2_token: None,
                                                            lt_token: Token![<](Span::call_site()),
                                                            args: {
                                                                let mut punct = Punctuated::new();
                                                                punct.extend_one(Pair::End(
                                                                    GenericArgument::Type(to_type),
                                                                ));
                                                                punct
                                                            },
                                                            gt_token: Token![>](Span::call_site()),
                                                        },
                                                    ),
                                                },
                                            ]
                                            .into_iter();
                                            Punctuated::from_iter(res)
                                        },
                                    },
                                })),
                            ),
                        }
                    },
                    block,
                }),
            ],
        }
    }

    pub fn from_schema_for_record(schema: &SchemaType) -> ItemImpl {
        let block = parse_quote!({
            println!();
        });
        try_from_impl(
            schema.identity(),
            &Ident::new("Record", Span::call_site()),
            block,
        )
    }
}
