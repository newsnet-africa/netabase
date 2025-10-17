use syn::{PathSegment, Token, punctuated::Punctuated, visit::Visit};

use crate::{item_info::netabase_definitions::ModuleInfo, util::append_ident};

#[derive(Default)]
pub struct DefinitionsVisitor<'ast> {
    pub modules: Vec<ModuleInfo<'ast>>,
    pub current_path: Punctuated<PathSegment, Token![::]>,
    inner: bool,
}

impl<'a> Visit<'a> for DefinitionsVisitor<'a> {
    fn visit_item_mod(&mut self, i: &'a syn::ItemMod) {
        let mut module_info = ModuleInfo::default();
        if self.inner {
            self.current_path.push(i.ident.clone().into());
        }
        module_info.path = self.current_path.clone();
        if let Some((_, v_content)) = &i.content {
            v_content.iter().for_each(|ci| {
                if let syn::Item::Struct(item_struct) = ci
                    && Self::check_derive(item_struct, "NetabaseModel")
                {
                    let key_name = append_ident(&item_struct.ident, "Key");
                    module_info.models.push(item_struct);
                    module_info.keys.push(key_name);
                } else if let syn::Item::Mod(item_mod) = ci {
                    self.inner = true;
                    self.visit_item_mod(item_mod);
                }
            });
        }
        self.modules.push(module_info);
    }
}
