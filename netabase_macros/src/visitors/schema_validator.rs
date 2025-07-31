use syn::visit::Visit;

use crate::visitors::{
    key_finder::{get_schema_field_keys, get_schema_outer_key},
    schema_finder::SchemaType,
    utils::{KeyInfo, KeyType, SchemaInfo},
};

#[derive(Default)]
pub(crate) struct SchemaValidator<'ast> {
    pub info: SchemaInfo<'ast>,
    pub valid_schema: bool,
<<<<<<< HEAD
=======
}

pub(crate) fn contains_netabase_derive<'a>(schema_type: &SchemaType<'a>) -> bool {
    schema_type
        .attributes()
        .iter()
        .any(|att| att.path().is_ident("NetabaseSchema"))
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
}

impl<'ast> SchemaValidator<'ast> {
    pub fn get_schema_key<'b>(
        &'b self,
        schema: Option<&'b SchemaType<'ast>>,
    ) -> Option<KeyType<'ast>> {
        //TODO: use result instead cause the none case is technicaly an erruh innit
<<<<<<< HEAD
        if let Some(schema) = schema {
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
                _ => {
                    panic!("Field keys can only be paths (Not closures)");
                    None
                }
                (KeyType::SchemaKey(expr_closure), None) => todo!(),
                (KeyType::SchemaKey(expr_closure), Some(_)) => todo!(),
            }
        } else {
            None
        }
=======
        schema.map(get_schema_field_keys)
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
    }
}

impl<'ast> Visit<'ast> for SchemaValidator<'ast> {
    fn visit_item(&mut self, i: &'ast syn::Item) {
        // TODO: Actually, put the generators in here so its done in one pass
        self.info.schema_type = SchemaType::try_from(i).ok();
        self.info.schema_key = Some(KeyInfo {
            generation_type: self.get_schema_key(self.info.schema_type.as_ref()),
        });

        match (self.info.schema_type, self.info.schema_key.clone()) {
            (None, _) => panic!("Schema needs to be an Enum or Struct"),
            (_, None) => panic!("Schema needs a key"),
            (Some(_), Some(_)) => self.valid_schema = true,
        }
    }
}
