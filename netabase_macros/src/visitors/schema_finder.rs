use std::collections::HashMap;

use crate::SchemaValidator;
use crate::visitors::utils::schema_finder::SchemaType;
use crate::visitors::utils::{SchemaInfo, schema_validator::contains_netabase_derive};
use proc_macro::Span;
use syn::token::Semi;
use syn::{PathSegment, Token, punctuated::Punctuated, visit::Visit};

#[derive(Default)]
pub struct SchemaFinder<'ast> {
    pub current_path: Punctuated<PathSegment, Token![::]>,
    pub schemas: Vec<SchemaInfo<'ast>>,
    pub schema_validator: SchemaValidator<'ast>,
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
                    self.schema_validator.visit_item(item);
                    if self.schema_validator.valid_schema {
                        self.schema_validator.info.path = {
                            let mut local_path = self.current_path.clone();
                            local_path.push(PathSegment {
                                ident: i.ident.clone(),
                                arguments: syn::PathArguments::None,
                            });
                            local_path
                        };
                        self.schemas.push(self.schema_validator.info.clone());
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
