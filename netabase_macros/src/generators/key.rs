use syn::{Ident, ItemStruct, parse_quote};

use crate::visitors::netabase_schema_derive::NetabaseSchemaVisitor;

pub fn generate_key(schema_visitor: NetabaseSchemaVisitor) -> ItemStruct {
    let mut key_name = if let Some(name) = schema_visitor.name {
        name.to_string()
    } else {
        panic!("Schema not found")
    };
    key_name.push_str("Key");
    let key_name = Ident::new(&key_name, proc_macro2::Span::call_site());
    let key_type = if let Some(keys) = schema_visitor.keys {
        &keys.ty
    } else {
        panic!("Key type not found")
    };
    let key_derive = if let Some(keys) = schema_visitor.key_derive {
        keys.tokens.clone()
    } else {
        quote::quote!(Encode, Decode)
    };
    // panic!("{:?}", key_derive.to_token_stream().to_string());
    parse_quote! {
        #[derive(#key_derive)]
        pub struct #key_name(#key_type);
    }
}
