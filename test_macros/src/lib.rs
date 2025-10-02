use netabase_macros::netabase_schema_module;
pub mod name {
    struct Hi;
}
#[netabase_schema_module(MySchema, MyKeys)]
pub mod schema {
    use bincode::{Decode, Encode};
    use netabase::traits::{NetabaseModel, NetabaseModelKey};
    use netabase_macros::{NetabaseModel, NetabaseModelKey, key_name};

    #[derive(NetabaseModel, Debug, Clone, Encode, Decode)]
    #[key_name(ThingKey)]
    struct Thing {
        #[key]
        this_k: Vec<u8>,
    }

    #[derive(NetabaseModel, Debug, Clone, Encode, Decode)]
    #[key_name(AnotherThingKey)]
    struct AnotherThing {
        #[key]
        id: u32,
        name: String,
    }

    pub mod inner {
        use bincode::{Decode, Encode};
        use netabase::traits::{NetabaseModel, NetabaseModelKey};
        use netabase_macros::{NetabaseModel, NetabaseModelKey, key_name};

        #[derive(NetabaseModel, Debug, Clone, Encode, Decode)]
        #[key_name(OkayKey)]
        pub struct Inner {
            #[key]
            shit: u8,
        }
        pub mod nested {
            use bincode::{Decode, Encode};
            use netabase::traits::{NetabaseModel, NetabaseModelKey};
            use netabase_macros::{NetabaseModel, NetabaseModelKey, key_name};

            #[derive(NetabaseModel, Debug, Clone, Encode, Decode)]
            #[key_name(Shit)]
            pub struct Nested {
                #[key]
                pl: String,
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_generated_key_struct() {
//         // Test that the generated ThingKey struct works
//         let key1 = ThingKey("test1".to_string());
//         let key2 = ThingKey("test2".to_string());
//         let key3 = ThingKey("test1".to_string());

//         // Test Clone
//         let key1_clone = key1.clone();
//         assert_eq!(key1, key1_clone);

//         // Test PartialEq
//         assert_eq!(key1, key3);
//         assert_ne!(key1, key2);

//         // Test Debug (should not panic)
//         println!("Key debug: {:?}", key1);
//     }

//     #[test]
//     fn test_encode_decode_functionality() {
//         // Test that the automatically added Encode and Decode derives work
//         let thing = Thing {
//             this_k: "test_value".to_string(),
//         };

//         // Test encoding
//         let encoded = bincode::encode_to_vec(&thing, bincode::config::standard())
//             .expect("Failed to encode Thing");

//         // Test decoding
//         let (decoded, _): (Thing, usize) =
//             bincode::decode_from_slice(&encoded, bincode::config::standard())
//                 .expect("Failed to decode Thing");

//         // Verify the decoded value matches the original
//         assert_eq!(thing.this_k, decoded.this_k);

//         println!(
//             "Successfully encoded and decoded Thing with value: {}",
//             decoded.this_k
//         );
//     }

//     #[test]
//     fn test_existing_derives_preserved() {
//         // Test that structs with existing derives get Encode/Decode added
//         let thing = AnotherThing {
//             id: 42,
//             name: "test".to_string(),
//         };

//         // Test that existing derives still work
//         let cloned = thing.clone();
//         assert_eq!(thing.id, cloned.id);
//         assert_eq!(thing.name, cloned.name);

//         // Test Debug
//         println!("AnotherThing debug: {:?}", thing);

//         // Test that Encode/Decode were added automatically
//         let encoded = bincode::encode_to_vec(&thing, bincode::config::standard())
//             .expect("Failed to encode AnotherThing");

//         let (decoded, _): (AnotherThing, usize) =
//             bincode::decode_from_slice(&encoded, bincode::config::standard())
//                 .expect("Failed to decode AnotherThing");

//         assert_eq!(thing.id, decoded.id);
//         assert_eq!(thing.name, decoded.name);

//         // Test the generated key struct
//         let key1 = AnotherThingKey(42);
//         let key2 = AnotherThingKey(42);
//         assert_eq!(key1, key2); // Should work due to PartialEq derive
//     }
// }
