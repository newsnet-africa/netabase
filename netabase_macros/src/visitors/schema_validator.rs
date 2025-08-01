use crate::visitors::utils::key_finder::{get_schema_field_keys, get_schema_outer_key};
use crate::visitors::utils::schema_finder::SchemaType;
use crate::visitors::utils::{KeyType, SchemaInfo};
use syn::{Item, visit::Visit};

/// Result type for schema validation
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Errors that can occur during schema validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub span: Option<proc_macro2::Span>,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, span: proc_macro2::Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

pub struct SchemaValidator<'ast> {
    pub info: SchemaInfo<'ast>,
    pub valid_schema: bool,
}

impl<'ast> Default for SchemaValidator<'ast> {
    fn default() -> Self {
        Self {
            info: SchemaInfo::default(),
            valid_schema: false,
        }
    }
}

pub fn contains_netabase_derive<'a>(schema_type: &SchemaType<'a>) -> bool {
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
        if let Some(schema) = schema {
            match (
                get_schema_field_keys::<'ast, 'b>(schema),
                get_schema_outer_key::<'ast, 'b>(schema),
            ) {
                (KeyType::FieldKeys(hash_map), Some(KeyType::SchemaKey(_outer))) => {
                    if hash_map.is_empty() {
                        get_schema_outer_key::<'ast, 'b>(schema)
                    } else {
                        // Schema key closures and field keys are mutually exclusive
                        None
                    }
                }
                (KeyType::FieldKeys(hash_map), None) => {
                    if !hash_map.is_empty() {
                        Some(KeyType::FieldKeys(hash_map))
                    } else {
                        // At least one key is needed
                        None
                    }
                }
                (KeyType::SchemaKey(_), None) => get_schema_outer_key::<'ast, 'b>(schema),
                (KeyType::SchemaKey(_), Some(_)) => get_schema_outer_key::<'ast, 'b>(schema),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl<'ast> Visit<'ast> for SchemaValidator<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Struct(item_struct) => {
                if contains_netabase_derive(&SchemaType::Struct(item_struct)) {
                    self.info.schema_type = Some(SchemaType::Struct(item_struct));
                    self.info.schema_key = self.get_schema_key(self.info.schema_type.as_ref());
                    self.valid_schema = true;
                }
            }
            Item::Enum(item_enum) => {
                if contains_netabase_derive(&SchemaType::Enum(item_enum)) {
                    self.info.schema_type = Some(SchemaType::Enum(item_enum));
                    self.info.schema_key = self.get_schema_key(self.info.schema_type.as_ref());
                    self.valid_schema = true;
                }
            }
            _ => {}
        }
    }
}
