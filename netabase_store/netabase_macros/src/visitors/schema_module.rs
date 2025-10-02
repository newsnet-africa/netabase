use std::collections::HashMap;

use syn::{
    Ident, PathSegment, Token,
    punctuated::{Pair, Punctuated},
    visit::Visit,
};

use crate::util::{NetabaseItem, NetabaseItemStruct};

pub struct SchemaModuleVisitor {
    pub k_v_pairs:
        HashMap<Punctuated<PathSegment, Token![::]>, Punctuated<PathSegment, Token![::]>>,
    pub current_path: Vec<Pair<PathSegment, Token![::]>>,
    pub schema_name: Ident,
    pub schema_key_name: Ident,
    pub first: bool,
}

impl SchemaModuleVisitor {
    pub fn new(schema: Ident, key: Ident) -> Self {
        Self {
            k_v_pairs: HashMap::default(),
            current_path: Vec::default(),
            schema_name: schema,
            schema_key_name: key,
            first: true,
        }
    }
}

impl<'ast> Visit<'ast> for SchemaModuleVisitor {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if !self.first {
            self.current_path.push(Pair::Punctuated(
                PathSegment {
                    ident: i.ident.clone(),
                    arguments: syn::PathArguments::None,
                },
                Token![::](proc_macro2::Span::call_site()),
            ));
        }
        self.first = false;
        if let Some((_, content)) = &i.content {
            content.iter().for_each(|item| match item {
                syn::Item::Struct(item_struct) => {
                    self.visit_item_struct(item_struct);
                }
                syn::Item::Mod(item_mod) => {
                    self.visit_item_mod(item_mod);
                }
                _ => {}
            });
        }
    }
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if let Some(ident) = i.is_schema() {
            let k_final_path = {
                let mut final_path = self.current_path.clone();
                final_path.push(Pair::new(
                    PathSegment {
                        ident,
                        arguments: syn::PathArguments::None,
                    },
                    None,
                ));
                Punctuated::from_iter(final_path)
            };

            let s_final_path = {
                let mut final_path = self.current_path.clone();
                final_path.push(Pair::new(
                    PathSegment {
                        ident: i.ident.clone(),
                        arguments: syn::PathArguments::None,
                    },
                    None,
                ));
                Punctuated::from_iter(final_path)
            };

            self.k_v_pairs.insert(s_final_path, k_final_path);
        }
    }
}
