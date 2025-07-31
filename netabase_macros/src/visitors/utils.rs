use quote::quote;
use std::collections::HashMap;

use quote::ToTokens;
use syn::{ExprClosure, Field, Fields, PathSegment, Token, Variant, punctuated::Punctuated};

use crate::visitors::schema_finder::SchemaType;

#[derive(Clone, Default)]
pub(crate) struct SchemaInfo<'ast> {
    pub schema_type: Option<SchemaType<'ast>>,
    pub path: Punctuated<PathSegment, Token![::]>,
<<<<<<< HEAD
    pub schema_key: Option<KeyType<'ast>>,
=======
    pub schema_key: Option<KeyInfo<'ast>>,
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
}

#[derive(Clone)]
pub(crate) enum KeyType<'schema> {
    FieldKeys(HashMap<Option<&'schema Variant>, &'schema Fields>),
    SchemaKey(&'schema ExprClosure),
}

<<<<<<< HEAD
pub(crate) mod schema_finder {
    use std::collections::HashMap;

    use syn::{
        Attribute, Field, Fields, Generics, Ident, Item, ItemEnum, ItemStruct, Variant, Visibility,
        punctuated::Punctuated, spanned::Spanned, token::Comma,
    };

    #[derive(Clone, Copy)]
    pub(crate) enum SchemaType<'ast> {
        Struct(&'ast ItemStruct),
        Enum(&'ast ItemEnum),
    }

    impl<'ast> SchemaType<'ast> {
        pub(crate) fn attributes<'b>(&'b self) -> &'ast Vec<Attribute> {
            match self {
                SchemaType::Struct(item_struct) => &item_struct.attrs,
                SchemaType::Enum(item_enum) => &item_enum.attrs,
            }
        }

        pub(crate) fn visibility<'b>(&'b self) -> &'ast Visibility {
            match self {
                SchemaType::Struct(item_struct) => &item_struct.vis,
                SchemaType::Enum(item_enum) => &item_enum.vis,
            }
        }
        pub(crate) fn identity<'b>(&'b self) -> &'ast Ident {
            match self {
                SchemaType::Struct(item_struct) => &item_struct.ident,
                SchemaType::Enum(item_enum) => &item_enum.ident,
            }
        }

        pub(crate) fn generics<'b>(&'b self) -> &'ast Generics {
            match self {
                SchemaType::Struct(item_struct) => &item_struct.generics,
                SchemaType::Enum(item_enum) => &item_enum.generics,
            }
        }

        pub(crate) fn variants<'b>(&'b self) -> Option<&'ast Punctuated<Variant, Comma>> {
            match self {
                SchemaType::Struct(_) => None,
                SchemaType::Enum(item_enum) => Some(&item_enum.variants),
            }
        }

        pub(crate) fn fields<'b>(&'b self) -> HashMap<Option<&'ast Variant>, &'ast Fields> {
            match self {
                SchemaType::Struct(item_struct) => {
                    let mut res: HashMap<Option<&'ast Variant>, &'ast syn::Fields> = HashMap::new();
                    res.insert(None, &item_struct.fields);
                    res
                }
                SchemaType::Enum(item_enum) => {
                    let mut res: HashMap<Option<&Variant>, &syn::Fields> = HashMap::new();
                    item_enum.variants.iter().for_each(|v| {
                        res.insert(Some(v), &v.fields);
                    });
                    res
                }
            }
        }
    }

    impl<'a> TryFrom<&'a Item> for SchemaType<'a> {
        type Error = syn::Error;

        fn try_from(value: &'a Item) -> Result<Self, Self::Error> {
            match value {
                Item::Enum(item_enum) => Ok(SchemaType::Enum(item_enum)),
                Item::Struct(item_struct) => Ok(SchemaType::Struct(item_struct)),
                _ => Err(syn::Error::new(
                    value.span(),
                    "Schema can only be an Enum or a Struct",
                )),
            }
        }
    }
}

pub(crate) mod schema_validator {
    use crate::visitors::utils::SchemaType;

    pub(crate) fn contains_netabase_derive<'a>(schema_type: &SchemaType<'a>) -> bool {
        schema_type
            .attributes()
            .iter()
            .any(|att| att.path().is_ident("Clone"))
    }
}
pub(crate) mod key_finder {

    use crate::visitors::utils::KeyType;
    use syn::{Expr, Field, Fields, Item, Meta};

    use crate::visitors::utils::schema_finder::SchemaType;

    pub(crate) fn get_schema_field_keys<'ast: 'b, 'b>(
        schema: &'b SchemaType<'ast>,
    ) -> KeyType<'ast> {
        KeyType::FieldKeys(
            schema
                .fields()
                .iter()
                .filter_map(
                    |(var, fie)| match (var, check_fields_for_key(fie.clone())) {
                        (None, None) => panic!("Fielded structs need a key field"),
                        (_, Some(_)) => Some((*var, *fie)),
                        (Some(_), None) => {
                            panic!("Every Variant needs a key");
                            None
=======
impl ToTokens for KeyType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            KeyType::FieldKeys(hash_map) => {
                let stream = hash_map.iter().filter_map(|(o, f)| {
                    o.as_ref().map(|op| {
                        quote! {
                            #op: #f,
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
                        }
                    })
                });
                tokens.extend_one(quote! {
                    #(#stream)*;
                });
            }
            KeyType::SchemaKey(expr_closure) => expr_closure.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct KeyInfo<'schema> {
    pub generation_type: Option<KeyType<'schema>>,
}
