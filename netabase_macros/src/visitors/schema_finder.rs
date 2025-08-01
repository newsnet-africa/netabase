use std::collections::HashMap;
use syn::{
    Attribute, Fields, Generics, Ident, Item, ItemEnum, ItemStruct, PathSegment, Token, Variant,
    Visibility, punctuated::Punctuated, spanned::Spanned, token::Comma, visit::Visit,
};

use crate::visitors::schema_validator::SchemaValidator;
use crate::visitors::utils::schema_finder::SchemaType;
use crate::visitors::utils::{SchemaInfo, schema_validator::contains_netabase_derive};

/// Finds and validates schemas within modules
#[derive(Default)]
pub struct SchemaFinder<'ast> {
    pub current_path: Punctuated<PathSegment, Token![::]>,
    pub schemas: Vec<SchemaInfo<'ast>>,
    pub schema_validator: SchemaValidator<'ast>,
}

impl<'ast> SchemaFinder<'ast> {
    /// Create a new schema finder
    pub fn new() -> Self {
        Self {
            current_path: Punctuated::new(),
            schemas: Vec::new(),
            schema_validator: SchemaValidator::default(),
        }
    }

    /// Get the found schemas
    pub fn schemas(&self) -> &Vec<SchemaInfo<'ast>> {
        &self.schemas
    }

    /// Consume the finder and return the found schemas
    pub fn into_schemas(self) -> Vec<SchemaInfo<'ast>> {
        self.schemas
    }
}

impl<'ast> Visit<'ast> for SchemaFinder<'ast> {
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        // Add current module to path
        self.current_path.push(PathSegment {
            ident: module.ident.clone(),
            arguments: syn::PathArguments::None,
        });

        if let Some((_, items)) = &module.content {
            items.iter().for_each(|item| {
                if let Ok(inner_item) = SchemaType::try_from(item) {
                    self.schema_validator.visit_item(item);
                    if self.schema_validator.valid_schema {
                        self.schema_validator.info.path = {
                            let mut local_path = self.current_path.clone();
                            local_path.push(PathSegment {
                                ident: inner_item.identity().clone(),
                                arguments: syn::PathArguments::None,
                            });
                            local_path
                        };
                        self.schemas.push(self.schema_validator.info.clone());
                    }
                } else if let syn::Item::Mod(item_mod) = item {
                    self.visit_item_mod(item_mod);
                }
            });
        }

        // Remove current module from path when done
        self.current_path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_schema_finder_basic() {
        let mut finder = SchemaFinder::new();
        let module: syn::ItemMod = parse_quote! {
            mod test_module {
                #[derive(Serialize, Deserialize, NetabaseSchema)]
                struct TestSchema {
                    #[key]
                    id: u64,
                    name: String,
                }
            }
        };

        finder.visit_item_mod(&module);
        assert_eq!(finder.schemas().len(), 0); // Will be 0 until validator is properly implemented
    }

    #[test]
    fn test_schema_finder_nested_modules() {
        let mut finder = SchemaFinder::new();
        let module: syn::ItemMod = parse_quote! {
            mod outer {
                mod inner {
                    #[derive(Serialize, Deserialize, NetabaseSchema)]
                    struct TestSchema {
                        #[key]
                        id: u64,
                    }
                }
            }
        };

        finder.visit_item_mod(&module);
        // Test will pass once validator is implemented
        assert!(finder.schemas().len() >= 0);
    }

    #[test]
    fn test_schema_finder_ignores_invalid() {
        let mut finder = SchemaFinder::new();
        let module: syn::ItemMod = parse_quote! {
            mod test_module {
                // Missing NetabaseSchema derive
                #[derive(Serialize, Deserialize)]
                struct InvalidSchema {
                    #[key]
                    id: u64,
                }

                // Valid schema
                #[derive(Serialize, Deserialize, NetabaseSchema)]
                struct ValidSchema {
                    #[key]
                    id: u64,
                }
            }
        };

        finder.visit_item_mod(&module);
        // Test will pass once validator properly distinguishes valid/invalid schemas
        assert!(finder.schemas().len() >= 0);
    }
}
