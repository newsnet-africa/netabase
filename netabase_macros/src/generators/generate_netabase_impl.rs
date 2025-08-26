use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemImpl, parse_quote};

use crate::{
    SchemaValidator,
    generate_netabase_impl::netabase_schema_key::generate_netabase_key,
    visitors::{Key, validation_error::VisitError},
};

pub fn generate_netabase_macro(input: SchemaValidator) -> TokenStream {
    let impl_item = generate_netabase_impl(&input).expect("Fix later i guess?");
    let (key, key_impl) = generate_netabase_key(
        input.key().expect("Another fix later"),
        input.ident().expect("Fix later"),
    )
    .expect("fix later");

    quote! {
        #[macro_use]
        extern crate bincode;
        #key
        #key_impl
        #impl_item
    }
}

pub fn generate_netabase_impl(input: &SchemaValidator) -> Result<ItemImpl, VisitError> {
    let ident = input.ident()?;

    let key_ident = Key::ident(ident);
    Ok(parse_quote! {
        impl NetabaseSchema for #ident {
            type Key = #key_ident;
        }
    })
}

pub mod netabase_schema_key {
    use std::str::FromStr;

    use proc_macro2::Span;
    use quote::ToTokens;
    use syn::{
        Ident, ImplItem, ImplItemFn, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ReturnType,
        Type, Variant, parse_quote,
    };

    use crate::visitors::{
        Key,
        validation_error::{KeyError, OuterKeyError, VisitError},
    };

    pub enum KeyItemType {
        StructKey(ItemStruct),
        EnumKey(ItemEnum),
    }

    impl ToTokens for KeyItemType {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                KeyItemType::StructKey(item_struct) => {
                    item_struct.to_tokens(tokens);
                }
                KeyItemType::EnumKey(item_enum) => {
                    item_enum.to_tokens(tokens);
                }
            }
        }
    }

    pub fn generate_netabase_key(
        key: &Key,
        schema_name: &Ident,
    ) -> Result<(KeyItemType, ItemImpl), VisitError> {
        let name = Key::ident(schema_name);
        match key {
            Key::Outer { sig } => {
                if let ReturnType::Type(_, boxed_type) = &sig.output {
                    let type_name = *boxed_type.clone();
                    let key_gen = generate_key_impl(&key, &name);
                    Ok((
                        KeyItemType::StructKey(parse_quote!(
                            #[derive(Encode, Decode, Clone)]
                            pub struct #name(#type_name);
                        )),
                        key_gen,
                    ))
                } else {
                    Err(VisitError::KeyError(KeyError::OuterKeyError(
                        OuterKeyError::ReturnTypeNotFound,
                    )))
                }
            }
            Key::StructInner { field, .. } => {
                let type_name = &field.ty;
                let key_gen = generate_key_impl(&key, &name);
                Ok((
                    KeyItemType::StructKey(parse_quote!(
                        #[derive(Encode, Decode, Clone)]
                        pub struct #name(#type_name);
                    )),
                    key_gen,
                ))
            }
            Key::EnumInner { variant_fields } => {
                let variant_iter = variant_fields.iter().map(|(v, f)| -> Variant {
                    let variant_ident = &v.ident;
                    let type_name = &f.0.ty;
                    parse_quote!(
                        #variant_ident(#type_name)
                    )
                });
                let key_gen = generate_key_impl(&key, &name);

                Ok((
                    KeyItemType::EnumKey(parse_quote!(
                        #[derive(Encode, Decode, Clone)]
                        pub enum #name {
                            #(#variant_iter,)*
                        }
                    )),
                    key_gen,
                ))
            }
        }
    }
    pub fn generate_key_impl(key: &Key, key_ident: &Ident) -> ItemImpl {
        parse_quote!(
            impl netabase::NetabaseSchemaKey for #key_ident {
            }
        )
    }

    pub fn generate_key_getter(
        key_item: &KeyItemType,
        key: &Key,
    ) -> Result<ImplItemFn, VisitError> {
        match (key_item, key) {
            (KeyItemType::StructKey(item_struct), Key::Outer { sig }) => {
                let name = &item_struct.ident;
                let fn_call = &sig.ident;
                Ok(parse_quote!(
                    fn key(&self) -> #name {
                        #name(self.#fn_call())
                    }
                ))
            }
            (
                KeyItemType::StructKey(item_struct),
                Key::StructInner {
                    field,
                    tuple_number,
                },
            ) => {
                let name = &item_struct.ident;
                let field_name = {
                    match &field.ident {
                        Some(ident) => ident,
                        None => {
                            let tuple = format!("{}", tuple_number.unwrap());
                            &Ident::new(&tuple, Span::call_site())
                        }
                    }
                };
                Ok(parse_quote! {
                    fn key(&self) -> #name {
                        #name(self.#field_name.clone())
                    }:
                })
            }
            (KeyItemType::EnumKey(item_enum), Key::EnumInner { variant_fields }) => {
                //TODO: Match against itself
                let name = &item_enum.ident;
                let fn_call = &sig.ident;
                Ok(parse_quote!(
                    fn key(&self) -> #name {
                        #name(self.#fn_call())
                    }
                ))
            }
        }
    }
}
