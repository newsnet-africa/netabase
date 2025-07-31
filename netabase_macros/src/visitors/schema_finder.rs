use std::collections::HashMap;
use syn::{
    Attribute, Fields, Generics, Ident, Item, ItemEnum, ItemStruct, PathSegment, Token, Variant,
    Visibility, punctuated::Punctuated, spanned::Spanned, token::Comma, visit::Visit,
};

use crate::SchemaValidator;
use crate::visitors::utils::schema_finder::SchemaType;
use crate::visitors::utils::{SchemaInfo, schema_validator::contains_netabase_derive};
use proc_macro::Span;
use syn::token::Semi;
use syn::{PathSegment, Token, punctuated::Punctuated, visit::Visit};

/// Finds and validates schemas within modules
#[derive(Default)]
pub struct SchemaFinder<'ast> {
    pub current_path: Punctuated<PathSegment, Token![::]>,
    pub schemas: Vec<SchemaInfo<'ast>>,
    pub schema_validator: SchemaValidator<'ast>,
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
                                ident: module.ident.clone(),
                                arguments: syn::PathArguments::None,
                            });
                            local_path
                        };
                        self.schemas.push(self.schema_validator.info.clone());
                    }
                } else if let syn::Item::Mod(item_mod) = item {
                    self.visit_item_mod(item_mod);
                } else {
                    panic!("Schema must be Struct or Enum")
                }
            });
        }

        // Remove current module from path when done
        self.current_path.pop();
    }
}

/// Schema type wrapper for easier handling
#[derive(Clone, Copy)]
pub enum SchemaType<'ast> {
    Struct(&'ast ItemStruct),
    Enum(&'ast ItemEnum),
}

impl<'ast> SchemaType<'ast> {
    /// Get the attributes of the schema
    pub fn attributes(&self) -> &'ast Vec<Attribute> {
        match self {
            SchemaType::Struct(item_struct) => &item_struct.attrs,
            SchemaType::Enum(item_enum) => &item_enum.attrs,
        }
    }

    /// Get the visibility of the schema
    pub fn visibility(&self) -> &'ast Visibility {
        match self {
            SchemaType::Struct(item_struct) => &item_struct.vis,
            SchemaType::Enum(item_enum) => &item_enum.vis,
        }
    }

    /// Get the identifier of the schema
    pub fn identity(&self) -> &'ast Ident {
        match self {
            SchemaType::Struct(item_struct) => &item_struct.ident,
            SchemaType::Enum(item_enum) => &item_enum.ident,
        }
    }

    /// Get the generics of the schema
    pub fn generics(&self) -> &'ast Generics {
        match self {
            SchemaType::Struct(item_struct) => &item_struct.generics,
            SchemaType::Enum(item_enum) => &item_enum.generics,
        }
    }

    /// Get the variants of the schema (only for enums)
    pub fn variants(&self) -> Option<&'ast Punctuated<Variant, Comma>> {
        match self {
            SchemaType::Struct(_) => None,
            SchemaType::Enum(item_enum) => Some(&item_enum.variants),
        }
    }

    /// Get all fields organized by variant (None for structs)
    pub fn fields(&self) -> HashMap<Option<&'ast Variant>, &'ast Fields> {
        match self {
            SchemaType::Struct(item_struct) => {
                let mut fields = HashMap::new();
                fields.insert(None, &item_struct.fields);
                fields
            }
            SchemaType::Enum(item_enum) => {
                let mut fields = HashMap::new();
                for variant in &item_enum.variants {
                    fields.insert(Some(variant), &variant.fields);
                }
                fields
            }
        }
    }

    /// Check if this is a struct
    pub fn is_struct(&self) -> bool {
        matches!(self, SchemaType::Struct(_))
    }

    /// Check if this is an enum
    pub fn is_enum(&self) -> bool {
        matches!(self, SchemaType::Enum(_))
    }
}

impl<'ast> TryFrom<&'ast Item> for SchemaType<'ast> {
    type Error = syn::Error;

    fn try_from(value: &'ast Item) -> Result<Self, Self::Error> {
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
        assert_eq!(finder.schemas().len(), 1);
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
        assert_eq!(finder.schemas().len(), 1);

        let schema = &finder.schemas()[0];
        assert_eq!(schema.path.len(), 3); // outer::inner::TestSchema
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
        assert_eq!(finder.schemas().len(), 1);
    }
}
