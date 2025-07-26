use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    AttrStyle, Field, Ident, Item, ItemFn, ItemMod, Meta, PathSegment, Token,
    punctuated::Punctuated, visit::Visit,
};

enum ValidationItem<'a> {
    ValidSchema(Ident),
    Invalid,
    Module(&'a ItemMod),
}

fn validate_schema<'a>(item: &'a Item) -> ValidationItem<'a> {
    let trait_name = "NetabaseSchema";
    let (attributes, item_ident) = {
        match item {
            Item::Enum(item_enum) => (item_enum.attrs.clone(), item_enum.ident.clone()),
            Item::Struct(item_struct) => (item_struct.attrs.clone(), item_struct.ident.clone()),
            Item::Mod(m) => {
                return ValidationItem::Module(m);
            }
            _ => {
                return ValidationItem::Invalid;
            }
        }
    };
    if attributes.iter().any(|att| match (&att.style, &att.meta) {
        (AttrStyle::Outer, Meta::List(meta_list)) => {
            if meta_list.path.is_ident("derive") {
                att.parse_nested_meta(|meta| {
                    if meta.path.is_ident(trait_name) {
                        Ok(())
                    } else {
                        Err(syn::Error::new(
                            item_ident.span(),
                            format!("Schema: {} should derive {trait_name}", item_ident.clone()),
                        ))
                    }
                })
                .is_ok()
            } else {
                false
            }
        }
        _ => false,
    }) {
        ValidationItem::ValidSchema(item_ident)
    } else {
        ValidationItem::Invalid
    }
}

#[derive(Default)]
pub(super) struct ValidSchemaFinder<'ast> {
    pub(super) valid_schemas: Vec<(&'ast Item, syn::Path)>,
    pub(super) current_path: Punctuated<PathSegment, Token![::]>,
}

impl<'ast> Visit<'ast> for ValidSchemaFinder<'ast> {
    fn visit_item(&mut self, i: &'ast syn::Item) {
        match validate_schema(i) {
            ValidationItem::ValidSchema(ident_name) => {
                let mut item_path = self.current_path.clone();
                item_path.push(ident_name.into());
                self.valid_schemas.push((
                    i,
                    syn::Path {
                        leading_colon: None,
                        segments: item_path,
                    },
                ))
            }
            ValidationItem::Invalid => (),
            ValidationItem::Module(item_mod) => self.visit_item_mod(item_mod),
        }
    }

    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        let mod_name = i.ident.clone();
        self.current_path.push(mod_name.into());
        syn::visit::visit_item_mod(self, i);
    }
}

#[derive(Default)]
pub(super) enum KeyGenerator<'a> {
    Function(&'a ItemFn),
    Field(&'a Field),
    #[default]
    None,
}
#[derive(Default)]
pub(super) struct SchemaValidator<'ast> {
    key: KeyGenerator<'ast>,
}

impl<'ast> SchemaValidator<'ast> {
    pub fn is_key(f: &Field) -> Option<&Meta> {
        f.attrs.iter().find_map(|att| {
            if let Meta::Path(path) = &att.meta
                && path.is_ident("NetabaseKey")
            {
                Some(&att.meta)
            } else if let Meta::List(meta_list) = &att.meta && meta_list.path.is_ident("NetabaseKey"){
               Some(&att.meta)
            } else if let Meta::NameValue(named_value) = &att.meta && named_value.path.is_ident("NetabaseKey") {
                match named_value.value {
                        syn::Expr::Closure(closure) => {
                            // TODO: How can an item without a key have one generated for it?
                        }
                    syn::Expr::Path(path)
                    syn::Expr::MethodCall(method_call)
                }
                Some(&att.meta)
            } else {
                None
            }
        })
    }
}

impl<'ast> Visit<'ast> for SchemaValidator<'ast> {
    fn visit_item(&mut self, i: &'ast syn::Item) {
        match i {
            Item::Enum(item_enum) => {}
            Item::Struct(item_struct) => {}
            _ => {}
        }
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if !i.variants.is_empty() {
            i.variants.iter().for_each(|v| {
                v.fields.iter().any(|f| {
                    if SchemaValidator::is_key(f) {
                        if
                    }
                });
            });
        };
    }
}
