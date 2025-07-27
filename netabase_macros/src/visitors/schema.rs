//! Schema processing and discovery for netabase macros
//!
//! This module handles the discovery, validation, and processing of netabase schemas
//! within Rust modules and items, providing comprehensive schema management functionality.

use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    AttrStyle, Field, Fields, Ident, Item, ItemEnum, ItemMod, ItemStruct, Meta, Path, PathSegment,
    Type, Variant, punctuated::Punctuated, spanned::Spanned, token::Token, visit::Visit,
};

use crate::visitors::{
    errors::{NetabaseError, NetabaseResult},
    key_generation::{KeyGenerator, KeyGeneratorLabels},
    validation::{ValidationContext, ValidationResult, Validator},
};

/// Information about a discovered schema
#[derive(Debug, Clone)]
pub struct SchemaInfo<'ast> {
    pub name: Ident,
    pub path: Path,
    pub schema_type: SchemaType,
    pub key_fields: Vec<(&'ast Field, KeyGenerator<'ast>)>,
}

/// Types of schemas that can be processed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    Struct,
    Enum,
}

/// Validation item categorization
#[derive(Debug)]
pub enum ValidationItem<'a> {
    ValidSchema(Ident),
    Invalid,
    Module(&'a ItemMod),
}

/// Schema discovery and validation
#[derive(Default)]
pub struct SchemaProcessor<'ast> {
    pub discovered_schemas: Vec<SchemaInfo<'ast>>,
    pub current_path: Punctuated<PathSegment, syn::Token![::]>,
    pub validation_context: ValidationContext,
}

impl<'ast> SchemaProcessor<'ast> {
    /// Create a new schema processor
    pub fn new() -> Self {
        Self {
            discovered_schemas: Vec::new(),
            current_path: Punctuated::new(),
            validation_context: ValidationContext::new(),
        }
    }

    /// Create a schema processor with custom validation context
    pub fn with_context(context: ValidationContext) -> Self {
        Self {
            discovered_schemas: Vec::new(),
            current_path: Punctuated::new(),
            validation_context: context,
        }
    }

    /// Get all discovered schemas
    pub fn get_schemas(&self) -> &[SchemaInfo<'ast>] {
        &self.discovered_schemas
    }

    /// Get schemas filtered by type
    pub fn get_schemas_by_type(&self, schema_type: SchemaType) -> Vec<&SchemaInfo<'ast>> {
        self.discovered_schemas
            .iter()
            .filter(|info| info.schema_type == schema_type)
            .collect()
    }

    /// Get the current module path
    pub fn current_path(&self) -> &Punctuated<PathSegment, syn::Token![::]> {
        &self.current_path
    }

    /// Validate and categorize a schema item
    pub fn validate_schema_item(&self, item: &'ast Item) -> ValidationItem<'ast> {
        let trait_name = "NetabaseSchema";

        let (attributes, item_ident) = match item {
            Item::Enum(item_enum) => (item_enum.attrs.clone(), item_enum.ident.clone()),
            Item::Struct(item_struct) => (item_struct.attrs.clone(), item_struct.ident.clone()),
            Item::Mod(m) => {
                return ValidationItem::Module(m);
            }
            _ => {
                return ValidationItem::Invalid;
            }
        };

        // Check for NetabaseSchema derive attribute
        let has_derive = attributes.iter().any(|att| match (&att.style, &att.meta) {
            (AttrStyle::Outer, Meta::List(meta_list)) => {
                if meta_list.path.is_ident("derive") {
                    att.parse_nested_meta(|meta| {
                        if meta.path.is_ident(trait_name) {
                            Ok(())
                        } else {
                            Err(syn::Error::new(
                                item_ident.span(),
                                format!(
                                    "Schema: {} should derive {trait_name}",
                                    item_ident.clone()
                                ),
                            ))
                        }
                    })
                    .is_ok()
                } else {
                    false
                }
            }
            _ => false,
        });

        if has_derive {
            ValidationItem::ValidSchema(item_ident)
        } else {
            ValidationItem::Invalid
        }
    }

    /// Process a struct schema
    pub fn process_struct_schema(&mut self, item_struct: &'ast ItemStruct) -> NetabaseResult<()> {
        let key_fields = self.extract_key_fields(&item_struct.fields)?;

        // Validate the struct schema
        let context = self
            .validation_context
            .clone()
            .with_item(item_struct.ident.clone(), None);

        let validation_result =
            crate::visitors::validation::SchemaValidator::validate_struct_schema(
                item_struct,
                &context,
            );

        if !validation_result.is_valid {
            // Return the first error if validation failed
            if let Some(error) = validation_result.errors.into_iter().next() {
                return Err(error);
            }
        }

        // Create schema info
        let mut item_path = self.current_path.clone();
        item_path.push(item_struct.ident.clone().into());

        let schema_info = SchemaInfo {
            name: item_struct.ident.clone(),
            path: Path {
                leading_colon: None,
                segments: item_path,
            },
            schema_type: SchemaType::Struct,
            key_fields,
        };

        self.discovered_schemas.push(schema_info);
        Ok(())
    }

    /// Process an enum schema
    pub fn process_enum_schema(&mut self, item_enum: &'ast ItemEnum) -> NetabaseResult<()> {
        let mut all_key_fields = Vec::new();

        // Process each variant
        for variant in &item_enum.variants {
            let variant_key_fields = self.extract_key_fields(&variant.fields)?;
            all_key_fields.extend(variant_key_fields);
        }

        // Validate the enum schema
        let context = self
            .validation_context
            .clone()
            .with_item(item_enum.ident.clone(), None);

        let validation_result =
            crate::visitors::validation::SchemaValidator::validate_enum_schema(item_enum, &context);

        if !validation_result.is_valid {
            // Return the first error if validation failed
            if let Some(error) = validation_result.errors.into_iter().next() {
                return Err(error);
            }
        }

        // Create schema info
        let mut item_path = self.current_path.clone();
        item_path.push(item_enum.ident.clone().into());

        let schema_info = SchemaInfo {
            name: item_enum.ident.clone(),
            path: Path {
                leading_colon: None,
                segments: item_path,
            },
            schema_type: SchemaType::Enum,
            key_fields: all_key_fields,
        };

        self.discovered_schemas.push(schema_info);
        Ok(())
    }

    /// Extract key fields from a Fields collection
    pub fn extract_key_fields(
        &self,
        fields: &'ast Fields,
    ) -> NetabaseResult<Vec<(&'ast Field, KeyGenerator<'ast>)>> {
        let mut key_fields = Vec::new();

        match fields {
            Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    // Use unwrap_or to gracefully handle any parsing errors
                    let key_gen = self
                        .analyze_field_for_key(field)
                        .unwrap_or(KeyGenerator::None);
                    if !matches!(key_gen, KeyGenerator::None) {
                        key_fields.push((field, key_gen));
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                for field in &fields_unnamed.unnamed {
                    // Use unwrap_or to gracefully handle any parsing errors
                    let key_gen = self
                        .analyze_field_for_key(field)
                        .unwrap_or(KeyGenerator::None);
                    if !matches!(key_gen, KeyGenerator::None) {
                        key_fields.push((field, key_gen));
                    }
                }
            }
            Fields::Unit => {
                // Unit structs/variants have no fields
            }
        }

        Ok(key_fields)
    }

    /// Analyze a field to determine its key generation strategy
    pub fn analyze_field_for_key(&self, field: &'ast Field) -> NetabaseResult<KeyGenerator<'ast>> {
        for attr in &field.attrs {
            match &attr.meta {
                // Simple key attribute: #[key]
                Meta::Path(path) if path.is_ident("key") || path.is_ident("NetabaseKey") => {
                    return Ok(KeyGenerator::Field(field));
                }

                // Key list attribute: #[key(...)]
                Meta::List(meta_list)
                    if meta_list.path.is_ident("key") || meta_list.path.is_ident("NetabaseKey") =>
                {
                    // For now, treat list attributes as simple field keys
                    // This could be extended to handle more complex configurations
                    return Ok(KeyGenerator::Field(field));
                }

                // Key with value: #[key = something] - parse closure if present
                Meta::NameValue(named_value)
                    if named_value.path.is_ident("key")
                        || named_value.path.is_ident("NetabaseKey") =>
                {
                    match &named_value.value {
                        syn::Expr::Closure(closure) => {
                            // Check if closure takes field value (FieldClosure) or item (ItemClosure)
                            if self.is_item_level_closure(closure, field) {
                                return Ok(KeyGenerator::ItemClosure(closure));
                            } else {
                                return Ok(KeyGenerator::FieldClosure(closure));
                            }
                        }
                        _ => {
                            // Not a closure, treat as simple field key
                            return Ok(KeyGenerator::Field(field));
                        }
                    }
                }
                _ => continue,
            }
        }

        Ok(KeyGenerator::None)
    }

    /// Determine if a closure is item-level (takes the whole struct/enum) or field-level
    fn is_item_level_closure(&self, closure: &syn::ExprClosure, field: &Field) -> bool {
        // Analyze the closure input to determine if it's item-level or field-level
        if let Some(input) = closure.inputs.first() {
            if let syn::Pat::Type(pat_type) = input {
                let input_type_str = pat_type.ty.to_token_stream().to_string();
                let field_type_str = field.ty.to_token_stream().to_string();

                // If the input type doesn't match the field type, it's likely item-level
                if input_type_str != field_type_str {
                    // Check if it's a reference to the field type
                    if let syn::Type::Reference(type_ref) = pat_type.ty.as_ref() {
                        let referenced_type_str = type_ref.elem.to_token_stream().to_string();
                        return referenced_type_str != field_type_str;
                    }
                    return true;
                }
            }
        }

        // Default to field-level if we can't determine
        false
    }

    /// Generate key function labels for a schema
    pub fn generate_key_labels(
        &self,
        schema_info: &SchemaInfo<'ast>,
    ) -> Vec<KeyGeneratorLabels<'ast>> {
        let mut labels = Vec::new();

        for (field, key_gen) in &schema_info.key_fields {
            let key_labels = KeyGeneratorLabels::new(field, key_gen.clone());
            labels.push(key_labels);
        }

        labels
    }

    /// Get the type of a schema item
    fn get_item_type_from_schema(&self, schema_info: &SchemaInfo<'ast>) -> Type {
        // Create a type path from the schema name
        let ident = &schema_info.name;
        syn::parse_quote!(#ident)
    }
}

impl<'ast> Visit<'ast> for SchemaProcessor<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        match self.validate_schema_item(item) {
            ValidationItem::ValidSchema(_ident_name) => {
                match item {
                    Item::Struct(item_struct) => {
                        if let Err(_err) = self.process_struct_schema(item_struct) {
                            // For now, we'll ignore errors but could enhance error handling
                            eprintln!("Failed to process struct schema: {}", item_struct.ident);
                        }
                    }
                    Item::Enum(item_enum) => {
                        if let Err(_err) = self.process_enum_schema(item_enum) {
                            // For now, we'll ignore errors but could enhance error handling
                            eprintln!("Failed to process enum schema: {}", item_enum.ident);
                        }
                    }
                    _ => {}
                }
            }
            ValidationItem::Invalid => {
                // Item is not a valid schema, ignore
            }
            ValidationItem::Module(item_mod) => {
                self.visit_item_mod(item_mod);
            }
        }
    }

    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        let mod_name = item_mod.ident.clone();
        self.current_path.push(mod_name.into());

        // Continue visiting the module contents
        syn::visit::visit_item_mod(self, item_mod);

        // Remove the module name from the path when done
        self.current_path.pop();
    }
}

/// Legacy schema finder for backward compatibility
#[derive(Default)]
pub struct ValidSchemaFinder<'ast> {
    pub valid_schemas: Vec<(&'ast Item, Path)>,
    pub current_path: Punctuated<PathSegment, syn::Token![::]>,
}

impl<'ast> ValidSchemaFinder<'ast> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'ast> Visit<'ast> for ValidSchemaFinder<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        let processor = SchemaProcessor::new();
        match processor.validate_schema_item(item) {
            ValidationItem::ValidSchema(ident_name) => {
                let mut item_path = self.current_path.clone();
                item_path.push(ident_name.into());
                self.valid_schemas.push((
                    item,
                    Path {
                        leading_colon: None,
                        segments: item_path,
                    },
                ));
            }
            ValidationItem::Invalid => {}
            ValidationItem::Module(item_mod) => self.visit_item_mod(item_mod),
        }
    }

    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        let mod_name = item_mod.ident.clone();
        self.current_path.push(mod_name.into());
        syn::visit::visit_item_mod(self, item_mod);
        self.current_path.pop();
    }
}

/// Legacy schema validator for backward compatibility
#[derive(Default)]
pub struct SchemaValidator<'ast> {
    pub key_fields: Vec<(&'ast Field, KeyGenerator<'ast>)>,
    pub enum_key_closure: Option<&'ast syn::ExprClosure>,
}

impl<'ast> SchemaValidator<'ast> {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze enum attributes for top-level key closure
    pub fn analyze_enum_for_key_closure(
        &self,
        item_enum: &'ast syn::ItemEnum,
    ) -> Option<&'ast syn::ExprClosure> {
        for attr in &item_enum.attrs {
            if let syn::Meta::NameValue(named_value) = &attr.meta {
                if named_value.path.is_ident("key") || named_value.path.is_ident("NetabaseKey") {
                    if let syn::Expr::Closure(closure) = &named_value.value {
                        return Some(closure);
                    }
                }
            }
        }
        None
    }

    /// Analyze a field for key generation (legacy method)
    pub fn is_key(field: &'ast Field) -> KeyGenerator<'ast> {
        let processor = SchemaProcessor::new();
        processor
            .analyze_field_for_key(field)
            .unwrap_or(KeyGenerator::None)
    }

    /// Extract key fields from a Fields collection (legacy method)
    pub fn keys(fields: &'ast Fields) -> Vec<(&'ast Field, KeyGenerator<'ast>)> {
        let processor = SchemaProcessor::new();
        processor.extract_key_fields(fields).unwrap_or_default()
    }
}

impl<'ast> Visit<'ast> for SchemaValidator<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Enum(item_enum) => {
                self.visit_item_enum(item_enum);
            }
            Item::Struct(item_struct) => {
                self.visit_item_struct(item_struct);
            }
            _ => {}
        }
    }

    fn visit_fields(&mut self, fields: &'ast Fields) {
        let keys = Self::keys(fields);
        // Allow schemas without explicit key fields - they can still be valid
        self.key_fields.extend(keys);
    }

    fn visit_item_enum(&mut self, item_enum: &'ast ItemEnum) {
        // Check for enum-level key closure
        self.enum_key_closure = self.analyze_enum_for_key_closure(item_enum);

        if !item_enum.variants.is_empty() {
            for variant in &item_enum.variants {
                self.visit_fields(&variant.fields);
            }
        }
    }

    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        self.visit_fields(&item_struct.fields);
    }
}

/// Utility functions for schema processing
pub mod utils {
    use super::*;

    /// Quick check if an item is a valid netabase schema
    pub fn is_valid_schema(item: &Item) -> bool {
        let processor = SchemaProcessor::new();
        matches!(
            processor.validate_schema_item(item),
            ValidationItem::ValidSchema(_)
        )
    }

    /// Extract all schemas from a module
    pub fn extract_schemas_from_module(module: &ItemMod) -> Vec<SchemaInfo> {
        let mut processor = SchemaProcessor::new();
        processor.visit_item_mod(module);
        processor.discovered_schemas
    }

    /// Get the schema type of an item
    pub fn get_schema_type(item: &Item) -> Option<SchemaType> {
        match item {
            Item::Struct(_) => Some(SchemaType::Struct),
            Item::Enum(_) => Some(SchemaType::Enum),
            _ => None,
        }
    }
}
