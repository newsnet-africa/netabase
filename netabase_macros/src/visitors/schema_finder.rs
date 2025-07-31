use std::collections::HashMap;
use syn::{
    Attribute, Fields, Generics, Ident, ItemEnum, ItemStruct, Variant, Visibility,
    spanned::Spanned, token::Comma,
};

use crate::visitors::key_finder::{self, KeyFinder};
use crate::visitors::schema_validator::SchemaValidator;
use crate::visitors::utils::SchemaInfo;
use syn::Item;
use syn::{PathSegment, Token, punctuated::Punctuated, visit::Visit};

#[derive(Default)]
pub struct SchemaFinder<'ast> {
    pub current_path: Punctuated<PathSegment, Token![::]>,
    pub schemas: Vec<SchemaInfo<'ast>>,
}

impl<'ast> Visit<'ast> for SchemaFinder<'ast> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        self.current_path.push(PathSegment {
            ident: i.ident.clone(),
            arguments: syn::PathArguments::None,
        });
        if let Some((_, items)) = &i.content {
            items.iter().for_each(|item| {
                if let Ok(sch) = SchemaType::try_from(item) {
                    let mut schema_validator = SchemaValidator::default();
                    let mut key_finder = KeyFinder::default();
                    schema_validator.visit_item(item);

                    let mut local_path = self.current_path.clone();
                    local_path.push(PathSegment {
                        ident: sch.identity().clone(),
                        arguments: syn::PathArguments::None,
                    });
                    schema_validator.info.path = local_path;

                    self.schemas.push(schema_validator.info.clone());
                } else if let Item::Mod(module) = item {
                    self.visit_item_mod(module);
                }
            });
        } else {
            panic!("Schema module should contain items");
        }
        self.current_path.pop();
    }
}

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
