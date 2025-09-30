use syn::{ItemEnum, Variant, parse_quote};

use crate::SchemaModuleVisitor;

pub fn generate_module_schema(module_visitor: SchemaModuleVisitor) -> (ItemEnum, ItemEnum) {
    let (schemas, keys): (Vec<Variant>, Vec<Variant>) = module_visitor
        .format_paths()
        .iter()
        .map(|((k, k_name), (v, v_name))| {
            eprintln!("Processing Pair: {:?}, {:?}", k.to_string(), v.to_string());
            (
                parse_quote! {
                    #k_name( #k )
                },
                parse_quote! {
                    #v_name( #v )
                },
            )
        })
        .unzip();
    let (schema_name, key_name) = (module_visitor.schema_name, module_visitor.schema_key_name);
    (
        parse_quote! {
            #[derive(derive_more::From, derive_more::TryInto, Clone)]
            pub enum #schema_name {
                #(#schemas),*
            }
        },
        parse_quote! {
            #[derive(derive_more::From, derive_more::TryInto, Clone)]
            pub enum #key_name {
                #(#keys),*
            }
        },
    )
}
