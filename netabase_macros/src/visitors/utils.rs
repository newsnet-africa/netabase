use std::collections::HashMap;

use syn::{ExprClosure, Field, Fields, PathSegment, Token, Variant, punctuated::Punctuated};

use crate::visitors::utils::schema_finder::SchemaType;

#[derive(Clone, Default)]
pub(crate) struct SchemaInfo<'ast> {
    pub schema_type: Option<SchemaType<'ast>>,
    pub path: Punctuated<PathSegment, Token![::]>,
    pub schema_key: Option<KeyType<'ast>>,
}

#[derive(Clone)]
pub(crate) enum KeyType<'ast> {
    FieldKeys(HashMap<Option<&'ast Variant>, &'ast Fields>),
    SchemaKey(&'ast ExprClosure),
}

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
                        }
                    },
                )
                .collect(),
        )
    }

    pub fn get_schema_key_item<'ast: 'b, 'b>(item: &'b SchemaType<'ast>) -> Option<KeyType<'ast>> {
        //TODO: use result instead cause the none case is technicaly an erruh innit

        match (get_schema_field_keys(item), get_schema_outer_key(item)) {
            (KeyType::FieldKeys(hash_map), Some(outer)) => {
                if hash_map.is_empty() {
                    Some(outer)
                } else {
                    panic!("Schema key closures and field keys are mutually exclusive");
                    None
                }
            }
            (KeyType::FieldKeys(hash_map), None) => {
                if hash_map.is_empty() {
                    panic!("At least one key is needed");
                    None
                } else {
                    Some(KeyType::FieldKeys(hash_map))
                }
            }
            _ => {
                panic!("Field keys can only be paths (Not closures)");
                None
            }
        }
    }
    pub(crate) fn get_schema_outer_key<'ast: 'b, 'b>(
        item: &'b SchemaType<'ast>,
    ) -> Option<KeyType<'ast>> {
        item.attributes().iter().find_map(|att| {
            if let Meta::NameValue(nv) = &att.meta
                && let Expr::Closure(c) = &nv.value
                && nv.path.is_ident("key")
            {
                Some(KeyType::SchemaKey(c))
            } else {
                None
            }
        })
    }
    pub(crate) fn check_fields_for_key(fields: &Fields) -> Option<&Field> {
        fn check_field(field: &Field) -> bool {
            field
                .attrs
                .iter()
                .any(|att| att.meta.path().is_ident("key"))
        }
        let mut key_count: usize = 0;
        match fields {
            Fields::Named(fields_named) => {
                fields_named
                    .named
                    .iter()
                    .find(|f| match (check_field(f), key_count) {
                        (true, 0) => {
                            key_count += 1;
                            true
                        }
                        (true, _) => {
                            panic!("Only one key field is valid");
                            false
                        }
                        (false, _) => false,
                    })
            }
            Fields::Unnamed(fields_unnamed) => {
                fields_unnamed
                    .unnamed
                    .iter()
                    .find(|f| match (check_field(f), key_count) {
                        (true, 0) => {
                            key_count += 1;
                            true
                        }
                        (true, _) => {
                            panic!("Only one key field is valid");
                            false
                        }
                        (false, _) => false,
                    })
            }
            Fields::Unit => {
                panic!("Only Fielded items can be Schemas");
                None
            }
        }
    }
}

pub mod key_validator {
    use syn::ExprClosure;

    use crate::visitors::utils::{
        KeyType, key_finder::get_schema_outer_key, schema_finder::SchemaType,
    };

    fn check_outer_key<'a>(schema: &SchemaType<'a>) -> bool {
        fn check_expr_input_type<'a>(expr: &ExprClosure, schema: &SchemaType<'a>) -> bool {
            expr.inputs.iter().any(|inp| {
                if let syn::Pat::Type(pat_type) = inp
                    && let syn::Type::Path(type_path) = pat_type.ty.as_ref()
                {
                    type_path.path.is_ident(schema.identity())
                } else {
                    false
                }
            })
        }

        if let Some(KeyType::SchemaKey(e)) = get_schema_outer_key(&schema)
            && check_expr_input_type(e, &schema)
        {
            true
        } else {
            panic!("Closure must accept ");
            false
        }
    }
}
