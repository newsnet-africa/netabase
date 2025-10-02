use syn::{DeriveInput, ItemImpl, parse_quote};

pub fn generate_enum_into_record(enum_def: &DeriveInput) -> ItemImpl {
    let name = &enum_def.ident;
    parse_quote! {
        #[cfg(feature = "libp2p")]
        impl TryFrom<#name> for libp2p::kad::Record {
            type Error = netabase_store::errors::NetabaseError;
            fn try_from(value: #name) -> Result<Self, Self::Error> {
                value.to_record()
            }
        }
    }
}

pub fn generate_enum_from_record(enum_def: &DeriveInput) -> ItemImpl {
    let name = &enum_def.ident;
    parse_quote! {
        #[cfg(feature = "libp2p")]
        impl TryFrom<libp2p::kad::Record> for #name {
            type Error = netabase_store::errors::NetabaseError;
            fn try_from(value: libp2p::kad::Record) -> Result<Self, Self::Error> {
                Self::from_record(value)
            }
        }
    }
}
