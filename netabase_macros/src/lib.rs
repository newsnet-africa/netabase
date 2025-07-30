use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemMod, ItemStruct, Path, parse_macro_input, visit::Visit};

use crate::visitors::schema_finder::SchemaFinder;

mod generators;
mod visitors;

#[proc_macro_attribute]
pub fn schema(module: TokenStream, module1: TokenStream) -> TokenStream {
    let module1 = parse_macro_input!(module1 as ItemStruct);
    let id = module1.ident.clone();
    // let mut finder = SchemaFinder::default();
    // finder.visit_item_mod(&module);
    // let s = finder
    //     .schemas
    //     .iter()
    //     .map(|i| Path {
    //         leading_colon: None,
    //         segments: i.path.clone(),
    //     })
    //     .collect::<Vec<Path>>();
    quote! {
        #id
    }
    .into()
}
