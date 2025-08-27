use proc_macro::{Diagnostic, Span};
use syn::visit::Visit;
use syn::{Ident, Item, ItemEnum, ItemStruct, Path};
use syn::{Meta, parse_quote};

use crate::visitors::Key;

#[derive(Default, Debug)]
pub struct SchemaCounterVisitor {
    schema_path: Option<Path>,
    schemas_skel: Vec<(Path, Ident, Ident)>,
}

impl Visit<'_> for SchemaCounterVisitor {
    fn visit_item_mod(&mut self, schema_module: &'_ syn::ItemMod) {
        let id = schema_module.ident.clone();
        let old_path = self.schema_path.clone();

        // Update the current path
        match self.schema_path.clone() {
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

    pub fn clear(&mut self) {
        self.schemas_skel.clear();
        self.schema_path = None;
    }

    fn process_enum_item(&mut self, item_enum: &ItemEnum) {
        if let Some(schema_name) = Self::check_netabase_schema_enum(item_enum) {
            let full_path = self.build_full_path(&schema_name);
            self.schemas_skel
                .push((full_path, schema_name.clone(), Key::ident(&schema_name)));
        }
    }

    fn process_struct_item(&mut self, item_struct: &ItemStruct) {
        if let Some(schema_name) = Self::check_netabase_schema_struct(item_struct) {
            let full_path = self.build_full_path(&schema_name);
            self.schemas_skel
                .push((full_path, schema_name.clone(), Key::ident(&schema_name)));
        }
    }

    fn build_full_path(&self, name: &Ident) -> Path {
        match self.schema_path.clone() {
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

    fn check_netabase_schema_key_enum(item_enum: &ItemEnum) -> Option<Ident> {
        let contains_attribute = item_enum
            .attrs
            .iter()
            .any(|a| a.path().is_ident("__netabase_schema_key"));
        item_enum.attrs.iter().find_map(|attr| {
            if let Meta::List(meta_list) = &attr.meta
                && contains_attribute
            {
                if meta_list.path.is_ident("derive") {
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
        let contains_attribute = item_struct
            .attrs
            .iter()
            .any(|a| a.path().is_ident("__netabase_schema_key"));
        item_struct.attrs.iter().find_map(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("derive") && contains_attribute {
                    Some(item_struct.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
