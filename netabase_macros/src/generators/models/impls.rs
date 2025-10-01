use syn::{ItemImpl, parse_quote};

use crate::NetabaseModelVisitor;

impl<'ast> NetabaseModelVisitor<'ast> {
    pub fn generate_netabase_schema_trait(&self) -> ItemImpl {
        let name = match self.name {
            Some(id) => id,
            None => panic!("Cannot find modl name"),
        };

        let key_field = match self.key_field {
            Some(id) => match &id.ident {
                Some(id) => id,
                None => panic!("Ident not found"),
            },
            None => panic!("Cannot find model key"),
        };

        let key_name = match &self.key_name {
            Some(k) => k,
            None => panic!("Key has no name"),
        }; // TODO: Support for Tuple Struct

        parse_quote! {
            impl NetabaseModel for #name {
                type Key = #key_name;

                fn key(&self) -> Self::Key {
                    self.#key_field.clone().into()
                }
            }
        }
    }

    pub fn generate_into_record(&self) -> ItemImpl {
        let name = self.name;
        parse_quote! {
            impl TryFrom<#name> for libp2p::kad::Record {
                type Error = netabase::errors::NetabaseError;
                fn try_from(value: #name) -> Result<Self, Self::Error> {
                    value.to_record()
                }
            }
        }
    }

    pub fn generate_from_record(&self) -> ItemImpl {
        let name = self.name;
        parse_quote! {
            impl TryFrom<libp2p::kad::Record> for #name {
                type Error = netabase::errors::NetabaseError;
                fn try_from(value: libp2p::kad::Record) -> Result<Self, Self::Error> {
                    Self::from_record(value)
                }
            }
        }
    }
}

pub mod key_impl {
    use syn::{DeriveInput, ItemImpl, parse_quote};

    pub fn generate_netabase_schema_key_trait(key_struct: &DeriveInput) -> ItemImpl {
        let name = &key_struct.ident;
        parse_quote! {
            impl NetabaseModelKey for #name {

            }
        }
    }

    pub fn generate_into_record_key(key_struct: &DeriveInput) -> ItemImpl {
        let name = &key_struct.ident;
        parse_quote! {
            impl TryFrom<#name> for libp2p::kad::RecordKey {
                type Error = netabase::errors::NetabaseError;
                fn try_from(value: #name) -> Result<Self, Self::Error> {
                    value.to_record_key()
                }
            }
        }
    }
    pub fn generate_from_record_key(key_struct: &DeriveInput) -> ItemImpl {
        let name = &key_struct.ident;
        parse_quote! {
            impl TryFrom<libp2p::kad::RecordKey> for #name {
                type Error = netabase::errors::NetabaseError;
                fn try_from(value: libp2p::kad::RecordKey) -> Result<Self, Self::Error> {
                    Self::from_record_key(value)
                }
            }
        }
    }
}
