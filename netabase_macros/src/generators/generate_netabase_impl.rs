use proc_macro2::Span;
use syn::{Ident, ItemImpl, parse_quote};

use crate::{SchemaValidator, visitors::validation_error::VisitError};

pub fn generate_netabase_impl(input: SchemaValidator) -> Result<ItemImpl, VisitError> {
    let ident = input.ident()?;
    let mut key = {
        let mut k = input.ident()?.to_string();
        k.push_str("Key");
        Ident::new(&k, Span::call_site())
    };

    let gen_key = generate

    Ok(parse_quote! {
        impl NetabaseSchema for #ident {
            type Key = #key;
        }
    })
}

pub mod netabase_schema_key {
    use proc_macro::{Ident, token_stream};
    use syn::{Item, ReturnType, parse_quote};

    use crate::visitors::{Key, validation_error::VisitError};

    pub fn generate_netabase_key(key: &Key, schema_name: Ident) -> Result<Item, VisitError> {
        match key {
            Key::Outer { sig } => {
                if let ReturnType::Type(_, boxed_type) = &sig.output {
                    let mut name = schema_name.to_string();
                    name.push_str("Key");
                    panic!("{name:?}");
                    Ok(parse_quote!(
                        pub struct #name;
                    ))
                } else {
                    Ok(parse_quote!())
                }
            }
            Key::StructInner { field } => todo!(),
            Key::EnumInner { variant_fields } => todo!(),
        }
    }
}
