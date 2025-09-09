use crate::generators::GenerationError;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Fields, Ident, ItemEnum, Meta, Path, Type, Variant, parse_quote};

/// Generator for creating schema-related enums from collected schema data
#[derive(Default, Debug)]
pub struct SchemaEnumGenerator<'a> {
    schemas: &'a [(Path, Ident, Ident)],
}

impl<'a> SchemaEnumGenerator<'a> {
    /// Create a new generator with schema data
    pub fn new(schemas: &'a [(Path, Ident, Ident)]) -> Self {
        Self { schemas }
    }
}

impl<'a> SchemaEnumGenerator<'a> {
    /// Generate an enum containing all found schemas
    pub fn generate_schemas_enum(&self, enum_ident: &Ident) -> Result<ItemEnum, GenerationError> {
        if self.schemas.is_empty() {
            return Ok(self.generate_empty_enum(enum_ident, "No schemas found"));
        }

        let mut schema_ident = enum_ident.to_string();
        schema_ident.push_str("Schema");
        let enum_ident = Ident::new(&schema_ident, proc_macro2::Span::call_site());
        let variants = self.generate_schema_variants();

        Ok(parse_quote! {
            #[derive(Debug, Clone, ::macro_exports::__netabase_bincode::Encode, ::macro_exports::__netabase_bincode::Decode)]
            pub enum #enum_ident {
                #(#variants),*
            }

        })
    }

    pub fn generate_schema_keys_enum_from_attr(
        attr: &Attribute,
        enum_name: &Ident,
    ) -> Result<ItemEnum, GenerationError> {
        let arms: Vec<Variant> = {
            if let Meta::List(list_of_var) = &attr.meta {
                // Parse variant names and create new variants that hold key types
                let mut variants = Vec::new();
                let mut current_tokens = TokenStream::new();

                for token in list_of_var.tokens.clone() {
                    match &token {
                        proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',' => {
                            // Parse the accumulated tokens as a variant and extract key name
                            if !current_tokens.is_empty() {
                                if let Ok(parsed_variant) =
                                    syn::parse2::<Variant>(current_tokens.clone())
                                {
                                    let key_name = &parsed_variant.ident;
                                    // Extract the schema path and convert it to key path
                                    if let Fields::Unnamed(fields) = &parsed_variant.fields
                                        && let Some(field) = fields.unnamed.first()
                                        && let Type::Path(type_path) = &field.ty
                                    {
                                        let mut key_path = type_path.path.clone();
                                        // Replace the last segment (schema name) with key name
                                        if let Some(last_segment) = key_path.segments.last_mut() {
                                            last_segment.ident = key_name.clone();
                                        }
                                        let new_variant = parse_quote! {
                                            #key_name(#key_path)
                                        };
                                        variants.push(new_variant);
                                    }
                                }
                                current_tokens = TokenStream::new();
                            }
                        }
                        _ => {
                            current_tokens.extend(std::iter::once(token));
                        }
                    }
                }

                // Handle the last variant (after the last comma or if there's only one)
                if !current_tokens.is_empty()
                    && let Ok(parsed_variant) = syn::parse2::<Variant>(current_tokens)
                {
                    let key_name = &parsed_variant.ident;
                    // Extract the schema path and convert it to key path
                    if let Fields::Unnamed(fields) = &parsed_variant.fields
                        && let Some(field) = fields.unnamed.first()
                        && let Type::Path(type_path) = &field.ty
                    {
                        let mut key_path = type_path.path.clone();
                        // Replace the last segment (schema name) with key name
                        if let Some(last_segment) = key_path.segments.last_mut() {
                            last_segment.ident = key_name.clone();
                        }
                        let new_variant = parse_quote! {
                            #key_name(#key_path)
                        };
                        variants.push(new_variant);
                    }
                }

                variants
            } else {
                vec![]
            }
        };

        Ok(parse_quote! {
            #[derive(NetabaseSchemaKey, Debug, Clone, ::macro_exports::__netabase_bincode::Encode, ::macro_exports::__netabase_bincode::Decode)]
            pub enum #enum_name {
                #(#arms),*
            }
        })
    }

    /// Generate both enums as a combined TokenStream
    pub fn generate_both_enums(
        &self,
        registry_enum_name: &Ident,
    ) -> Result<TokenStream, GenerationError> {
        let schemas_enum = self.generate_schemas_enum(registry_enum_name)?;
        let keys_enum = self.generate_keys_enum(registry_enum_name)?;

        let mut schema_name = registry_enum_name.to_string();
        schema_name.push_str("Schema");
        let schema_name = Ident::new(&schema_name, proc_macro2::Span::call_site());
        let mut schema_key_name = registry_enum_name.to_string();
        schema_key_name.push_str("Key");
        let schema_key_name = Ident::new(&schema_key_name, proc_macro2::Span::call_site());

        Ok(quote! {
            #schemas_enum

            #keys_enum

            pub struct #registry_enum_name {
                _schema: #schema_name,
                _keys: #schema_key_name,
            }
        })
    }

    /// Generate an enum containing all schema keys
    pub fn generate_keys_enum(&self, enum_ident: &Ident) -> Result<ItemEnum, GenerationError> {
        if self.schemas.is_empty() {
            let mut key_ident = enum_ident.to_string();
            key_ident.push_str("Key");
            let key_enum_ident = Ident::new(&key_ident, proc_macro2::Span::call_site());
            return Ok(self.generate_empty_enum(&key_enum_ident, "No schema keys found"));
        }

        let mut key_ident = enum_ident.to_string();
        key_ident.push_str("Key");
        let enum_ident = Ident::new(&key_ident, proc_macro2::Span::call_site());
        let _variants = self.generate_key_variants_for_enum();

        Ok(parse_quote! {
            #[derive(Debug, Clone, ::macro_exports::__netabase_bincode::Encode, ::macro_exports::__netabase_bincode::Decode)]
            pub enum #enum_ident {}
        })
    }

    fn generate_key_variants_for_enum(&self) -> Vec<TokenStream> {
        self.schemas
            .iter()
            .map(|(_, schema_name, _)| {
                let variant_name = format!("{}Key", schema_name);
                let variant_ident = Ident::new(&variant_name, proc_macro2::Span::call_site());
                // Use String as the key type since that's what most keys are
                quote! {
                    #variant_ident(String)
                }
            })
            .collect()
    }
}

impl<'a> SchemaEnumGenerator<'a> {
    fn generate_empty_enum(&self, enum_ident: &Ident, _comment: &str) -> ItemEnum {
        parse_quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_ident {
                // #comment
            }
        }
    }

    fn generate_schema_variants(&self) -> Vec<TokenStream> {
        self.schemas
            .iter()
            .map(|(path, schema_name, _)| {
                let variant_name = schema_name;
                quote! {
                    #variant_name(#path)
                }
            })
            .collect()
    }

    fn generate_key_variants(&self) -> Vec<Variant> {
        self.schemas
            .iter()
            .map(|(path, _, key_name)| {
                let variant_name = key_name;
                parse_quote! {
                    #variant_name(#path)
                }
            })
            .collect()
    }
}
