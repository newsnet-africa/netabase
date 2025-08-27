use proc_macro2::TokenStream;
use quote::quote;
use syn::visit::Visit;
use syn::{Field, Meta, parse_quote};
use syn::{Fields, Ident, Item, ItemEnum, ItemMod, ItemStruct, Path, Type};

use crate::SchemaValidator;

#[derive(Default)]
pub struct SchemaCounterVisitor {
    schema_path: Option<Path>,
    schemas_skel: Vec<(Path, Ident, Ident)>,
}

impl Visit<'_> for SchemaCounterVisitor {
    fn visit_item_mod(&mut self, schema_module: &'_ syn::ItemMod) {
        let id = schema_module.ident.clone();
        let old_path = self.schema_path.clone();

        // Update the current path
        match &self.schema_path {
            Some(mut path) => {
                path.segments.push(parse_quote!(#id));
                self.schema_path = Some(path);
            }
            None => {
                self.schema_path = Some(parse_quote!(#id));
            }
        }

        if let Some((_, content)) = &schema_module.content {
            for item in content {
                match item {
                    Item::Enum(item_enum) => {
                        self.process_enum_item(item_enum);
                    }
                    Item::Mod(item_mod) => {
                        self.visit_item_mod(item_mod);
                    }
                    Item::Struct(item_struct) => {
                        self.process_struct_item(item_struct);
                    }
                    _ => {
                        // Skip other item types
                    }
                }
            }
        }

        // Restore the previous path
        self.schema_path = old_path;
    }
}

impl SchemaCounterVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_schemas(&self) -> &Vec<(Path, Ident, Ident)> {
        &self.schemas_skel
    }

    fn process_enum_item(&mut self, item_enum: &ItemEnum) {
        if let Some(schema_name) = Self::check_netabase_schema_enum(item_enum) {
            if let Some(key_name) = Self::check_netabase_schema_key_enum(item_enum) {
                let full_path = self.build_full_path(&schema_name);
                self.schemas_skel.push((full_path, schema_name, key_name));
            }
        }
    }

    fn process_struct_item(&mut self, item_struct: &ItemStruct) {
        if let Some(schema_name) = Self::check_netabase_schema_struct(item_struct) {
            if let Some(key_name) = Self::check_netabase_schema_key_struct(item_struct) {
                let full_path = self.build_full_path(&schema_name);
                self.schemas_skel.push((full_path, schema_name, key_name));
            }
        }
    }

    fn build_full_path(&self, name: &Ident) -> Path {
        match &self.schema_path {
            Some(mut path) => {
                path.segments.push(parse_quote!(#name));
                path
            }
            None => {
                parse_quote!(#name)
            }
        }
    }

    fn check_netabase_schema_enum(item_enum: &ItemEnum) -> Option<Ident> {
        item_enum.attrs.iter().find_map(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("derive")
                    && meta_list.tokens.to_string().contains("NetabaseSchema")
                {
                    Some(item_enum.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn check_netabase_schema_struct(item_struct: &ItemStruct) -> Option<Ident> {
        item_struct.attrs.iter().find_map(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("derive")
                    && meta_list.tokens.to_string().contains("NetabaseSchema")
                {
                    Some(item_struct.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn check_netabase_schema<'ast>(item: &'ast Item) -> Option<Ident> {
        match item {
            Item::Enum(item_enum) => Self::check_netabase_schema_enum(item_enum),
            Item::Struct(item_struct) => Self::check_netabase_schema_struct(item_struct),
            _ => None,
        }
    }

    fn check_netabase_schema_key_enum(item_enum: &ItemEnum) -> Option<Ident> {
        item_enum.attrs.iter().find_map(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("derive")
                    && meta_list.tokens.to_string().contains("NetabaseSchemaKey")
                {
                    Some(item_enum.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn check_netabase_schema_key_struct(item_struct: &ItemStruct) -> Option<Ident> {
        item_struct.attrs.iter().find_map(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("derive")
                    && meta_list.tokens.to_string().contains("NetabaseSchemaKey")
                {
                    Some(item_struct.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn check_netabase_schema_key<'ast>(item: &'ast Item) -> Option<Ident> {
        match item {
            Item::Enum(item_enum) => Self::check_netabase_schema_key_enum(item_enum),
            Item::Struct(item_struct) => Self::check_netabase_schema_key_struct(item_struct),
            _ => None,
        }
    }

    /// Generate an enum containing all found schemas
    pub fn generate_schemas_enum(&self, enum_name: &str) -> TokenStream {
        let enum_ident = syn::Ident::new(enum_name, proc_macro2::Span::call_site());

        if self.schemas_skel.is_empty() {
            return quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub enum #enum_ident {
                    // No schemas found
                }
            };
        }

        let variants: Vec<_> = self
            .schemas_skel
            .iter()
            .map(|(path, schema_name, _)| {
                let variant_name = schema_name;
                quote! {
                    #variant_name(#path)
                }
            })
            .collect();

        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum #enum_ident {
                #(#variants),*
            }
        }
    }

    /// Generate an enum containing all found schema keys
    pub fn generate_schema_keys_enum(&self, enum_name: &str) -> TokenStream {
        let enum_ident = syn::Ident::new(enum_name, proc_macro2::Span::call_site());

        if self.schemas_skel.is_empty() {
            return quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub enum #enum_ident {
                    // No schema keys found
                }
            };
        }

        let variants: Vec<_> = self
            .schemas_skel
            .iter()
            .map(|(path, _, key_name)| {
                let variant_name = key_name;
                quote! {
                    #variant_name(#path)
                }
            })
            .collect();

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Clear collected schemas (useful for reusing the visitor)
    pub fn clear(&mut self) {
        self.schemas_skel.clear();
        self.schema_path = None;
    }
}
