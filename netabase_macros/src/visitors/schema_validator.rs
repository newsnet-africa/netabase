use crate::visitors::schema_finder::SchemaType;
use syn::{Attribute, Item, ItemEnum, ItemStruct};

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

/// Validates individual schemas for NetabaseSchema derive compliance
#[derive(Default)]
pub struct SchemaValidator;

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate that an item is a valid schema (struct or enum)
    pub fn validate_schema_item<'a>(&self, item: &'a Item) -> ValidationResult<SchemaType<'a>> {
        match item {
            Item::Struct(item_struct) => {
                self.validate_struct(item_struct)?;
                Ok(SchemaType::Struct(item_struct))
            }
            Item::Enum(item_enum) => {
                self.validate_enum(item_enum)?;
                Ok(SchemaType::Enum(item_enum))
            }
            _ => Err(ValidationError::new(
                "Schema can only be applied to structs or enums",
            )),
        }
    }

    /// Validate a struct for schema compliance
    fn validate_struct(&self, item_struct: &ItemStruct) -> ValidationResult<()> {
        // Validate field structure
        match &item_struct.fields {
            syn::Fields::Named(_) => Ok(()),
            syn::Fields::Unnamed(_) => Ok(()),
            syn::Fields::Unit => Err(ValidationError::new(
                "Unit structs cannot be used as schemas",
            )),
        }
    }

    /// Validate an enum for schema compliance
    fn validate_enum(&self, item_enum: &ItemEnum) -> ValidationResult<()> {
        // Ensure enum has variants
        if item_enum.variants.is_empty() {
            return Err(ValidationError::new(
                "Enums used as schemas must have at least one variant",
            ));
        }

        Ok(())
    }

    /// Check if the schema has the NetabaseSchema derive attribute
    pub fn has_netabase_derive(&self, schema_type: &SchemaType) -> bool {
        schema_type.attributes().iter().any(|attr| {
            if let syn::Meta::List(list) = &attr.meta {
                list.path.is_ident("derive") && list.tokens.to_string().contains("NetabaseSchema")
            } else {
                false
            }
        })
    }

    /// Validate that a schema is properly annotated for netabase
    pub fn validate_netabase_schema(&self, schema_type: &SchemaType) -> ValidationResult<()> {
        if !self.has_netabase_derive(schema_type) {
            return Err(ValidationError::new("Schema must derive NetabaseSchema"));
        }

        // Note: We don't validate Encode/Decode derives here because they may be
        // implemented manually by the user, or the derives may be processed after
        // our macro runs.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_valid_struct_schema() {
        let validator = SchemaValidator::new();
        let item: Item = parse_quote! {
            #[derive(NetabaseSchema)]
            struct TestSchema {
                #[key]
                id: u64,
                name: String,
            }
        };

        let schema_type = validator.validate_schema_item(&item).unwrap();
        assert!(validator.validate_netabase_schema(&schema_type).is_ok());
    }

    #[test]
    fn test_invalid_unit_struct() {
        let validator = SchemaValidator::new();
        let item: Item = parse_quote! {
            struct TestSchema;
        };

        assert!(validator.validate_schema_item(&item).is_err());
    }

    #[test]
    fn test_valid_enum_schema() {
        let validator = SchemaValidator::new();
        let item: Item = parse_quote! {
            #[derive(NetabaseSchema)]
            enum TestEnum {
                Variant1,
                Variant2,
            }
        };

        let schema_type = validator.validate_schema_item(&item).unwrap();
        assert!(validator.validate_netabase_schema(&schema_type).is_ok());
    }
}
