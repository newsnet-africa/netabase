use proc_macro2::Span;

use syn::{
    Ident, Item, ItemStruct, PathSegment, Token,
    punctuated::{Pair, Punctuated},
    visit::Visit,
};

pub struct NativeModel<'ast> {
    pub model: &'ast ItemStruct,
    pub path: Vec<Pair<PathSegment, Token![::]>>,
}

impl<'ast> NativeModel<'ast> {
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
        Self::create_path(&self.path, &self.model.ident)
    }

    pub fn key_name(&self) -> Ident {
        let key_name = format!("{}Key", self.model.ident);
        Ident::new(&key_name, self.model.ident.span())
    }

    pub fn key_path(&self) -> Punctuated<PathSegment, syn::token::PathSep> {
        Self::create_path(&self.path, &self.key_name())
    }
}

#[derive(Default)]
pub struct SchemaVisitor<'ast> {
    pub items: Vec<NativeModel<'ast>>,
    current_path: Vec<Pair<PathSegment, Token![::]>>,
}

impl<'ast> Visit<'ast> for SchemaVisitor<'ast> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if let Some((_, content)) = &i.content {
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
                            self.items.push(NativeModel {
                                model: item_struct,
                                path: self.current_path.clone(),
                            });
                        }
                    }
                    Item::Mod(item_mod) => {
                        // push path, recurse, then pop
                        self.current_path.push(Pair::new(
                            PathSegment {
                                ident: item_mod.ident.clone(),
                                arguments: syn::PathArguments::None,
                            },
                            Some(Token![::](Span::call_site())),
                        ));
                        // Recurse with the updated path
                        self.visit_item_mod(item_mod);
                        self.current_path.pop();
                        // continue; (models found in child modules are already pushed)
                    }
                    _ => {}
                }
            }
        }
    }
}
