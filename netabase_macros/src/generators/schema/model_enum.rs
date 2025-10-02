use syn::{ItemEnum, ItemImpl, Variant, parse_quote};

use crate::{SchemaModuleVisitor, append_ident};

pub fn generate_module_schema(
    module_visitor: SchemaModuleVisitor,
) -> (ItemEnum, ItemEnum, Vec<ItemImpl>) {
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

    let schema_enum = parse_quote! {
        #[derive(derive_more::From, derive_more::TryInto, Clone, Encode, Decode, Debug, strum::EnumDiscriminants)]
        #[strum_discriminants(derive(strum::EnumIter, strum::AsRefStr, Hash))]
        pub enum #schema_name {
            #(#schemas),*
        }
    };

    let key_enum = parse_quote! {
        #[derive(derive_more::From, derive_more::TryInto, Clone, Encode, Decode, Debug)]
        pub enum #key_name {
            #(#keys),*
        }
    };

    // Generate implementations for the schema enum
    let mut impls = vec![];

    // NetabaseSchema trait impl
    let discriminant_ident = append_ident(&schema_name, "Discriminants");

    impls.push(parse_quote! {
        impl netabase::traits::NetabaseSchema for #schema_name {
            type SchemaDiscriminants = #discriminant_ident;
        }
    });

    // TryFrom<SchemaEnum> for Record
    impls.push(parse_quote! {
        impl TryFrom<#schema_name> for libp2p::kad::Record {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: #schema_name) -> Result<Self, Self::Error> {
                <#schema_name as netabase::traits::NetabaseSchema>::to_record(&value)
            }
        }
    });

    // TryFrom<Record> for SchemaEnum
    impls.push(parse_quote! {
        impl TryFrom<libp2p::kad::Record> for #schema_name {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: libp2p::kad::Record) -> Result<Self, Self::Error> {
                <#schema_name as netabase::traits::NetabaseSchema>::from_record(value)
            }
        }
    });

    // TryFrom<SchemaEnum> for IVec
    impls.push(parse_quote! {
        impl TryFrom<#schema_name> for sled::IVec {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: #schema_name) -> Result<Self, Self::Error> {
                <#schema_name as netabase::traits::NetabaseSchema>::to_ivec(&value)
            }
        }
    });

    // TryFrom<IVec> for SchemaEnum
    impls.push(parse_quote! {
        impl TryFrom<sled::IVec> for #schema_name {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: sled::IVec) -> Result<Self, Self::Error> {
                <#schema_name as netabase::traits::NetabaseSchema>::from_ivec(value)
            }
        }
    });

    // NetabaseKeys trait impl
    impls.push(parse_quote! {
        impl netabase::traits::NetabaseKeys for #key_name {}
    });

    // TryFrom<KeyEnum> for RecordKey
    impls.push(parse_quote! {
        impl TryFrom<#key_name> for libp2p::kad::RecordKey {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: #key_name) -> Result<Self, Self::Error> {
                <#key_name as netabase::traits::NetabaseKeys>::to_record_key(&value)
            }
        }
    });

    // TryFrom<RecordKey> for KeyEnum
    impls.push(parse_quote! {
        impl TryFrom<libp2p::kad::RecordKey> for #key_name {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: libp2p::kad::RecordKey) -> Result<Self, Self::Error> {
                <#key_name as netabase::traits::NetabaseKeys>::from_record_key(value)
            }
        }
    });

    // TryFrom<KeyEnum> for IVec
    impls.push(parse_quote! {
        impl TryFrom<#key_name> for sled::IVec {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: #key_name) -> Result<Self, Self::Error> {
                <#key_name as netabase::traits::NetabaseKeys>::to_ivec(&value)
            }
        }
    });

    // TryFrom<IVec> for KeyEnum
    impls.push(parse_quote! {
        impl TryFrom<sled::IVec> for #key_name {
            type Error = netabase::errors::NetabaseError;
            fn try_from(value: sled::IVec) -> Result<Self, Self::Error> {
                <#key_name as netabase::traits::NetabaseKeys>::from_ivec(value)
            }
        }
    });

    // Helper functions for getting discriminant lists (simplified without strum)
    impls.push(parse_quote! {
        impl #schema_name {
            pub fn discriminants() -> Vec<&'static str> {
                vec![] // Simplified for now, can be enhanced later
            }
        }
    });

    impls.push(parse_quote! {
        impl #key_name {
            pub fn discriminants() -> Vec<&'static str> {
                vec![] // Simplified for now, can be enhanced later
            }
        }
    });

    (schema_enum, key_enum, impls)
}
