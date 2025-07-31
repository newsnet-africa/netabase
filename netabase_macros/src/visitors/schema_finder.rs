use std::collections::HashMap;
use syn::{
    Attribute, Fields, Generics, Ident, Item, ItemEnum, ItemStruct, PathSegment, Token, Variant,
    Visibility, punctuated::Punctuated, spanned::Spanned, token::Comma, visit::Visit,
};

use crate::visitors::{
    key_finder::{KeyInfoBuilder, KeyValidator},
    schema_validator::SchemaValidator,
    utils::SchemaInfo,
};

/// Finds and validates schemas within modules
#[derive(Default)]
pub struct SchemaFinder<'ast> {
    current_path: Punctuated<PathSegment, Token![::]>,
    schemas: Vec<SchemaInfo<'ast>>,
    schema_validator: SchemaValidator,
    key_validator: KeyValidator,
}

impl<'ast> SchemaFinder<'ast> {
    /// Create a new schema finder
    pub fn new() -> Self {
        Self {
            current_path: Punctuated::new(),
            schemas: Vec::new(),
            schema_validator: SchemaValidator::new(),
            key_validator: KeyValidator::new(),
        }
    }

    /// Get all found schemas
    pub fn schemas(&self) -> &[SchemaInfo<'ast>] {
        &self.schemas
    }

    /// Get found schemas by consuming the finder
    pub fn into_schemas(self) -> Vec<SchemaInfo<'ast>> {
        self.schemas
    }

    /// Process an item and add it to schemas if valid
    fn process_item(&mut self, item: &'ast Item) -> Option<SchemaInfo<'ast>> {
        // First, validate that the item is a valid schema type
        let schema_type = match self.schema_validator.validate_schema_item(item) {
            Ok(schema_type) => schema_type,
            Err(_) => return None, // Not a valid schema, skip silently
        };

        // Check if it has the NetabaseSchema derive
        if !self.schema_validator.has_netabase_derive(&schema_type) {
            return None; // Not annotated with NetabaseSchema, skip
        }

        // Validate the NetabaseSchema requirements
        if let Err(_) = self.schema_validator.validate_netabase_schema(&schema_type) {
            return None; // Invalid NetabaseSchema, skip
        }

        // Extract and validate keys
        let key_info = match self.key_validator.validate_and_extract_keys(&schema_type) {
            Ok(key_type) => KeyInfoBuilder::new().with_key_type(key_type).build(),
            Err(_) => return None, // Invalid keys, skip
        };

        // Create the full path for this schema
        let mut schema_path = self.current_path.clone();
        schema_path.push(PathSegment {
            ident: schema_type.identity().clone(),
            arguments: syn::PathArguments::None,
        });

        // Create and return the schema info
        Some(SchemaInfo {
            schema_type: Some(schema_type),
            path: schema_path,
            schema_key: Some(key_info),
        })
    }
}

impl<'ast> Visit<'ast> for SchemaFinder<'ast> {
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        // Add current module to path
        self.current_path.push(PathSegment {
            ident: module.ident.clone(),
            arguments: syn::PathArguments::None,
        });

        // Process module contents
        if let Some((_, items)) = &module.content {
            for item in items {
                match item {
                    Item::Mod(nested_module) => {
                        // Recursively visit nested modules
                        self.visit_item_mod(nested_module);
                    }
                    _ => {
                        // Process potential schema items
                        if let Some(schema_info) = self.process_item(item) {
                            self.schemas.push(schema_info);
                        }
                    }
                }
            }
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
