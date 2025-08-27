use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

use crate::visitors::Key;

/// Generator for creating schema-related enums from collected schema data
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
    pub fn generate_schemas_enum(&self, enum_name: &str) -> TokenStream {
        let enum_ident = syn::Ident::new(enum_name, proc_macro2::Span::call_site());

        if self.schemas.is_empty() {
            return self.generate_empty_enum(&enum_ident, "No schemas found");
        }

        let variants = self.generate_schema_variants();

        quote! {
            #[derive(NetabaseSchema, Debug, Clone)]
            #[__netabase_registery]
            pub enum #enum_ident {
                #(#variants),*
            }

        }
    }

    /// Generate an enum containing all found schema keys
    pub fn generate_schema_keys_enum(&self, enum_name: &str) -> TokenStream {
        let enum_ident = syn::Ident::new(enum_name, proc_macro2::Span::call_site());

        if self.schemas.is_empty() {
            return self.generate_empty_enum(&enum_ident, "No schema keys found");
        }

        let variants = self.generate_key_variants();

        quote! {
            #[derive(NetabaseSchemaKey, Debug, Clone)]
            pub enum #enum_ident {
                #(#variants),*
            }
        }
    }

    /// Generate both enums as a combined TokenStream
    pub fn generate_both_enums(
        &self,
        schemas_enum_name: &str,
        keys_enum_name: &str,
    ) -> TokenStream {
        let schemas_enum = self.generate_schemas_enum(schemas_enum_name);
        let keys_enum = self.generate_schema_keys_enum(keys_enum_name);

        quote! {
            #schemas_enum

            #keys_enum
        }
    }
}

impl<'a> SchemaEnumGenerator<'a> {
    fn generate_empty_enum(&self, enum_ident: &Ident, comment: &str) -> TokenStream {
        quote! {
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

    fn generate_key_variants(&self) -> Vec<TokenStream> {
        self.schemas
            .iter()
            .map(|(path, _, key_name)| {
                let variant_name = key_name;
                let mut path = path.clone();
                match path.segments.last_mut() {
                    Some(seg) => seg.ident = Key::ident(&seg.ident),
                    None => todo!(),
                }
                quote! {
                    #variant_name(#path)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_empty_schemas_enum() {
        let generator = SchemaEnumGenerator::new(&[]);
        let result = generator.generate_schemas_enum("TestEnum");

        let expected = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum TestEnum {
                // No schemas found
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_schemas_enum_generation() {
        let schemas = vec![
            (
                parse_quote!(user::User),
                parse_quote!(User),
                parse_quote!(UserKey),
            ),
            (
                parse_quote!(post::Post),
                parse_quote!(Post),
                parse_quote!(PostKey),
            ),
        ];

        let generator = SchemaEnumGenerator::new(&schemas);
        let result = generator.generate_schemas_enum("AllSchemas");

        // The result should contain both User and Post variants
        let result_string = result.to_string();
        assert!(result_string.contains("User(user :: User)"));
        assert!(result_string.contains("Post(post :: Post)"));
    }

    #[test]
    fn test_keys_enum_generation() {
        let schemas = vec![
            (
                parse_quote!(user::User),
                parse_quote!(User),
                parse_quote!(UserKey),
            ),
            (
                parse_quote!(post::Post),
                parse_quote!(Post),
                parse_quote!(PostKey),
            ),
        ];

        let generator = SchemaEnumGenerator::new(&schemas);
        let result = generator.generate_schema_keys_enum("AllKeys");

        // The result should contain both UserKey and PostKey variants
        let result_string = result.to_string();
        assert!(result_string.contains("UserKey(user :: User)"));
        assert!(result_string.contains("PostKey(post :: Post)"));
    }

    #[test]
    fn test_both_enums_generation() {
        let schemas = vec![(
            parse_quote!(user::User),
            parse_quote!(User),
            parse_quote!(UserKey),
        )];

        let generator = SchemaEnumGenerator::new(&schemas);
        let result = generator.generate_both_enums("AllSchemas", "AllKeys");

        let result_string = result.to_string();
        assert!(result_string.contains("pub enum AllSchemas"));
        assert!(result_string.contains("pub enum AllKeys"));
    }
}
