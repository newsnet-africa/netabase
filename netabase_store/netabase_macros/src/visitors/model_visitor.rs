use syn::{Ident, visit::Visit};

use crate::{
    item_info::netabase_model::{ModelKeyInfo, ModelLinkInfo},
    util::extract_fields,
};

#[derive(Default)]
pub struct ModelVisitor<'ast> {
    pub name: Option<&'ast Ident>,
    pub key: Option<ModelKeyInfo<'ast>>,
    pub links: Vec<ModelLinkInfo<'ast>>,
}

impl<'a> Visit<'a> for ModelVisitor<'a> {
    fn visit_derive_input(&mut self, i: &'a syn::DeriveInput) {
        self.name = Some(&i.ident);
        self.key = match ModelKeyInfo::find_keys(&extract_fields(&i)) {
            Ok(k) => Some(k),
            Err(e) => panic!("Error parsing Model: {e}"),
        };
        self.links = ModelLinkInfo::find_link(&extract_fields(&i)).collect();
    }
}
