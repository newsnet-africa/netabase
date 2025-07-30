use std::collections::HashMap;

use crate::visitors::utils::schema_finder::SchemaType;
use crate::visitors::utils::{SchemaInfo, schema_validator::contains_netabase_derive};
use syn::{PathSegment, Token, punctuated::Punctuated, visit::Visit};

pub struct SchemaFinder<'ast> {
    pub current_path: Punctuated<PathSegment, Token![::]>,
    pub schemas: Vec<SchemaInfo<'ast>>,
}

impl<'ast> Default for SchemaFinder<'ast> {
    fn default() -> Self {
        SchemaFinder {
            current_path: Punctuated::new(),
            schemas: vec![],
        }
    }
}

impl<'ast> Visit<'ast> for SchemaFinder<'ast> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        self.current_path.push(PathSegment {
            ident: i.ident.clone(),
            arguments: syn::PathArguments::None,
        });
        if let Some((_, items)) = &i.content {
            items.iter().for_each(|item| {
                if let Ok(inner_item) = SchemaType::try_from(item) {
                    if contains_netabase_derive(&inner_item) {
                        self.schemas.push(SchemaInfo {
                            path: Punctuated::new(),
                            schema_type: Some(inner_item.clone()),
                            schema_key: None,
                        });
                    }
                } else if let syn::Item::Mod(item_mod) = item {
                    self.visit_item_mod(i);
                } else {
                    panic!("Schema must be Struct or Enum")
                }
            });
        } else {
            panic!("Schema module should contain items");
        }
    }
}
