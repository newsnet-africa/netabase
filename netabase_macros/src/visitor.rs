use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    DataStruct,
    Field,
    Ident,
    Item,
    ItemEnum,
    ItemStruct,
    PathSegment,
    Token, // CHANGED: add ItemEnum
    punctuated::{Pair, Punctuated},
    visit::Visit,
};

pub struct NativeModel<'ast> {
    pub model: &'ast ItemStruct,
    pub key: &'ast ItemEnum, // CHANGED: key is an enum, not ItemStruct
    pub path: Vec<Pair<PathSegment, Token![::]>>,
}

impl<'ast> NativeModel<'ast> {
    // CHANGED: make helper accept any Ident reference so it works for structs or enums
    fn create_path(
        current_path: &Vec<Pair<PathSegment, syn::token::PathSep>>,
        item_ident: &Ident,
    ) -> Punctuated<PathSegment, syn::token::PathSep> {
        let mut final_path = current_path.clone();
        final_path.push(Pair::new(
            PathSegment {
                ident: item_ident.clone(),
                arguments: syn::PathArguments::None,
            },
            None,
        ));
        Punctuated::from_iter(final_path)
    }

    pub fn model_path(&self) -> Punctuated<PathSegment, syn::token::PathSep> {
        Self::create_path(&self.path, &self.model.ident) // unchanged except uses ident
    }
    pub fn key_path(&self) -> Punctuated<PathSegment, syn::token::PathSep> {
        Self::create_path(&self.path, &self.key.ident) // CHANGED: use enum.ident
    }
}

#[derive(Default)]
pub struct SchemaVisitor<'ast> {
    pub items: Vec<NativeModel<'ast>>,
}

impl<'ast> Visit<'ast> for SchemaVisitor<'ast> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if let Some((_, content)) = &i.content {
            // state per-module, persisted across sibling items
            let mut current_path: Vec<Pair<PathSegment, Token![::]>> = vec![];
            let mut model: Option<&'ast ItemStruct> = None;
            let mut key: Option<&'ast ItemEnum> = None;

            for item in content {
                match item {
                    Item::Struct(item_struct) => {
                        if item_struct
                            .attrs
                            .iter()
                            .any(|att| att.path().is_ident("native_db"))
                            && item_struct
                                .attrs
                                .iter()
                                .any(|att| att.path().is_ident("native_model"))
                        {
                            model = Some(item_struct);
                        }
                        if let (Some(m), Some(k)) = (model, key) {
                            self.items.push(NativeModel {
                                model: m,
                                key: k,
                                path: current_path.clone(),
                            });
                            // clear model/key if you expect one-to-one matches per-file
                            model = None;
                            key = None;
                        }
                    }
                    Item::Enum(item_enum) => {
                        if let Some(m) = model {
                            let mname = m.ident.to_string();
                            let ename = item_enum.ident.to_string();
                            if ename.contains(&mname) && ename.contains("Key") {
                                key = Some(item_enum);
                            }
                        }
                        if let (Some(m), Some(k)) = (model, key) {
                            self.items.push(NativeModel {
                                model: m,
                                key: k,
                                path: current_path.clone(),
                            });
                            model = None;
                            key = None;
                        }
                    }
                    Item::Mod(item_mod) => {
                        // push path, recurse, then pop
                        current_path.push(Pair::new(
                            PathSegment {
                                ident: item_mod.ident.clone(),
                                arguments: syn::PathArguments::None,
                            },
                            Some(Token![::](Span::call_site())),
                        ));
                        // recurse with the updated path by temporarily swapping state onto self
                        let saved_items_len = self.items.len();
                        // Recurse: visitor will push models found; it will not have our current_path unless you store it on self
                        self.visit_item_mod(item_mod);
                        current_path.pop();
                        // continue; (models found in child modules are already pushed)
                    }
                    _ => {}
                }
            }
        }
    }
}
