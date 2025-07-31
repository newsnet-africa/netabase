#![feature(extend_one)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, visit::Visit, DeriveInput, ItemMod};

use crate::visitors::{
    key_finder::KeyValidator,
    schema_finder::{SchemaFinder, SchemaType},
    schema_validator::SchemaValidator,
    utils::{FieldKeyInfo, KeyType, SchemaInfo},
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
    let schema_validator = SchemaValidator::new();
    let key_validator = KeyValidator::new();

    // Convert DeriveInput to Item for validation
    let item = derive_input_to_item(input.clone())?;

    // Validate that this is a valid schema
    let schema_type = schema_validator
        .validate_schema_item(&item)
        .map_err(|e| MacroError::new(e.message))?;

    // Validate key configuration
    let key_type = key_validator
        .validate_and_extract_keys(&schema_type)
        .map_err(|e| MacroError::new(e.message))?;

    // Generate all necessary code components
    let key_struct = generate_complete_key_struct(&schema_type);
    let record_conversions = generate_all_record_conversions(&schema_type, &key_type);
    let trait_impls = generate_all_trait_impls(&schema_type, &key_type);

    // Combine all generated code
    let expanded = quote! {
        #key_struct
        #record_conversions
        #trait_impls
    };

    Ok(expanded.into())
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
fn generate_key_extractors(schemas: &[SchemaInfo]) -> proc_macro2::TokenStream {
    let extractors: Vec<proc_macro2::TokenStream> = schemas
        .iter()
        .filter_map(|schema| {
            let schema_type = schema.schema_type.as_ref()?;
            let schema_name = schema_type.identity();
            let key_info = schema.schema_key.as_ref()?;

            match key_info.generation_type()? {
                KeyType::FieldKeys(_field_map) => {
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
                KeyType::SchemaKey(_closure) => {
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
                KeyType::KeyFunction(_func_name) => {
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
            }
        })
        .collect();

    quote! {
        #(#extractors)*
    }
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
