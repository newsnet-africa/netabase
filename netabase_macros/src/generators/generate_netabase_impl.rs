use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Ident, ImplItemFn, ItemImpl, parse_quote};

use crate::{
    SchemaValidator,
    generate_netabase_impl::netabase_schema_key::{generate_key_getter, generate_netabase_key},
    visitors::{Key, validation_error::VisitError},
};

pub fn generate_netabase_macro(input: SchemaValidator) -> TokenStream {
    let key_item = generate_netabase_key(
        input.key().expect("Another fix later"),
        input
            .ident()
            .expect("Fix later: generate netabase macro fn"),
    )
    .unwrap_or_else(|e| match e {
        VisitError::RegistryNotSchema => netabase_schema_key::KeyItemType::Registery,
        VisitError::KeyError(key_error) => todo!(),
        VisitError::ParseError(error) => todo!(),
        VisitError::InvalidSchemaType => todo!(),
    });
    let key = input.key().expect("Fix later: key gen");
    let k_fun = generate_key_getter(&key_item, key).expect("Fix later: key fun");

    let impl_item = generate_netabase_impl(&input, k_fun).expect("Fix later i guess?");
    quote! {
        #key_item
        #impl_item
    }
}

pub fn generate_netabase_impl(
    input: &SchemaValidator,
    key: ImplItemFn,
) -> Result<ItemImpl, VisitError> {
    let ident = input.ident()?;

    let key_ident = Key::ident(ident);
    Ok(parse_quote! {
        impl netabase::netabase_trait::NetabaseSchema for #ident {
            type Key = #key_ident;
            #key
        }
    })
}

pub mod netabase_schema_key {
    use std::str::FromStr;

    use proc_macro2::Span;
    use quote::ToTokens;
    use syn::{
        Arm, Fields, Ident, ImplItemFn, ItemEnum, ItemImpl, ItemStruct, ReturnType, Variant,
        parse_quote,
    };

    use crate::visitors::{
        Key,
        validation_error::{KeyError, OuterKeyError, VisitError},
    };

    pub enum KeyItemType {
        Registery,
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
                KeyItemType::Registery => {}
            }
        }
    }

    pub fn generate_netabase_key(
        key: &Key,
        schema_name: &Ident,
    ) -> Result<KeyItemType, VisitError> {
        let name = Key::ident(schema_name);
        match key {
            Key::Outer { sig } => {
                if let ReturnType::Type(_, boxed_type) = &sig.output {
                    let type_name = *boxed_type.clone();
                    Ok(KeyItemType::StructKey(parse_quote!(
                        #[derive(NetabaseSchemaKey, Debug, Encode, Decode, Clone)]
                        pub struct #name(#type_name);
                    )))
                } else {
                    Err(VisitError::KeyError(KeyError::OuterKeyError(
                        OuterKeyError::ReturnTypeNotFound,
                    )))
                }
            }
            Key::StructInner { field, .. } => {
                let type_name = &field.ty;
                Ok(KeyItemType::StructKey(parse_quote!(
                    #[derive(NetabaseSchemaKey, Debug, Encode, Decode, Clone)]
                    pub struct #name(#type_name);
                )))
            }
            Key::EnumInner { variant_fields } => {
                let variant_iter = variant_fields.iter().map(|(v, f)| -> Variant {
                    let variant_ident = &v.ident;
                    let type_name = &f.ty;
                    parse_quote!(
                        #variant_ident(#type_name)
                    )
                });

                Ok(KeyItemType::EnumKey(parse_quote!(
                    #[derive(NetabaseSchemaKey, Debug, Encode, Decode, Clone)]
                    pub enum #name {
                        #(#variant_iter,)*
                    }
                )))
            }
            Key::Registry => Err(VisitError::RegistryNotSchema),
        }
    }
    pub fn generate_key_impl(key_ident: &Ident) -> ItemImpl {
        parse_quote!(
            impl netabase::netabase_trait::NetabaseSchemaKey for #key_ident {
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
            (KeyItemType::StructKey(item_struct), Key::StructInner { field }) => {
                let name = &item_struct.ident;
                let field_name = {
                    match &field.ident {
                        Some(ident) => ident,
                        None => &Ident::new("0", Span::call_site()),
                    }
                };
                Ok(parse_quote! {
                    fn key(&self) -> #name {
                        #name(self.#field_name.clone())
                    }
                })
            }
            (KeyItemType::EnumKey(item_enum), Key::EnumInner { variant_fields }) => {
                //TODO: Match against itself
                let name = &item_enum.ident;
                let enumkey_variantfields =
                    item_enum.variants.iter().zip(variant_fields.iter()).map(
                        |(key_variants, (v, f))| {
                            let var_pattern: Arm = {
                                let variant_name = &v.ident;
                                let res_variant_name = &key_variants.ident;
                                match (&v.fields, &f.ident) {
                                    (Fields::Named(fields_named), Some(field_name)) => {
                                        parse_quote! {
                                            Self::#variant_name { #field_name, .. } => #name::#res_variant_name(#field_name.clone())
                                        }
                                    },
                                    (Fields::Unnamed(fields_unnamed), None) => {
                                        parse_quote! {
                                            Self::#variant_name(first, .. ) => #name::#res_variant_name(first.clone())
                                        }
                                    },
                                    _ => todo!()
                                }

                            };
                            var_pattern
                        },
                    );
                Ok(parse_quote!(
                    fn key(&self) -> #name {
                        match self {
                            #(#enumkey_variantfields,)*
                        }
                    }
                ))
            }
            (KeyItemType::StructKey(item_struct), Key::EnumInner { variant_fields }) => todo!(),
            (KeyItemType::EnumKey(item_enum), Key::Outer { sig }) => todo!(),
            (KeyItemType::EnumKey(item_enum), Key::StructInner { field }) => todo!(),
            (KeyItemType::Registery, Key::Registry) => todo!(),
            (_, _) => todo!(),
        }
    }
}
