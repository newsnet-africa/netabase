use quote::ToTokens;
use syn::{Ident, ItemStruct, parse_quote};

use crate::{append_ident, visitors::netabase_schema_derive::NetabaseModelVisitor};

impl<'ast> NetabaseModelVisitor<'ast> {
    pub fn generate_key(&self) -> ItemStruct {
        let name = match self.name {
            Some(r) => r,
            None => panic!("Schema not found"),
        };
        let key_name = match &self.key_name {
            Some(sp) => sp,
            None => &append_ident(name, "Key"),
        };

        let key_type = if let Some(keys) = self.key_field {
            &keys.ty
        } else {
            panic!("Key type not found")
        };
        let mut key_derive = quote::quote!(
            Encode,
            Decode,
            Debug,
            Clone,
            derive_more::From,
            derive_more::Into,
            NetabaseModelKey
        );
        if let Some(keys) = self.key_derive {
            keys.tokens.clone().to_tokens(&mut key_derive);
        }
        parse_quote! {
            #[derive(#key_derive)]
            pub struct #key_name(#key_type);
        }
    }
}
