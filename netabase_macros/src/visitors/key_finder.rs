use crate::visitors::{
    key_validator::check_outer_key,
    schema_finder::SchemaType,
    utils::{KeyInfo, KeyType},
};
use syn::{Expr, Field, Fields, Item, Meta, visit::Visit};

#[derive(Clone, Default)]
pub(crate) struct KeyFinder<'ast> {
    pub key: KeyInfo<'ast>,
}

pub(crate) fn get_schema_field_keys<'ast: 'b, 'b>(schema: &'b SchemaType<'ast>) -> KeyType<'ast> {
    KeyType::FieldKeys(
        schema
            .fields()
            .iter()
            .filter_map(|(var, fie)| match (var, check_fields_for_key(fie)) {
                (None, None) => None,
                (_, Some(_)) => Some((*var, *fie)),
                (Some(_), None) => {
                    panic!("Every Variant needs a key");
                    None
                }
            })
            .collect(),
    )
}

fn get_schema_keys<'ast: 'b, 'b>(schema: &'b SchemaType<'ast>) -> Option<KeyType<'ast>> {
    match (
        get_schema_field_keys::<'ast, 'b>(schema),
        get_schema_outer_key::<'ast, 'b>(schema),
    ) {
        (KeyType::FieldKeys(hash_map), Some(KeyType::SchemaKey(outer))) => {
            if hash_map.is_empty() {
                Some(KeyType::SchemaKey(outer))
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
        (KeyType::SchemaKey(expr_closure), Some(_)) => {
            if check_outer_key(expr_closure, schema) {
                Some(KeyType::SchemaKey(expr_closure))
            } else {
                panic!("Key closure must be valid");
                None
            }
        }
        (KeyType::SchemaKey(expr_closure), None) => panic!("Fielded keys cannot be closures"),
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

impl<'ast> Visit<'ast> for KeyFinder<'ast> {
    fn visit_item(&mut self, i: &'ast syn::Item) {
        if let Ok(schema) = SchemaType::try_from(i)
            && let Some(kt) = get_schema_keys(&schema)
        {
            self.key = KeyInfo {
                generation_type: Some(kt),
            };
        }
    }
}
