//! Key struct generation for netabase schemas
//!
//! This module generates key wrapper structs for each schema that implement
//! the necessary conversion traits to work with libp2p kad records.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::visitors::schema_finder::SchemaType;

/// Generate a key struct for a given schema
pub fn generate_key_struct(schema_type: &SchemaType) -> TokenStream {
    let schema_name = schema_type.identity();
    let key_struct_name = generate_key_struct_name(schema_name);

    quote! {
        /// Generated key struct for #schema_name
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct #key_struct_name(Vec<u8>);

        impl #key_struct_name {
            /// Create a new key from raw bytes
            pub fn new(bytes: Vec<u8>) -> Self {
                Self(bytes)
            }

            /// Get the raw bytes of the key
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            /// Convert to owned bytes
            pub fn into_bytes(self) -> Vec<u8> {
                self.0
            }
        }
    }
}

/// Generate From/Into implementations between the key struct and libp2p::kad::RecordKey
pub fn generate_key_conversions(schema_type: &SchemaType) -> TokenStream {
    let key_struct_name = generate_key_struct_name(schema_type.identity());

    quote! {
        // Convert from libp2p::kad::RecordKey to our key struct
        impl From<libp2p::kad::RecordKey> for #key_struct_name {
            fn from(record_key: libp2p::kad::RecordKey) -> Self {
                Self(record_key.to_vec())
            }
        }

        // Convert from our key struct to libp2p::kad::RecordKey
        impl From<#key_struct_name> for libp2p::kad::RecordKey {
            fn from(key: #key_struct_name) -> Self {
                libp2p::kad::RecordKey::new(&key.0)
            }
        }

        // Convert from our key struct to libp2p::kad::RecordKey (reference)
        impl From<&#key_struct_name> for libp2p::kad::RecordKey {
            fn from(key: &#key_struct_name) -> Self {
                libp2p::kad::RecordKey::new(&key.0)
            }
        }
    }
}

/// Generate Encode/Decode implementations for the key struct
pub fn generate_key_encoding(schema_type: &SchemaType) -> TokenStream {
    let key_struct_name = generate_key_struct_name(schema_type.identity());

    quote! {
        // Implement bincode Encode for the key struct
        impl bincode::Encode for #key_struct_name {
            fn encode<E: bincode::enc::Encoder>(
                &self,
                encoder: &mut E,
            ) -> Result<(), bincode::error::EncodeError> {
                self.0.encode(encoder)
            }
        }

        // Implement bincode Decode for the key struct
        impl bincode::Decode<()> for #key_struct_name {
            fn decode<D: bincode::de::Decoder>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                let bytes = Vec::<u8>::decode(decoder)?;
                Ok(Self(bytes))
            }
        }

        // Implement bincode BorrowDecode for the key struct
        impl<'de> bincode::BorrowDecode<'de, ()> for #key_struct_name {
            fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                let bytes = Vec::<u8>::borrow_decode(decoder)?;
                Ok(Self(bytes))
            }
        }
    }
}

/// Generate the complete key struct with all implementations
pub fn generate_complete_key_struct(schema_type: &SchemaType) -> TokenStream {
    let struct_def = generate_key_struct(schema_type);
    let conversions = generate_key_conversions(schema_type);
    let encoding = generate_key_encoding(schema_type);

    quote! {
        #struct_def
        #conversions
        #encoding
    }
}

/// Generate the key struct name from the schema name
pub fn generate_key_struct_name(schema_name: &Ident) -> Ident {
    let key_name = format!("{}Key", schema_name);
    Ident::new(&key_name, schema_name.span())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_key_struct_name_generation() {
        let schema_name: Ident = parse_quote!(User);
        let key_name = generate_key_struct_name(&schema_name);
        assert_eq!(key_name.to_string(), "UserKey");
    }

    #[test]
    fn test_key_struct_generation() {
        let item: syn::Item = parse_quote! {
            struct User {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let generated = generate_key_struct(&schema_type);

        // The generated code should contain the key struct definition
        let generated_str = generated.to_string();
        assert!(generated_str.contains("pub struct UserKey"));
        assert!(generated_str.contains("Vec<u8>"));
    }

    #[test]
    fn test_key_conversions_generation() {
        let item: syn::Item = parse_quote! {
            struct User {
                id: u64,
                name: String,
            }
        };

        let schema_type = SchemaType::try_from(&item).unwrap();
        let generated = generate_key_conversions(&schema_type);

        let generated_str = generated.to_string();
        assert!(generated_str.contains("impl From<libp2p::kad::RecordKey> for UserKey"));
        assert!(generated_str.contains("impl From<UserKey> for libp2p::kad::RecordKey"));
    }
}
