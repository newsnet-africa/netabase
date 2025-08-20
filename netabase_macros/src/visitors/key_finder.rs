use crate::visitors::utils::FieldKeyInfo;
use crate::visitors::{
    schema_validator::{ValidationError, ValidationResult},
    utils::schema_finder::SchemaType,
    utils::{KeyInfo, KeyType},
};
use std::collections::HashMap;
use syn::{
    Expr, ExprClosure, Field, Fields, GenericArgument, Lit, Meta, PathArguments, ReturnType, Type,
    Variant,
};

/// Validates and extracts key information from schemas
#[derive(Default)]
pub struct KeyValidator;

impl KeyValidator {
    /// Create a new key validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract and validate key information from a schema
    pub fn validate_and_extract_keys<'ast>(
        &self,
        schema: &SchemaType<'ast>,
    ) -> ValidationResult<KeyType<'ast>> {
        // Try to get outer key first (key attribute on the schema itself)
        if let Some(outer_key) = self.extract_outer_key(schema)? {
            // If we have an outer key, make sure there are no field keys
            if self.has_field_keys(schema) {
                return Err(ValidationError::new(
                    "Schema key closures and field keys are mutually exclusive",
                ));
            }
            return Ok(outer_key);
        }

        // If no outer key, look for field keys
        let field_keys = self.extract_field_keys(schema)?;
        if field_keys.is_empty() {
            return Err(ValidationError::new(
                "Schema must have at least one key (either outer key or field key)",
            ));
        }

        Ok(KeyType::FieldKeys(field_keys))
    }

    /// Extract outer key (key attribute on schema) if present
    fn extract_outer_key<'ast>(
        &self,
        schema: &SchemaType<'ast>,
    ) -> ValidationResult<Option<KeyType<'ast>>> {
        for attr in schema.attributes() {
            if let Meta::NameValue(nv) = &attr.meta {
                if nv.path.is_ident("key") {
                    if let Expr::Closure(closure) = &nv.value {
                        self.validate_key_closure(closure, schema)?;
                        return Ok(Some(KeyType::SchemaKey(closure)));
                    } else {
                        return Err(ValidationError::new("Schema key must be a closure"));
                    }
                } else if nv.path.is_ident("key_fn") {
                    if let Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            let func_name = lit_str.value();
                            self.validate_key_function(&func_name, schema)?;
                            return Ok(Some(KeyType::KeyFunction(func_name)));
                        } else {
                            return Err(ValidationError::new("key_fn must be a string literal"));
                        }
                    } else {
                        return Err(ValidationError::new("key_fn must be a string literal"));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Extract field keys from the schema
    fn extract_field_keys<'ast>(
        &self,
        schema: &SchemaType<'ast>,
    ) -> ValidationResult<HashMap<Option<&'ast Variant>, Vec<FieldKeyInfo<'ast>>>> {
        let mut field_keys = HashMap::new();
        let fields = schema.fields();

        for (variant, fields) in fields {
            let key_field_infos = self.find_key_fields(fields)?;
            if !key_field_infos.is_empty() {
                field_keys.insert(variant, key_field_infos);
            }
        }

        // Additional validation: check for unsupported patterns
        self.validate_key_patterns(schema, &field_keys)?;

        Ok(field_keys)
    }

    /// Check if schema has any field keys
    fn has_field_keys(&self, schema: &SchemaType) -> bool {
        let fields = schema.fields();
        fields.values().any(|fields| self.field_has_key(fields))
    }

    /// Find the key fields in a Fields structure
    fn find_key_fields<'ast>(
        &self,
        fields: &'ast Fields,
    ) -> ValidationResult<Vec<FieldKeyInfo<'ast>>> {
        match fields {
            Fields::Named(named) => {
                let key_fields: Vec<FieldKeyInfo<'ast>> = named
                    .named
                    .iter()
                    .filter(|field| self.is_key_field(field))
                    .map(|field| FieldKeyInfo {
                        field,
                        index: None, // Named fields don't need indices
                    })
                    .collect();

                Ok(key_fields)
            }
            Fields::Unnamed(unnamed) => {
                let key_fields: Vec<FieldKeyInfo<'ast>> = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| self.is_key_field(field))
                    .map(|(index, field)| FieldKeyInfo {
                        field,
                        index: Some(index),
                    })
                    .collect();

                Ok(key_fields)
            }
            Fields::Unit => Ok(Vec::new()),
        }
    }

    /// Check if a field has the key attribute
    fn is_key_field(&self, field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|attr| attr.meta.path().is_ident("key"))
    }

    /// Check if any field in Fields has a key attribute
    fn field_has_key(&self, fields: &Fields) -> bool {
        match fields {
            Fields::Named(named) => named.named.iter().any(|field| self.is_key_field(field)),
            Fields::Unnamed(unnamed) => {
                unnamed.unnamed.iter().any(|field| self.is_key_field(field))
            }
            Fields::Unit => false,
        }
    }

    /// Validate that a key closure is properly formed
    fn validate_key_closure<'ast>(
        &self,
        closure: &ExprClosure,
        schema: &SchemaType<'ast>,
    ) -> ValidationResult<()> {
        // Check that closure has exactly one input
        if closure.inputs.len() != 1 {
            return Err(ValidationError::new(
                "Key closure must have exactly one parameter",
            ));
        }

        // Check that the input type matches the schema type
        if let Some(input) = closure.inputs.first() {
            match input {
                syn::Pat::Type(pat_type) => {
                    self.validate_closure_input_type(pat_type, schema)?;
                }
                syn::Pat::Ident(pat_ident) => {
                    // Allow untyped parameters, but warn that it's less strict
                    // The compiler will catch type mismatches anyway
                }
                _ => {
                    return Err(ValidationError::new(
                        "Closure input must be either a typed parameter (e.g., 'item: MyStruct') or an identifier (e.g., 'item')",
                    ));
                }
            }
        }

        // Validate the return type if explicitly specified
        self.validate_closure_return_type(closure)?;

        Ok(())
    }

    /// Validate the closure input type matches the schema
    fn validate_closure_input_type(
        &self,
        pat_type: &syn::PatType,
        schema: &SchemaType,
    ) -> ValidationResult<()> {
        match pat_type.ty.as_ref() {
            syn::Type::Path(type_path) => {
                // Handle both direct types (MyStruct) and references (&MyStruct)
                let type_ident = if let Some(segment) = type_path.path.segments.last() {
                    &segment.ident
                } else {
                    return Err(ValidationError::new(
                        "Invalid type path in closure parameter",
                    ));
                };

                if type_ident != schema.identity() {
                    return Err(ValidationError::new(format!(
                        "Closure input type must be {} or &{}, found {}",
                        schema.identity(),
                        schema.identity(),
                        type_ident
                    )));
                }
            }
            syn::Type::Reference(type_ref) => {
                if let syn::Type::Path(type_path) = type_ref.elem.as_ref() {
                    let type_ident = if let Some(segment) = type_path.path.segments.last() {
                        &segment.ident
                    } else {
                        return Err(ValidationError::new(
                            "Invalid type path in closure parameter",
                        ));
                    };

                    if type_ident != schema.identity() {
                        return Err(ValidationError::new(format!(
                            "Closure input type must be {} or &{}, found &{}",
                            schema.identity(),
                            schema.identity(),
                            type_ident
                        )));
                    }
                } else {
                    return Err(ValidationError::new(
                        "Closure input reference must point to a path type",
                    ));
                }
            }
            _ => {
                return Err(ValidationError::new(
                    "Closure input must be a path type or reference to a path type",
                ));
            }
        }
        Ok(())
    }

    /// Validate the closure return type is encodable/decodable
    fn validate_closure_return_type(&self, closure: &ExprClosure) -> ValidationResult<()> {
        if let ReturnType::Type(_, return_type) = &closure.output {
            self.validate_encodable_type(return_type)?;
        }
        // If no explicit return type, we'll rely on type inference and compilation
        Ok(())
    }

    /// Validate that a type can be encoded/decoded with bincode
    fn validate_encodable_type(&self, ty: &Type) -> ValidationResult<()> {
        match ty {
            // Basic encodable types
            Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    let type_name = segment.ident.to_string();
                    match type_name.as_str() {
                        // Primitive types that are always encodable
                        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                        | "i64" | "i128" | "isize" | "f32" | "f64" | "bool" | "char" | "String" => {
                            Ok(())
                        }

                        // Common standard library types
                        "Vec" | "Option" | "Result" => {
                            // For generic types, validate their parameters
                            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                                for arg in &args.args {
                                    if let GenericArgument::Type(inner_type) = arg {
                                        self.validate_encodable_type(inner_type)?;
                                    }
                                }
                            }
                            Ok(())
                        }

                        // For other types, we'll assume they implement Encode/Decode
                        // The compiler will catch it if they don't
                        _ => Ok(()),
                    }
                } else {
                    Err(ValidationError::new("Invalid type path in return type"))
                }
            }

            // Arrays are encodable if their element type is
            Type::Array(type_array) => self.validate_encodable_type(&type_array.elem),

            // Tuples are encodable if all their elements are
            Type::Tuple(type_tuple) => {
                for elem in &type_tuple.elems {
                    self.validate_encodable_type(elem)?;
                }
                Ok(())
            }

            // References to encodable types are fine for key generation
            Type::Reference(type_ref) => self.validate_encodable_type(&type_ref.elem),

            // Slices are encodable if converted to owned types
            Type::Slice(type_slice) => self.validate_encodable_type(&type_slice.elem),

            _ => {
                // For other types, we'll be permissive and let the compiler handle it
                Ok(())
            }
        }
    }

    /// Validate that a key function name is properly formed
    fn validate_key_function(&self, func_name: &str, _schema: &SchemaType) -> ValidationResult<()> {
        // Basic validation - function name should be a valid Rust identifier
        if func_name.is_empty() {
            return Err(ValidationError::new("Key function name cannot be empty"));
        }

        // Check if it's a valid Rust identifier
        if !func_name.chars().next().unwrap_or('0').is_alphabetic() && !func_name.starts_with('_') {
            return Err(ValidationError::new(
                "Key function name must start with a letter or underscore",
            ));
        }

        for ch in func_name.chars() {
            if !ch.is_alphanumeric() && ch != '_' {
                return Err(ValidationError::new(
                    "Key function name must contain only alphanumeric characters and underscores",
                ));
            }
        }

        Ok(())
    }
}

/// Builder for key information
#[derive(Default)]
pub struct KeyInfoBuilder<'ast> {
    key_type: Option<KeyType<'ast>>,
}

impl<'ast> KeyInfoBuilder<'ast> {
    pub fn new() -> Self {
        Self { key_type: None }
    }

    pub fn with_key_type(mut self, key_type: KeyType<'ast>) -> Self {
        self.key_type = Some(key_type);
        self
    }

    pub fn build(self) -> KeyInfo<'ast> {
        KeyInfo::new(self.key_type.unwrap_or(KeyType::FieldKeys(HashMap::new())))
    }
}

impl KeyValidator {
    /// Validate key patterns and reject unsupported configurations
    fn validate_key_patterns<'ast>(
        &self,
        schema: &SchemaType<'ast>,
        field_keys: &HashMap<Option<&'ast Variant>, Vec<FieldKeyInfo<'ast>>>,
    ) -> ValidationResult<()> {
        // Check for enum variants with multiple key fields (not supported yet)
        if matches!(schema, SchemaType::Enum(_)) && field_keys.len() > 1 {
            return Err(ValidationError::new(
                "UNSUPPORTED: Multiple enum variants with #[key] fields are not yet supported. \
                Current implementation only supports single-variant key extraction. \
                Please use only one variant with a #[key] field, or implement a custom key() method.",
            ));
        }

        // Check for complex key types that aren't supported
        for (variant, field_infos) in field_keys {
            for field_info in field_infos {
                self.validate_key_field_type(&field_info.field.ty, variant)?;
            }
        }

        Ok(())
    }

    /// Validate that the key field type is supported
    fn validate_key_field_type<'ast>(
        &self,
        field_type: &Type,
        variant: &Option<&'ast Variant>,
    ) -> ValidationResult<()> {
        match field_type {
            Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    let type_name = segment.ident.to_string();

                    // Check for unsupported types
                    match type_name.as_str() {
                        "Vec" | "HashMap" | "BTreeMap" | "HashSet" | "BTreeSet" => {
                            return Err(ValidationError::new(format!(
                                "UNSUPPORTED: Collection type '{}' cannot be used as a key field. \
                                    Only primitive types (u8, u16, u32, u64, i8, i16, i32, i64, String, bool) \
                                    are currently supported as key fields. \
                                    Consider using a String representation or implement a custom key() method.",
                                type_name
                            )));
                        }
                        "Option" => {
                            return Err(ValidationError::new(
                                "UNSUPPORTED: Option<T> cannot be used as a key field. \
                                Key fields must always have a value. \
                                Consider using a default value or implement a custom key() method."
                                    .to_string(),
                            ));
                        }
                        _ => {
                            // Check if it's a generic type with parameters
                            if !segment.arguments.is_empty() {
                                return Err(ValidationError::new(format!(
                                    "UNSUPPORTED: Generic type '{}' with parameters cannot be used as a key field. \
                                        Only simple primitive types are currently supported as key fields. \
                                        Consider using a String representation or implement a custom key() method.",
                                    type_name
                                )));
                            }
                        }
                    }
                }
            }
            Type::Array(_) | Type::Slice(_) => {
                return Err(ValidationError::new(
                    "UNSUPPORTED: Array and slice types cannot be used as key fields. \
                    Only primitive types (u8, u16, u32, u64, i8, i16, i32, i64, String, bool) \
                    are currently supported as key fields. \
                    Consider using a String representation or implement a custom key() method."
                        .to_string(),
                ));
            }
            Type::Tuple(_) => {
                return Err(ValidationError::new(
                    "UNSUPPORTED: Tuple types cannot be used as key fields. \
                    Only primitive types (u8, u16, u32, u64, i8, i16, i32, i64, String, bool) \
                    are currently supported as key fields. \
                    Consider using a String representation or implement a custom key() method."
                        .to_string(),
                ));
            }
            _ => {
                return Err(ValidationError::new(
                    "UNSUPPORTED: Complex types cannot be used as key fields. \
                    Only primitive types (u8, u16, u32, u64, i8, i16, i32, i64, String, bool) \
                    are currently supported as key fields. \
                    Consider using a String representation or implement a custom key() method."
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_field_key_validation() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            struct TestSchema {
                #[key]
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::FieldKeys(fields) => assert_eq!(fields.len(), 1),
            _ => panic!("Expected field keys"),
        }
    }

    #[test]
    fn test_multiple_key_fields_error() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            struct TestSchema {
                #[key]
                id: u64,
                #[key]
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_outer_key_validation() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |s: TestSchema| s.id]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_closure_with_reference_parameter() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |item: &TestSchema| item.id]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_closure_with_untyped_parameter() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |item| item.id]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_closure_with_explicit_return_type() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |item: TestSchema| -> u64 { item.id }]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_closure_with_complex_return_type() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |item: TestSchema| -> Vec<u8> { vec![item.id as u8] }]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_closure_with_tuple_return_type() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |item: TestSchema| -> (u64, String) { (item.id, item.name.clone()) }]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_closure_wrong_input_type() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |s: WrongType| s.id]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_closure_multiple_parameters() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |s: TestSchema, extra: u64| s.id + extra]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_closure_no_parameters() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = || 42u64]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_enum_closure_validation() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |item: MyEnum| match item {
                MyEnum::Variant1 { id } => *id,
                MyEnum::Variant2 { id } => *id,
            }]
            enum MyEnum {
                Variant1 { id: u64 },
                Variant2 { id: u64 },
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::SchemaKey(_) => (),
            _ => panic!("Expected schema key"),
        }
    }

    #[test]
    fn test_conflicting_field_and_closure_keys() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key = |s: TestSchema| s.id]
            struct TestSchema {
                #[key]
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_function_key_validation() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = "test_key_func"]
            struct TestSchema {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::KeyFunction(func_name) => {
                assert_eq!(func_name, "test_key_func");
            }
            _ => panic!("Expected key function"),
        }
    }

    #[test]
    fn test_function_key_invalid_name() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = "123invalid"]
            struct TestSchema {
                id: u64,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_function_key_empty_name() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = ""]
            struct TestSchema {
                id: u64,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_function_key_with_special_chars() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = "test-key"]
            struct TestSchema {
                id: u64,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_function_key_underscore_name() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = "_private_key_func"]
            struct TestSchema {
                id: u64,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::KeyFunction(func_name) => {
                assert_eq!(func_name, "_private_key_func");
            }
            _ => panic!("Expected key function"),
        }
    }

    #[test]
    fn test_conflicting_field_and_function_keys() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = "test_func"]
            struct TestSchema {
                #[key]
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        assert!(validator.validate_and_extract_keys(&schema_type).is_err());
    }

    #[test]
    fn test_enum_function_key() {
        let validator = KeyValidator::new();
        let item: syn::Item = parse_quote! {
            #[key_fn = "enum_key_func"]
            enum TestEnum {
                Variant1 { id: u64 },
                Variant2 { id: u64 },
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let key_info = validator.validate_and_extract_keys(&schema_type).unwrap();

        match key_info {
            KeyType::KeyFunction(func_name) => {
                assert_eq!(func_name, "enum_key_func");
            }
            _ => panic!("Expected key function"),
        }
    }
}
