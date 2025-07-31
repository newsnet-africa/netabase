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
}

pub(crate) fn contains_netabase_derive<'a>(schema_type: &SchemaType<'a>) -> bool {
    schema_type
        .attributes()
        .iter()
        .any(|att| att.path().is_ident("NetabaseSchema"))
}

impl<'ast> SchemaValidator<'ast> {
    pub fn get_schema_key<'b>(
        &'b self,
        schema: Option<&'b SchemaType<'ast>>,
    ) -> Option<KeyType<'ast>> {
        //TODO: use result instead cause the none case is technicaly an erruh innit
        schema.map(get_schema_field_keys)
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
