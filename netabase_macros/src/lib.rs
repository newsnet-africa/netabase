use proc_macro::TokenStream;
// use proc_macro::quote;
use quote::quote;
use syn::{AttrStyle, Item, ItemMod, Meta, MetaList};

fn validate_schema(item: &Item) -> bool {
    let trait_name = "Clone";
    let mut item_ident = None;
    let attributes = {
        match item {
            Item::Enum(item_enum) => {
                item_ident = Some(item_enum.ident.clone());
                item_enum.attrs.clone()
            }
            Item::Struct(item_struct) => {
                item_ident = Some(item_struct.ident.clone());
                item_struct.attrs.clone()
            }
            _ => {
                vec![]
            }
        }
    };
    attributes.iter().any(|att| match (&att.style, &att.meta) {
        (AttrStyle::Outer, Meta::List(meta_list)) => {
            let mut correct_trait = false;
            if meta_list.path.is_ident("derive") {
                att.parse_nested_meta(|meta| {
                    if meta.path.is_ident(trait_name) {
                        correct_trait = true;
                        Ok(())
                    } else {
                        panic!(
                            "Schema: {} should derive {trait_name}",
                            item_ident.clone().unwrap()
                        );
                        Ok(())
                    }
                })
                .is_ok()
            } else {
                false
            }
        }
        _ => false,
    })
}

fn valid_items_in_module(item_mod: ItemMod) -> Vec<Item> {
    if let Some(l) = item_mod.content.map(|(_, items)| {
        items
            .iter()
            .filter(|val| validate_schema(val))
            .cloned()
            .collect::<Vec<Item>>()
    }) {
        l
    } else {
        vec![]
    }
}

#[proc_macro_attribute]
pub fn schemas(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let schema_module = syn::parse_macro_input!(item as ItemMod);
    let valid_items = valid_items_in_module(schema_module)
        .iter()
        .map(|i| match i {
            Item::Enum(item_enum) => {
                (item_enum.ident, item_enum.generics.params) // TODO: Generics
            }
            Item::Struct(item_struct) => {}
            _ => {
                vec![]
            }
        });
    quote! {
        pub enum NetabaseSchema {
            #()
        }
    }
    .into()
}
