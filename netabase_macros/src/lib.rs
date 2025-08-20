#![feature(extend_one)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemMod, parse_macro_input, visit::Visit};

use crate::{
    generators::{from_traits2, schema::from_schema_for_record},
    visitors::{
        schema_finder::SchemaFinder,
        schema_validator::SchemaValidator,
        utils::{KeyType, SchemaInfo, schema_finder::SchemaType},
    },
};

mod generators;
use generators::{
    generate_all_record_conversions, generate_all_trait_impls, generate_complete_key_struct,
};
mod visitors;

/// Result type for macro operations
type MacroResult<T> = Result<T, MacroError>;

/// Errors that can occur in macro processing
#[derive(Debug)]
struct MacroError {
    message: String,
    span: Option<proc_macro2::Span>,
}

impl MacroError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    fn with_span(message: impl Into<String>, span: proc_macro2::Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

impl From<MacroError> for syn::Error {
    fn from(error: MacroError) -> Self {
        match error.span {
            Some(span) => syn::Error::new(span, error.message),
            None => syn::Error::new(proc_macro2::Span::call_site(), error.message),
        }
    }
}

/// Derive macro for individual schema validation
///
/// This macro validates that a struct or enum is properly set up to be used
/// as a schema in the netabase system. It checks for proper key configuration,
/// serde compliance, and other requirements.
///
/// # Example
/// ```rust
/// #[derive(Serialize, Deserialize, NetabaseSchema)]
/// struct User {
///     #[key]
///     id: u64,
///     name: String,
/// }
/// ```
#[proc_macro_derive(NetabaseSchema, attributes(key, key_fn))]
pub fn derive_netabase_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match process_netabase_schema_derive(&input) {
        Ok(tokens) => tokens,
        Err(error) => {
            let syn_error: syn::Error = error.into();
            syn_error.to_compile_error().into()
        }
    }
}

/// Attribute macro for schema modules that finds and validates all schemas
///
/// This macro is applied to modules containing NetabaseSchema-derived types.
/// It finds all valid schemas within the module and generates the necessary
/// code for the netabase system.
///
/// # Example
/// ```rust
/// #[schema]
/// mod my_schemas {
///     #[derive(Serialize, Deserialize, NetabaseSchema)]
///     struct User {
///         #[key]
///         id: u64,
///         name: String,
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn schema(_args: TokenStream, input: TokenStream) -> TokenStream {
    let module = parse_macro_input!(input as ItemMod);

    match process_schema_module(&module) {
        Ok(tokens) => tokens,
        Err(error) => {
            let syn_error: syn::Error = error.into();
            syn_error.to_compile_error().into()
        }
    }
}

/// Process the NetabaseSchema derive macro
fn process_netabase_schema_derive(input: &DeriveInput) -> MacroResult<TokenStream> {
    use crate::visitors::key_finder::KeyValidator;

    let key_validator = KeyValidator::new();

    // Convert DeriveInput to Item for validation
    let item = derive_input_to_item(input.clone())?;

    // Validate that this is a valid schema
    let schema_type = SchemaType::try_from(&item)
        .map_err(|e| MacroError::new(format!("Invalid schema: {}", e)))?;

    // Extract key information
    let key_info = key_validator
        .validate_and_extract_keys(&schema_type)
        .map_err(|e| MacroError::new(format!("Key validation failed: {:?}", e)))?;

    // Generate implementation for NetabaseSchema
    let struct_name = &input.ident;
    let key_type_name = syn::Ident::new(&format!("{}Key", struct_name), struct_name.span());

    // Generate the key extraction logic based on the key type
    let key_extraction = match &key_info {
        crate::visitors::utils::KeyType::FieldKeys(field_keys) => {
            // Find the key fields for the main struct (None variant)
            if let Some(field_key_infos) = field_keys.get(&None) {
                if field_key_infos.is_empty() {
                    quote! {
                        #key_type_name("no_key_found".to_string())
                    }
                } else if field_key_infos.len() == 1 {
                    // Single key field
                    let field_key_info = &field_key_infos[0];
                    let field_name = field_key_info
                        .field
                        .ident
                        .as_ref()
                        .ok_or_else(|| MacroError::new("Key field must have a name"))?;
                    quote! {
                        #key_type_name(bincode::encode_to_vec(&self.#field_name, bincode::config::standard())
                            .unwrap_or_else(|_| vec![]))
                    }
                } else {
                    // Multiple key fields - create composite key
                    let field_names: Vec<_> = field_key_infos
                        .iter()
                        .map(|field_key_info| {
                            field_key_info
                                .field
                                .ident
                                .as_ref()
                                .ok_or_else(|| MacroError::new("Key field must have a name"))
                        })
                        .collect::<Result<Vec<_>, MacroError>>()?;

                    quote! {
                        #key_type_name({
                            let mut key_parts = Vec::new();
                            #(
                                key_parts.extend(bincode::encode_to_vec(&self.#field_names, bincode::config::standard())
                                    .unwrap_or_else(|_| vec![]));
                            )*
                            key_parts
                        })
                    }
                }
            } else {
                quote! {
                    #key_type_name(b"no_key_found".to_vec())
                }
            }
        }
        crate::visitors::utils::KeyType::SchemaKey(_closure) => {
            // For closure-based keys, we'd need to evaluate the closure
            // For now, use a placeholder
            quote! {
                #key_type_name(b"schema_key".to_vec())
            }
        }
        crate::visitors::utils::KeyType::KeyFunction(func_name) => {
            // For function-based keys, call the specified function
            let func_ident = syn::Ident::new(func_name, proc_macro2::Span::call_site());
            quote! {
                #key_type_name(bincode::encode_to_vec(&#func_ident(self), bincode::config::standard())
                    .unwrap_or_else(|_| vec![]))
            }
        }
    };

    let tokens = quote! {
        // Define a key type for this schema
        #[derive(Clone, bincode::Encode, Debug, PartialEq, Eq)]
        pub struct #key_type_name(Vec<u8>);

        // Manual implementation of Decode
        impl bincode::Decode<()> for #key_type_name {
            fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
                let inner: Vec<u8> = bincode::Decode::decode(decoder)?;
                Ok(#key_type_name(inner))
            }
        }

        // Manual implementation of BorrowDecode
        impl<'de> bincode::BorrowDecode<'de, ()> for #key_type_name {
            fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
                let inner: Vec<u8> = bincode::BorrowDecode::borrow_decode(decoder)?;
                Ok(#key_type_name(inner))
            }
        }

        // Display implementation for the key type
        impl std::fmt::Display for #key_type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        // Implementation for NetabaseSchemaKey for the generated key type
        impl netabase::NetabaseSchemaKey for #key_type_name {
        }

        // Implementation for NetabaseSchema derive
        impl netabase::NetabaseSchema for #struct_name {
            type Key = #key_type_name;
            fn key(&self) -> Self::Key {
                #key_extraction
            }
        }

        // Implementation for From<libp2p::kad::Record> for the struct
        impl From<libp2p::kad::Record> for #struct_name {
            fn from(record: libp2p::kad::Record) -> Self {
                // Deserialize from record.value using bincode
                bincode::decode_from_slice(&record.value, bincode::config::standard())
                    .expect("Failed to deserialize record")
                    .0
            }
        }

        // Implementation for Into<libp2p::kad::Record> for the struct
        impl Into<libp2p::kad::Record> for #struct_name {
            fn into(self) -> libp2p::kad::Record {
                // Serialize to record.value using bincode
                use libp2p::kad::{Record, RecordKey};
                let serialized = bincode::encode_to_vec(&self, bincode::config::standard())
                    .expect("Failed to serialize record");
                Record::new(RecordKey::new(&self.key().0), serialized)
            }
        }

        // Implementation for From<libp2p::kad::RecordKey> for the generated key type
        impl From<libp2p::kad::RecordKey> for #key_type_name {
            fn from(record_key: libp2p::kad::RecordKey) -> Self {
                #key_type_name(record_key.to_vec())
            }
        }

        // Implementation for Into<libp2p::kad::RecordKey> for the generated key type
        impl Into<libp2p::kad::RecordKey> for #key_type_name {
            fn into(self) -> libp2p::kad::RecordKey {
                libp2p::kad::RecordKey::new(&self.0)
            }
        }
    };

    Ok(tokens.into())
}

/// Process the schema module attribute
fn process_schema_module(module: &ItemMod) -> MacroResult<TokenStream> {
    let mut finder = SchemaFinder::new();
    finder.visit_item_mod(module);

    let schemas = finder.into_schemas();

    if schemas.is_empty() {
        return Err(MacroError::new(
            "Schema module must contain at least one valid schema",
        ));
    }

    // Generate the modified module with additional functions
    let module_name = &module.ident;
    let _module_vis = &module.vis;
    let module_attrs = &module.attrs;
    let schema_count = schemas.len();

    // Get the module content, or create empty content if None
    let module_content = match &module.content {
        Some((_, items)) => items.clone(),
        None => {
            return Err(MacroError::new(
                "Schema module must have content (cannot be a module declaration)",
            ));
        }
    };

    let expanded = quote! {
        #(#module_attrs)*
        pub mod #module_name {
            #(#module_content)*

            /// Get schema count for this module
            pub fn get_schema_count() -> usize {
                #schema_count
            }

            /// Register all schemas in this module
            pub fn register_schemas() {
                // TODO: Schema registration logic would go here
            }
        }
    };

    Ok(expanded.into())
}

/// Generate schema registration code for found schemas
fn generate_schema_registrations(
    schemas: &[SchemaInfo],
) -> MacroResult<Vec<proc_macro2::TokenStream>> {
    let mut registrations = Vec::new();

    for schema in schemas {
        if !schema.is_valid() {
            return Err(MacroError::new(format!(
                "Invalid schema found: {:?}",
                schema.path_string()
            )));
        }

        let schema_type = schema.schema_type.as_ref().unwrap();
        let schema_name = schema_type.identity();
        let path_string = schema.path_string();

        let registration = quote! {
            // Register schema: #schema_name at path: #path_string
            // Registration logic would go here
        };

        registrations.push(registration);
    }

    Ok(registrations)
}

/// Generate schema registry code
fn generate_schema_registry(schemas: &[SchemaInfo]) -> proc_macro2::TokenStream {
    let schema_names: Vec<String> = schemas
        .iter()
        .filter_map(|schema| schema.name().map(|n| n.to_string()))
        .collect();

    let schema_paths: Vec<String> = schemas.iter().map(|schema| schema.path_string()).collect();

    quote! {
        pub const SCHEMA_NAMES: &[&str] = &[#(#schema_names),*];
        pub const SCHEMA_PATHS: &[&str] = &[#(#schema_paths),*];

        pub fn get_schema_count() -> usize {
            SCHEMA_NAMES.len()
        }
    }
}

/// Generate key extractor functions
fn generate_key_extractors(schemas: &[SchemaInfo]) -> MacroResult<TokenStream> {
    let extractors: Vec<proc_macro2::TokenStream> = schemas
        .iter()
        .filter_map(|schema| {
            let schema_type = schema.schema_type.as_ref()?;
            let schema_name = schema_type.identity();
            let key_info = schema.schema_key.as_ref()?;

            match key_info
                .generation_type()
                .map_err(|e| MacroError::new(e.to_string()))
                .ok()?
            {
                "field_keys" => {
                    // Generate field-based key extractor
                    Some({
                        let fn_name = syn::Ident::new(
                            &format!("extract_key_for_{}", schema_name),
                            proc_macro2::Span::call_site(),
                        );
                        quote! {
                            pub fn #fn_name(item: &#schema_name) -> Vec<u8> {
                                // TODO: Implement field-based key extraction
                                vec![]
                            }
                        }
                    })
                }
                "schema_key" => {
                    // Generate closure-based key extractor
                    Some({
                        let fn_name = syn::Ident::new(
                            &format!("extract_key_for_{}", schema_name),
                            proc_macro2::Span::call_site(),
                        );
                        quote! {
                            pub fn #fn_name(item: &#schema_name) -> Vec<u8> {
                                // TODO: Implement closure-based key extraction
                                vec![]
                            }
                        }
                    })
                }
                "key_function" => {
                    // Generate function-based key extractor
                    Some({
                        let fn_name = syn::Ident::new(
                            &format!("extract_key_for_{}", schema_name),
                            proc_macro2::Span::call_site(),
                        );
                        quote! {
                            pub fn #fn_name(item: &#schema_name) -> Vec<u8> {
                                // TODO: Implement function-based key extraction
                                vec![]
                            }
                        }
                    })
                }
                _ => None,
            }
        })
        .collect();

    Ok(quote! {
        #(#extractors)*
    }
    .into())
}

/// Convert DeriveInput to Item for compatibility with validators
fn derive_input_to_item(input: DeriveInput) -> Result<syn::Item, MacroError> {
    match input.data {
        syn::Data::Struct(data_struct) => Ok(syn::Item::Struct(syn::ItemStruct {
            attrs: input.attrs,
            vis: input.vis,
            struct_token: Default::default(),
            ident: input.ident,
            generics: input.generics,
            fields: data_struct.fields,
            semi_token: data_struct.semi_token,
        })),
        syn::Data::Enum(data_enum) => Ok(syn::Item::Enum(syn::ItemEnum {
            attrs: input.attrs,
            vis: input.vis,
            enum_token: Default::default(),
            ident: input.ident,
            generics: input.generics,
            brace_token: Default::default(),
            variants: data_enum.variants,
        })),
        syn::Data::Union(_) => Err(MacroError::new(
            "NetabaseSchema cannot be derived for unions",
        )),
    }
}

/// Utility function for testing and debugging
#[cfg(test)]
pub fn test_schema_finder(module: &ItemMod) -> Vec<SchemaInfo> {
    let mut finder = SchemaFinder::new();
    finder.visit_item_mod(module);
    finder.into_schemas()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_schema_module_processing() {
        let module: ItemMod = parse_quote! {
            mod test_schemas {
                #[derive(Serialize, Deserialize, NetabaseSchema)]
                struct User {
                    #[key]
                    id: u64,
                    name: String,
                }

                #[derive(Serialize, Deserialize, NetabaseSchema)]
                enum Status {
                    Active { #[key] id: u64 },
                    Inactive,
                }
            }
        };

        let schemas = test_schema_finder(&module);
        assert_eq!(schemas.len(), 2);

        // Verify the schemas are valid
        for schema in &schemas {
            assert!(schema.is_valid());
            assert!(schema.schema_key.as_ref().unwrap().is_valid());
        }
    }

    #[test]
    fn test_empty_schema_module() {
        let module: ItemMod = parse_quote! {
            mod empty_module {
                // No schemas here
                struct RegularStruct {
                    data: String,
                }
            }
        };

        let schemas = test_schema_finder(&module);
        assert_eq!(schemas.len(), 0);
    }

    #[test]
    fn test_nested_schema_modules() {
        let module: ItemMod = parse_quote! {
            mod outer {
                mod inner {
                    #[derive(Serialize, Deserialize, NetabaseSchema)]
                    struct NestedSchema {
                        #[key]
                        id: u64,
                    }
                }
            }
        };

        let schemas = test_schema_finder(&module);
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        assert_eq!(schema.path_string(), "outer::inner::NestedSchema");
    }
}
