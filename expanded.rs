warning: unused import: `ToTokens`
 --> netabase_macros/src/generator.rs:2:13
  |
2 | use quote::{ToTokens, quote};
  |             ^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default
warning: unused imports: `ExprAssign` and `ExprLet`
 --> netabase_macros/src/generator.rs:4:5
  |
4 |     ExprAssign, ExprLet, Field, Fields, FieldsUnnamed, GenericParam, Ident, ItemEnum, ItemImpl,
  |     ^^^^^^^^^^  ^^^^^^^
warning: unused variable: `input`
  --> netabase_macros/src/lib.rs:19:32
   |
19 | pub fn derive_netabase_catalog(input: TokenStream) -> TokenStream {
   |                                ^^^^^ help: if this is intentional, prefix it with an underscore: `_input`
   |
   = note: `#[warn(unused_variables)]` on by default
warning: unused variable: `input`
  --> netabase_macros/src/lib.rs:26:36
   |
26 | pub fn derive_netabase_catalog_ref(input: TokenStream) -> TokenStream {
   |                                    ^^^^^ help: if this is intentional, prefix it with an underscore: `_input`
warning: methods `key_name` and `key_path` are never used
  --> netabase_macros/src/visitor.rs:37:12
   |
17 | impl<'ast> NativeModel<'ast> {
   | ---------------------------- methods in this implementation
...
37 |     pub fn key_name(&self) -> Ident {
   |            ^^^^^^^^
...
42 |     pub fn key_path(&self) -> Punctuated<PathSegment, syn::token::PathSep> {
   |            ^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default
warning: struct `CatalogVisitor` is never constructed
   --> netabase_macros/src/visitor.rs:131:12
    |
131 | pub struct CatalogVisitor;
    |            ^^^^^^^^^^^^^^
warning: unused import: `libp2p::kad::store::RecordStore`
 --> src/network/database/native_db_store.rs:1:5
  |
1 | use libp2p::kad::store::RecordStore;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default
warning: field `database` is never read
 --> src/network/database/native_db_store.rs:5:5
  |
4 | pub struct NativeDBStore<'db, T> {
  |            ------------- field in this struct
5 |     database: Database<'db>,
  |     ^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default
    Checking test_macros v0.1.0 (/home/nsomi/Projects/NewsNet/netabase/test_macros)
netabase_schema: module has inline content = true
netabase_schema: found 11 native models
warning: unused imports: `DateTime`, `Duration`, and `Utc`
 --> test_macros/src/lib.rs:2:14
  |
2 | use chrono::{DateTime, Duration, Utc};
  |              ^^^^^^^^  ^^^^^^^^  ^^^
  |
  = note: `#[warn(unused_imports)]` on by default
warning: unused import: `native_db::transaction::query::PrimaryScanIterator`
 --> test_macros/src/lib.rs:3:5
  |
3 | use native_db::transaction::query::PrimaryScanIterator;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
warning: unused imports: `Deserialize` and `Serialize`
 --> test_macros/src/lib.rs:5:13
  |
5 | use serde::{Deserialize, Serialize};
  |             ^^^^^^^^^^^  ^^^^^^^^^
warning: unused import: `Duration`
  --> test_macros/src/lib.rs:99:32
   |
99 |         use chrono::{DateTime, Duration, Utc};
   |                                ^^^^^^^^
warning: variant `username` should have an upper camel case name
   --> test_macros/src/lib.rs:177:17
    |
177 |             pub username: String,
    |                 ^^^^^^^^ help: convert the identifier to upper camel case (notice the capitalization): `Username`
    |
    = note: `#[warn(non_camel_case_types)]` on by default
warning: variant `email` should have an upper camel case name
   --> test_macros/src/lib.rs:179:17
    |
179 |             pub email: String,
    |                 ^^^^^ help: convert the identifier to upper camel case: `Email`
warning: variant `user_id` should have an upper camel case name
   --> test_macros/src/lib.rs:223:17
    |
223 |             pub user_id: String,
    |                 ^^^^^^^ help: convert the identifier to upper camel case: `UserId`
warning: variant `created_at` should have an upper camel case name
   --> test_macros/src/lib.rs:225:17
    |
225 |             pub created_at: i64,
    |                 ^^^^^^^^^^ help: convert the identifier to upper camel case: `CreatedAt`
warning: variant `post_id` should have an upper camel case name
   --> test_macros/src/lib.rs:259:17
    |
259 |             pub post_id: String,
    |                 ^^^^^^^ help: convert the identifier to upper camel case: `PostId`
warning: variant `user_id` should have an upper camel case name
   --> test_macros/src/lib.rs:261:17
    |
261 |             pub user_id: String,
    |                 ^^^^^^^ help: convert the identifier to upper camel case: `UserId`
warning: variant `created_at` should have an upper camel case name
   --> test_macros/src/lib.rs:263:17
    |
263 |             pub created_at: i64,
    |                 ^^^^^^^^^^ help: convert the identifier to upper camel case: `CreatedAt`
warning: variant `post_id` should have an upper camel case name
   --> test_macros/src/lib.rs:282:17
    |
282 |             pub post_id: String,
    |                 ^^^^^^^ help: convert the identifier to upper camel case: `PostId`
warning: variant `uploaded_at` should have an upper camel case name
   --> test_macros/src/lib.rs:284:17
    |
284 |             pub uploaded_at: i64,
    |                 ^^^^^^^^^^^ help: convert the identifier to upper camel case: `UploadedAt`
warning: variant `user_id` should have an upper camel case name
   --> test_macros/src/lib.rs:305:17
    |
305 |             pub user_id: String,
    |                 ^^^^^^^ help: convert the identifier to upper camel case: `UserId`
warning: variant `target_id` should have an upper camel case name
   --> test_macros/src/lib.rs:307:17
    |
307 |             pub target_id: String, // post_id or comment_id
    |                 ^^^^^^^^^ help: convert the identifier to upper camel case: `TargetId`
warning: variant `created_at` should have an upper camel case name
   --> test_macros/src/lib.rs:309:17
    |
309 |             pub created_at: i64,
    |                 ^^^^^^^^^^ help: convert the identifier to upper camel case: `CreatedAt`
warning: variant `user_id` should have an upper camel case name
   --> test_macros/src/lib.rs:323:17
    |
323 |             pub user_id: String,
    |                 ^^^^^^^ help: convert the identifier to upper camel case: `UserId`
warning: variant `created_at` should have an upper camel case name
   --> test_macros/src/lib.rs:325:17
    |
325 |             pub created_at: i64,
    |                 ^^^^^^^^^^ help: convert the identifier to upper camel case: `CreatedAt`
warning: variant `date_timestamp` should have an upper camel case name
   --> test_macros/src/lib.rs:348:17
    |
348 |             pub date_timestamp: i64, // Unix timestamp for date
    |                 ^^^^^^^^^^^^^^ help: convert the identifier to upper camel case: `DateTimestamp`
warning: variant `created_at` should have an upper camel case name
   --> test_macros/src/lib.rs:368:17
    |
368 |             pub created_at: i64,
    |                 ^^^^^^^^^^ help: convert the identifier to upper camel case: `CreatedAt`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.86s

#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use crate::social_data::v1;
use chrono::{DateTime, Duration, Utc};
use native_db::transaction::query::PrimaryScanIterator;
use netabase_macros::netabase_schema;
use serde::{Deserialize, Serialize};

// # Comprehensive Social Media Test Schema
//
// This test module provides a comprehensive social media data model that exercises
// all supported data types and struct variations in the netabase macro system.
//
// ## Test Coverage
//
// ### Primitive Types Tested:
// - **Boolean**: `bool` (is_active, is_verified)
// - **Signed integers**: `i8`, `i16`, `i32`, `i64`, `i128` (all edge cases)
// - **Unsigned integers**: `u8`, `u16`, `u32`, `u64`, `u128` (including MAX values)
// - **Floating point**: `f32`, `f64` (including infinity and precision)
// - **Character**: `char` (Unicode characters like emojis)
// - **String**: `String` (including empty strings and Unicode text)
// - **Timestamps**: `i64` (Unix timestamps for date/time representation)
//
// ### Optional Types:
// - `Option<T>` for all primitive types
// - `Some(value)` and `None` cases
//
// ### Collection Types:
// - `Vec<String>` (tags, interests, languages)
// - `HashMap<String, String>` (metadata, settings)
// - Large collections (1000+ elements) for performance testing
//
// ### Struct Variations:
// - **Named field structs**: Traditional structs with named fields (User, Post, Comment)
// - **Unit-like structs**: Minimal structs with just ID (TestUnit)
// - **Tuple-like structs**: Structs with numbered field access (TestTuple)
//
// ### Social Media Domain Model:
// - `User`: Comprehensive user profiles with all field types
// - `Post`: Social media posts with engagement metrics and geographic data
// - `Comment`: Hierarchical comments with threading support
// - `Media`: File attachments with metadata (images, videos, documents)
// - `Reaction`: User reactions to posts/comments
// - `Notification`: System notifications with various types
// - `UserStats`: Daily user activity statistics
// - `HashTag`: Trending hashtags with popularity metrics
// - `PrimitiveTest`: Dedicated struct for testing all primitive types
//
// ### NativeDB Integration:
// - Primary keys (`#[primary_key]`)
// - Secondary keys (`#[secondary_key]`) for indexing
// - All structs implement required traits for database storage
//
// ### Serialization Testing:
// - Bincode serialization/deserialization for network transport
// - JSON serialization verification (ensuring bincode is used, not JSON)
// - Round-trip testing ensuring data integrity
// - Large dataset performance testing
//
// ### Edge Cases Covered:
// - Minimum and maximum values for all numeric types
// - Infinity values for floating point numbers
// - Empty strings and collections
// - Unicode characters and emojis
// - Deeply nested optional types
// - Large collections (performance testing)
//
// ### Network Integration:
// - KadRecord conversion for libp2p DHT storage
// - Reference enum testing for memory efficiency
// - Cow<'_, Record> patterns for zero-copy operations
//
// This comprehensive test suite ensures that the netabase macro system can handle
// real-world application data models with all supported Rust types and patterns.

// Comprehensive social media data model testing all types
pub mod social_data {
    use bincode::{Decode, Encode};
    use native_db::{ToKey, native_db};
    use native_model::{Model, native_model};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    // Type aliases for latest versions
    pub type User = v1::User;
    pub type Post = v1::Post;
    pub type Comment = v1::Comment;
    pub type Media = v1::Media;
    pub type Reaction = v1::Reaction;
    pub type Notification = v1::Notification;
    pub type UserStats = v1::UserStats;
    pub type HashTag = v1::HashTag;
    pub type PrimitiveTest = v1::PrimitiveTest;
    pub type TestTuple = v1::TestTuple;
    pub type TestUnit = v1::TestUnit;

    pub mod v1 {
        use super::*;
        use chrono::{DateTime, Duration, Utc};

        // Test all primitive types
        pub struct PrimitiveTest {
            #[primary_key]
            pub id: String,

            // Boolean
            pub is_active: bool,
            pub is_verified: bool,

            // Signed integers
            pub byte_value: i8,
            pub short_value: i16,
            pub int_value: i32,
            pub long_value: i64,
            pub huge_value: i128,

            // Unsigned integers
            pub ubyte_value: u8,
            pub ushort_value: u16,
            pub uint_value: u32,
            pub ulong_value: u64,
            pub uhuge_value: u128,

            // Floating point
            pub float_value: f32,
            pub double_value: f64,

            // Character and string
            pub char_value: char,
            pub text: String,

            // Optional types
            pub optional_number: Option<i32>,
            pub optional_text: Option<String>,

            // Collections
            pub tags: Vec<String>,
            pub metadata: HashMap<String, String>,
        }
        impl native_db::db_type::ToInput for PrimitiveTest {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(PrimitiveTest::native_model_id(),
                        PrimitiveTest::native_model_version(), "id",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum PrimitiveTestKey {}
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for PrimitiveTestKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    _ =>
                        // Unit-like struct test with minimal data

                        // Tuple-like struct test (named fields simulating tuple behavior)

                        // Comprehensive user model with named fields

                        // Profile information

                        // Timestamps using chrono

                        // Numeric data

                        // Boolean flags

                        // Collections

                        // Post model with various field types


                        // Engagement metrics

                        // Post settings

                        // Geographic data

                        // Comment model


                        // Media attachment model

                        // "image", "video", "audio", "document"

                        // Reaction model
                        // post_id or comment_id

                        // "like", "love", "laugh", "angry", "sad"
                        // "post", "comment"

                        // Notification model

                        // 1-5 priority level

                        // User statistics model
                        // Unix timestamp for date


                        // HashTag model





                        // Signed integers

                        // Unsigned integers

                        // Floating point

                        // Character and string

                        // Optional types

                        // Collections


                        // Test round-trip serialization


                        // Test specific extreme values

                        // Convert Unix timestamp to DateTime

                        // Test round-trip serialization


                        // Convert Unix timestamp to DateTime
                        // DateTime for 1990-05-15

                        // Ensure consistent ordering





                        // Test round-trip serialization with all field types


                        // Test specific timestamp fields

                        // Test optional fields

                        // Test collections

                        // Test numeric types

                        // Convert Unix timestamp to DateTime

                        // New York City coordinates


                        // Test serialization with geographic and engagement data


                        // Test geographic precision

                        // Test large numbers

                        // Test collections with various string types

                        // Convert Unix timestamp to DateTime
                        // 5 minutes later





                        // Test hierarchical relationship

                        // Test smaller integer types

                        // Test optional timestamp field

                        // Convert Unix timestamp to DateTime

                        // 50MB




                        // Test large numbers

                        // Test optional u32 fields

                        // Test floating point precision

                        // March 15, 2024





                        // Test timestamp specifically

                        // Test various integer sizes

                        // Test f32 precision

                        // Unix timestamp
                        // 2 hours later





                        // Test f64 precision

                        // Test large u64

                        // Test string collections

                        // Unix timestamp
                        // 30 minutes later





                        // Test multiple optional String fields

                        // Test optional timestamp field

                        // Test u8 field

                        // Convert Unix timestamp to DateTime

                        // Create instances of multiple schema variants to test the enum generation



                        // August 20, 1995

                        // Test all variants can be serialized and deserialized


                        // Verify each variant type is correctly recovered

                        // Convert Unix timestamp to DateTime

                        // December 25, 1992


                        // Test AsKadRecord trait implementation for reference enum


                        // Verify that we're using bincode, not serde_json for network serialization
                        // July 12, 1988


                        // Bincode data should not be valid JSON

                        // But should be valid bincode

                        // Test with edge case values for numeric types

                        // Signed integer edge cases

                        // Unsigned integer edge cases

                        // Floating point edge cases

                        // Unicode character
                        // Empty string

                        // Edge cases for optionals

                        // Empty collections


                        // Test serialization with edge case values


                        // Verify specific edge case values

                        // Test with large collections to verify performance and correctness



                        // Test serialization with large collections


                        // Verify some specific elements
                        {
                        ::std::rt::begin_panic("Unknown key");
                    }
                }
            }
        }
        impl native_model::Model for PrimitiveTest {
            fn native_model_id() -> u32 { 1 }
            fn native_model_id_str() -> &'static str { "1" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 1 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for PrimitiveTest {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.is_active, encoder)?;
                ::bincode::Encode::encode(&self.is_verified, encoder)?;
                ::bincode::Encode::encode(&self.byte_value, encoder)?;
                ::bincode::Encode::encode(&self.short_value, encoder)?;
                ::bincode::Encode::encode(&self.int_value, encoder)?;
                ::bincode::Encode::encode(&self.long_value, encoder)?;
                ::bincode::Encode::encode(&self.huge_value, encoder)?;
                ::bincode::Encode::encode(&self.ubyte_value, encoder)?;
                ::bincode::Encode::encode(&self.ushort_value, encoder)?;
                ::bincode::Encode::encode(&self.uint_value, encoder)?;
                ::bincode::Encode::encode(&self.ulong_value, encoder)?;
                ::bincode::Encode::encode(&self.uhuge_value, encoder)?;
                ::bincode::Encode::encode(&self.float_value, encoder)?;
                ::bincode::Encode::encode(&self.double_value, encoder)?;
                ::bincode::Encode::encode(&self.char_value, encoder)?;
                ::bincode::Encode::encode(&self.text, encoder)?;
                ::bincode::Encode::encode(&self.optional_number, encoder)?;
                ::bincode::Encode::encode(&self.optional_text, encoder)?;
                ::bincode::Encode::encode(&self.tags, encoder)?;
                ::bincode::Encode::encode(&self.metadata, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for PrimitiveTest {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        is_active: ::bincode::Decode::decode(decoder)?,
                        is_verified: ::bincode::Decode::decode(decoder)?,
                        byte_value: ::bincode::Decode::decode(decoder)?,
                        short_value: ::bincode::Decode::decode(decoder)?,
                        int_value: ::bincode::Decode::decode(decoder)?,
                        long_value: ::bincode::Decode::decode(decoder)?,
                        huge_value: ::bincode::Decode::decode(decoder)?,
                        ubyte_value: ::bincode::Decode::decode(decoder)?,
                        ushort_value: ::bincode::Decode::decode(decoder)?,
                        uint_value: ::bincode::Decode::decode(decoder)?,
                        ulong_value: ::bincode::Decode::decode(decoder)?,
                        uhuge_value: ::bincode::Decode::decode(decoder)?,
                        float_value: ::bincode::Decode::decode(decoder)?,
                        double_value: ::bincode::Decode::decode(decoder)?,
                        char_value: ::bincode::Decode::decode(decoder)?,
                        text: ::bincode::Decode::decode(decoder)?,
                        optional_number: ::bincode::Decode::decode(decoder)?,
                        optional_text: ::bincode::Decode::decode(decoder)?,
                        tags: ::bincode::Decode::decode(decoder)?,
                        metadata: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            PrimitiveTest {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_active: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_verified: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        byte_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        short_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        int_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        long_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        huge_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        ubyte_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        ushort_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        uint_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        ulong_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        uhuge_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        float_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        double_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        char_value: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        text: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        optional_number: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        optional_text: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        tags: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        metadata: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for PrimitiveTest {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "PrimitiveTest",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 +
                                                                            1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_active", &self.is_active)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_verified", &self.is_verified)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "byte_value", &self.byte_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "short_value", &self.short_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "int_value", &self.int_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "long_value", &self.long_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "huge_value", &self.huge_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "ubyte_value", &self.ubyte_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "ushort_value", &self.ushort_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "uint_value", &self.uint_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "ulong_value", &self.ulong_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "uhuge_value", &self.uhuge_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "float_value", &self.float_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "double_value", &self.double_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "char_value", &self.char_value)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "text", &self.text)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "optional_number", &self.optional_number)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "optional_text", &self.optional_text)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "tags", &self.tags)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "metadata", &self.metadata)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for PrimitiveTest {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __field10,
                            __field11,
                            __field12,
                            __field13,
                            __field14,
                            __field15,
                            __field16,
                            __field17,
                            __field18,
                            __field19,
                            __field20,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    10u64 => _serde::__private225::Ok(__Field::__field10),
                                    11u64 => _serde::__private225::Ok(__Field::__field11),
                                    12u64 => _serde::__private225::Ok(__Field::__field12),
                                    13u64 => _serde::__private225::Ok(__Field::__field13),
                                    14u64 => _serde::__private225::Ok(__Field::__field14),
                                    15u64 => _serde::__private225::Ok(__Field::__field15),
                                    16u64 => _serde::__private225::Ok(__Field::__field16),
                                    17u64 => _serde::__private225::Ok(__Field::__field17),
                                    18u64 => _serde::__private225::Ok(__Field::__field18),
                                    19u64 => _serde::__private225::Ok(__Field::__field19),
                                    20u64 => _serde::__private225::Ok(__Field::__field20),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "is_active" => _serde::__private225::Ok(__Field::__field1),
                                    "is_verified" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    "byte_value" => _serde::__private225::Ok(__Field::__field3),
                                    "short_value" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    "int_value" => _serde::__private225::Ok(__Field::__field5),
                                    "long_value" => _serde::__private225::Ok(__Field::__field6),
                                    "huge_value" => _serde::__private225::Ok(__Field::__field7),
                                    "ubyte_value" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    "ushort_value" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    "uint_value" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    "ulong_value" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    "uhuge_value" =>
                                        _serde::__private225::Ok(__Field::__field12),
                                    "float_value" =>
                                        _serde::__private225::Ok(__Field::__field13),
                                    "double_value" =>
                                        _serde::__private225::Ok(__Field::__field14),
                                    "char_value" =>
                                        _serde::__private225::Ok(__Field::__field15),
                                    "text" => _serde::__private225::Ok(__Field::__field16),
                                    "optional_number" =>
                                        _serde::__private225::Ok(__Field::__field17),
                                    "optional_text" =>
                                        _serde::__private225::Ok(__Field::__field18),
                                    "tags" => _serde::__private225::Ok(__Field::__field19),
                                    "metadata" => _serde::__private225::Ok(__Field::__field20),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"is_active" => _serde::__private225::Ok(__Field::__field1),
                                    b"is_verified" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    b"byte_value" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"short_value" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    b"int_value" => _serde::__private225::Ok(__Field::__field5),
                                    b"long_value" =>
                                        _serde::__private225::Ok(__Field::__field6),
                                    b"huge_value" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    b"ubyte_value" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    b"ushort_value" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    b"uint_value" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    b"ulong_value" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    b"uhuge_value" =>
                                        _serde::__private225::Ok(__Field::__field12),
                                    b"float_value" =>
                                        _serde::__private225::Ok(__Field::__field13),
                                    b"double_value" =>
                                        _serde::__private225::Ok(__Field::__field14),
                                    b"char_value" =>
                                        _serde::__private225::Ok(__Field::__field15),
                                    b"text" => _serde::__private225::Ok(__Field::__field16),
                                    b"optional_number" =>
                                        _serde::__private225::Ok(__Field::__field17),
                                    b"optional_text" =>
                                        _serde::__private225::Ok(__Field::__field18),
                                    b"tags" => _serde::__private225::Ok(__Field::__field19),
                                    b"metadata" => _serde::__private225::Ok(__Field::__field20),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<PrimitiveTest>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = PrimitiveTest;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct PrimitiveTest")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<i8>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<i16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<i32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<i128>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<u8>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field10 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(10usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field11 =
                                    match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(11usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field12 =
                                    match _serde::de::SeqAccess::next_element::<u128>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(12usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field13 =
                                    match _serde::de::SeqAccess::next_element::<f32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(13usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field14 =
                                    match _serde::de::SeqAccess::next_element::<f64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(14usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field15 =
                                    match _serde::de::SeqAccess::next_element::<char>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(15usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field16 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(16usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field17 =
                                    match _serde::de::SeqAccess::next_element::<Option<i32>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(17usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field18 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(18usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field19 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(19usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                let __field20 =
                                    match _serde::de::SeqAccess::next_element::<HashMap<String,
                                                    String>>(&mut __seq)? {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(20usize,
                                                        &"struct PrimitiveTest with 21 elements")),
                                    };
                                _serde::__private225::Ok(PrimitiveTest {
                                        id: __field0,
                                        is_active: __field1,
                                        is_verified: __field2,
                                        byte_value: __field3,
                                        short_value: __field4,
                                        int_value: __field5,
                                        long_value: __field6,
                                        huge_value: __field7,
                                        ubyte_value: __field8,
                                        ushort_value: __field9,
                                        uint_value: __field10,
                                        ulong_value: __field11,
                                        uhuge_value: __field12,
                                        float_value: __field13,
                                        double_value: __field14,
                                        char_value: __field15,
                                        text: __field16,
                                        optional_number: __field17,
                                        optional_text: __field18,
                                        tags: __field19,
                                        metadata: __field20,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<i8> =
                                    _serde::__private225::None;
                                let mut __field4: _serde::__private225::Option<i16> =
                                    _serde::__private225::None;
                                let mut __field5: _serde::__private225::Option<i32> =
                                    _serde::__private225::None;
                                let mut __field6: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field7: _serde::__private225::Option<i128> =
                                    _serde::__private225::None;
                                let mut __field8: _serde::__private225::Option<u8> =
                                    _serde::__private225::None;
                                let mut __field9: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field10: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field11: _serde::__private225::Option<u64> =
                                    _serde::__private225::None;
                                let mut __field12: _serde::__private225::Option<u128> =
                                    _serde::__private225::None;
                                let mut __field13: _serde::__private225::Option<f32> =
                                    _serde::__private225::None;
                                let mut __field14: _serde::__private225::Option<f64> =
                                    _serde::__private225::None;
                                let mut __field15: _serde::__private225::Option<char> =
                                    _serde::__private225::None;
                                let mut __field16: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field17:
                                        _serde::__private225::Option<Option<i32>> =
                                    _serde::__private225::None;
                                let mut __field18:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field19:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                let mut __field20:
                                        _serde::__private225::Option<HashMap<String, String>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_active"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_verified"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("byte_value"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i8>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("short_value"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i16>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("int_value"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i32>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("long_value"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("huge_value"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i128>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("ubyte_value"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u8>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("ushort_value"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field10 => {
                                            if _serde::__private225::Option::is_some(&__field10) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("uint_value"));
                                            }
                                            __field10 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field11 => {
                                            if _serde::__private225::Option::is_some(&__field11) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("ulong_value"));
                                            }
                                            __field11 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u64>(&mut __map)?);
                                        }
                                        __Field::__field12 => {
                                            if _serde::__private225::Option::is_some(&__field12) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("uhuge_value"));
                                            }
                                            __field12 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u128>(&mut __map)?);
                                        }
                                        __Field::__field13 => {
                                            if _serde::__private225::Option::is_some(&__field13) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("float_value"));
                                            }
                                            __field13 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<f32>(&mut __map)?);
                                        }
                                        __Field::__field14 => {
                                            if _serde::__private225::Option::is_some(&__field14) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("double_value"));
                                            }
                                            __field14 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<f64>(&mut __map)?);
                                        }
                                        __Field::__field15 => {
                                            if _serde::__private225::Option::is_some(&__field15) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("char_value"));
                                            }
                                            __field15 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<char>(&mut __map)?);
                                        }
                                        __Field::__field16 => {
                                            if _serde::__private225::Option::is_some(&__field16) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("text"));
                                            }
                                            __field16 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field17 => {
                                            if _serde::__private225::Option::is_some(&__field17) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("optional_number"));
                                            }
                                            __field17 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<i32>>(&mut __map)?);
                                        }
                                        __Field::__field18 => {
                                            if _serde::__private225::Option::is_some(&__field18) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("optional_text"));
                                            }
                                            __field18 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field19 => {
                                            if _serde::__private225::Option::is_some(&__field19) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("tags"));
                                            }
                                            __field19 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        __Field::__field20 => {
                                            if _serde::__private225::Option::is_some(&__field20) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("metadata"));
                                            }
                                            __field20 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<HashMap<String,
                                                                String>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_active")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_verified")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("byte_value")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("short_value")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("int_value")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("long_value")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("huge_value")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("ubyte_value")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("ushort_value")?,
                                    };
                                let __field10 =
                                    match __field10 {
                                        _serde::__private225::Some(__field10) => __field10,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("uint_value")?,
                                    };
                                let __field11 =
                                    match __field11 {
                                        _serde::__private225::Some(__field11) => __field11,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("ulong_value")?,
                                    };
                                let __field12 =
                                    match __field12 {
                                        _serde::__private225::Some(__field12) => __field12,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("uhuge_value")?,
                                    };
                                let __field13 =
                                    match __field13 {
                                        _serde::__private225::Some(__field13) => __field13,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("float_value")?,
                                    };
                                let __field14 =
                                    match __field14 {
                                        _serde::__private225::Some(__field14) => __field14,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("double_value")?,
                                    };
                                let __field15 =
                                    match __field15 {
                                        _serde::__private225::Some(__field15) => __field15,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("char_value")?,
                                    };
                                let __field16 =
                                    match __field16 {
                                        _serde::__private225::Some(__field16) => __field16,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("text")?,
                                    };
                                let __field17 =
                                    match __field17 {
                                        _serde::__private225::Some(__field17) => __field17,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("optional_number")?,
                                    };
                                let __field18 =
                                    match __field18 {
                                        _serde::__private225::Some(__field18) => __field18,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("optional_text")?,
                                    };
                                let __field19 =
                                    match __field19 {
                                        _serde::__private225::Some(__field19) => __field19,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("tags")?,
                                    };
                                let __field20 =
                                    match __field20 {
                                        _serde::__private225::Some(__field20) => __field20,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("metadata")?,
                                    };
                                _serde::__private225::Ok(PrimitiveTest {
                                        id: __field0,
                                        is_active: __field1,
                                        is_verified: __field2,
                                        byte_value: __field3,
                                        short_value: __field4,
                                        int_value: __field5,
                                        long_value: __field6,
                                        huge_value: __field7,
                                        ubyte_value: __field8,
                                        ushort_value: __field9,
                                        uint_value: __field10,
                                        ulong_value: __field11,
                                        uhuge_value: __field12,
                                        float_value: __field13,
                                        double_value: __field14,
                                        char_value: __field15,
                                        text: __field16,
                                        optional_number: __field17,
                                        optional_text: __field18,
                                        tags: __field19,
                                        metadata: __field20,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "is_active", "is_verified", "byte_value",
                                        "short_value", "int_value", "long_value", "huge_value",
                                        "ubyte_value", "ushort_value", "uint_value", "ulong_value",
                                        "uhuge_value", "float_value", "double_value", "char_value",
                                        "text", "optional_number", "optional_text", "tags",
                                        "metadata"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "PrimitiveTest", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<PrimitiveTest>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for PrimitiveTest {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "is_active", "is_verified", "byte_value",
                                "short_value", "int_value", "long_value", "huge_value",
                                "ubyte_value", "ushort_value", "uint_value", "ulong_value",
                                "uhuge_value", "float_value", "double_value", "char_value",
                                "text", "optional_number", "optional_text", "tags",
                                "metadata"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.is_active, &self.is_verified,
                                &self.byte_value, &self.short_value, &self.int_value,
                                &self.long_value, &self.huge_value, &self.ubyte_value,
                                &self.ushort_value, &self.uint_value, &self.ulong_value,
                                &self.uhuge_value, &self.float_value, &self.double_value,
                                &self.char_value, &self.text, &self.optional_number,
                                &self.optional_text, &self.tags, &&self.metadata];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "PrimitiveTest", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PrimitiveTest {
            #[inline]
            fn clone(&self) -> PrimitiveTest {
                PrimitiveTest {
                    id: ::core::clone::Clone::clone(&self.id),
                    is_active: ::core::clone::Clone::clone(&self.is_active),
                    is_verified: ::core::clone::Clone::clone(&self.is_verified),
                    byte_value: ::core::clone::Clone::clone(&self.byte_value),
                    short_value: ::core::clone::Clone::clone(&self.short_value),
                    int_value: ::core::clone::Clone::clone(&self.int_value),
                    long_value: ::core::clone::Clone::clone(&self.long_value),
                    huge_value: ::core::clone::Clone::clone(&self.huge_value),
                    ubyte_value: ::core::clone::Clone::clone(&self.ubyte_value),
                    ushort_value: ::core::clone::Clone::clone(&self.ushort_value),
                    uint_value: ::core::clone::Clone::clone(&self.uint_value),
                    ulong_value: ::core::clone::Clone::clone(&self.ulong_value),
                    uhuge_value: ::core::clone::Clone::clone(&self.uhuge_value),
                    float_value: ::core::clone::Clone::clone(&self.float_value),
                    double_value: ::core::clone::Clone::clone(&self.double_value),
                    char_value: ::core::clone::Clone::clone(&self.char_value),
                    text: ::core::clone::Clone::clone(&self.text),
                    optional_number: ::core::clone::Clone::clone(&self.optional_number),
                    optional_text: ::core::clone::Clone::clone(&self.optional_text),
                    tags: ::core::clone::Clone::clone(&self.tags),
                    metadata: ::core::clone::Clone::clone(&self.metadata),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PrimitiveTest { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PrimitiveTest {
            #[inline]
            fn eq(&self, other: &PrimitiveTest) -> bool {
                self.is_active == other.is_active &&
                                                                                                self.is_verified == other.is_verified &&
                                                                                            self.byte_value == other.byte_value &&
                                                                                        self.short_value == other.short_value &&
                                                                                    self.int_value == other.int_value &&
                                                                                self.long_value == other.long_value &&
                                                                            self.huge_value == other.huge_value &&
                                                                        self.ubyte_value == other.ubyte_value &&
                                                                    self.ushort_value == other.ushort_value &&
                                                                self.uint_value == other.uint_value &&
                                                            self.ulong_value == other.ulong_value &&
                                                        self.uhuge_value == other.uhuge_value &&
                                                    self.float_value == other.float_value &&
                                                self.double_value == other.double_value &&
                                            self.char_value == other.char_value && self.id == other.id
                                    && self.text == other.text &&
                                self.optional_number == other.optional_number &&
                            self.optional_text == other.optional_text &&
                        self.tags == other.tags && self.metadata == other.metadata
            }
        }
        pub struct TestUnit {
            #[primary_key]
            pub id: String,
            #[bincode(with_serde)]
            pub timestamp: DateTime<Utc>,
        }
        impl native_db::db_type::ToInput for TestUnit {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(TestUnit::native_model_id(),
                        TestUnit::native_model_version(), "id",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum TestUnitKey {}
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for TestUnitKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self { _ => { ::std::rt::begin_panic("Unknown key"); } }
            }
        }
        impl native_model::Model for TestUnit {
            fn native_model_id() -> u32 { 2 }
            fn native_model_id_str() -> &'static str { "2" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 2 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for TestUnit {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.timestamp),
                        encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for TestUnit {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        timestamp: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            TestUnit {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        timestamp: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for TestUnit {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "TestUnit", false as usize + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "timestamp", &self.timestamp)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for TestUnit {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field { __field0, __field1, __ignore, }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "timestamp" => _serde::__private225::Ok(__Field::__field1),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"timestamp" => _serde::__private225::Ok(__Field::__field1),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<TestUnit>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = TestUnit;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct TestUnit")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct TestUnit with 2 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct TestUnit with 2 elements")),
                                    };
                                _serde::__private225::Ok(TestUnit {
                                        id: __field0,
                                        timestamp: __field1,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1:
                                        _serde::__private225::Option<DateTime<Utc>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("timestamp"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("timestamp")?,
                                    };
                                _serde::__private225::Ok(TestUnit {
                                        id: __field0,
                                        timestamp: __field1,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "timestamp"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "TestUnit", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<TestUnit>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for TestUnit {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(f,
                    "TestUnit", "id", &self.id, "timestamp", &&self.timestamp)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TestUnit {
            #[inline]
            fn clone(&self) -> TestUnit {
                TestUnit {
                    id: ::core::clone::Clone::clone(&self.id),
                    timestamp: ::core::clone::Clone::clone(&self.timestamp),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TestUnit { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TestUnit {
            #[inline]
            fn eq(&self, other: &TestUnit) -> bool {
                self.id == other.id && self.timestamp == other.timestamp
            }
        }
        pub struct TestTuple {
            #[primary_key]
            pub field_0: String,
            pub field_1: String,
            pub field_2: i32,
            pub field_3: bool,
            #[bincode(with_serde)]
            pub field_4: DateTime<Utc>,
        }
        impl native_db::db_type::ToInput for TestTuple {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(TestTuple::native_model_id(),
                        TestTuple::native_model_version(), "field_0",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.field_0).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum TestTupleKey {}
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for TestTupleKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self { _ => { ::std::rt::begin_panic("Unknown key"); } }
            }
        }
        impl native_model::Model for TestTuple {
            fn native_model_id() -> u32 { 3 }
            fn native_model_id_str() -> &'static str { "3" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 3 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for TestTuple {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.field_0, encoder)?;
                ::bincode::Encode::encode(&self.field_1, encoder)?;
                ::bincode::Encode::encode(&self.field_2, encoder)?;
                ::bincode::Encode::encode(&self.field_3, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.field_4),
                        encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for TestTuple {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        field_0: ::bincode::Decode::decode(decoder)?,
                        field_1: ::bincode::Decode::decode(decoder)?,
                        field_2: ::bincode::Decode::decode(decoder)?,
                        field_3: ::bincode::Decode::decode(decoder)?,
                        field_4: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            TestTuple {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        field_0: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        field_1: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        field_2: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        field_3: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        field_4: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for TestTuple {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "TestTuple", false as usize + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "field_0", &self.field_0)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "field_1", &self.field_1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "field_2", &self.field_2)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "field_3", &self.field_3)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "field_4", &self.field_4)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for TestTuple {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "field_0" => _serde::__private225::Ok(__Field::__field0),
                                    "field_1" => _serde::__private225::Ok(__Field::__field1),
                                    "field_2" => _serde::__private225::Ok(__Field::__field2),
                                    "field_3" => _serde::__private225::Ok(__Field::__field3),
                                    "field_4" => _serde::__private225::Ok(__Field::__field4),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"field_0" => _serde::__private225::Ok(__Field::__field0),
                                    b"field_1" => _serde::__private225::Ok(__Field::__field1),
                                    b"field_2" => _serde::__private225::Ok(__Field::__field2),
                                    b"field_3" => _serde::__private225::Ok(__Field::__field3),
                                    b"field_4" => _serde::__private225::Ok(__Field::__field4),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<TestTuple>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = TestTuple;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct TestTuple")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct TestTuple with 5 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct TestTuple with 5 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<i32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct TestTuple with 5 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct TestTuple with 5 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct TestTuple with 5 elements")),
                                    };
                                _serde::__private225::Ok(TestTuple {
                                        field_0: __field0,
                                        field_1: __field1,
                                        field_2: __field2,
                                        field_3: __field3,
                                        field_4: __field4,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<i32> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field4:
                                        _serde::__private225::Option<DateTime<Utc>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("field_0"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("field_1"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("field_2"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i32>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("field_3"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("field_4"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("field_0")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("field_1")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("field_2")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("field_3")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("field_4")?,
                                    };
                                _serde::__private225::Ok(TestTuple {
                                        field_0: __field0,
                                        field_1: __field1,
                                        field_2: __field2,
                                        field_3: __field3,
                                        field_4: __field4,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["field_0", "field_1", "field_2", "field_3", "field_4"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "TestTuple", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<TestTuple>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for TestTuple {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(f,
                    "TestTuple", "field_0", &self.field_0, "field_1",
                    &self.field_1, "field_2", &self.field_2, "field_3",
                    &self.field_3, "field_4", &&self.field_4)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TestTuple {
            #[inline]
            fn clone(&self) -> TestTuple {
                TestTuple {
                    field_0: ::core::clone::Clone::clone(&self.field_0),
                    field_1: ::core::clone::Clone::clone(&self.field_1),
                    field_2: ::core::clone::Clone::clone(&self.field_2),
                    field_3: ::core::clone::Clone::clone(&self.field_3),
                    field_4: ::core::clone::Clone::clone(&self.field_4),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TestTuple { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TestTuple {
            #[inline]
            fn eq(&self, other: &TestTuple) -> bool {
                self.field_2 == other.field_2 && self.field_3 == other.field_3
                            && self.field_0 == other.field_0 &&
                        self.field_1 == other.field_1 &&
                    self.field_4 == other.field_4
            }
        }
        pub struct User {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub username: String,
            #[secondary_key]
            pub email: String,
            pub display_name: Option<String>,
            pub bio: Option<String>,
            pub avatar_url: Option<String>,
            pub cover_url: Option<String>,
            #[bincode(with_serde)]
            pub created_at: DateTime<Utc>,
            #[bincode(with_serde)]
            pub updated_at: DateTime<Utc>,
            #[bincode(with_serde)]
            pub birth_timestamp: Option<DateTime<Utc>>,
            #[bincode(with_serde)]
            pub last_active: DateTime<Utc>,
            pub followers_count: u32,
            pub following_count: u32,
            pub posts_count: u32,
            pub age: Option<u8>,
            pub is_verified: bool,
            pub is_private: bool,
            pub is_active: bool,
            pub allow_messages: bool,
            pub interests: Vec<String>,
            pub languages: Vec<String>,
            pub settings: HashMap<String, String>,
        }
        impl native_db::db_type::ToInput for User {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(User::native_model_id(),
                        User::native_model_version(), "email",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(User::native_model_id(),
                        User::native_model_version(), "username",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(User::native_model_id(),
                        User::native_model_version(), "id", <String>::key_names(),
                        ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key = (&self.email).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(User::native_model_id(),
                        User::native_model_version(), "email",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key =
                    (&self.username).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(User::native_model_id(),
                        User::native_model_version(), "username",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum UserKey {

            #[allow(non_camel_case_types, dead_code)]
            email,

            #[allow(non_camel_case_types, dead_code)]
            username,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for UserKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    UserKey::email =>
                        native_db::db_type::KeyDefinition::new(User::native_model_id(),
                            User::native_model_version(), "email",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    UserKey::username =>
                        native_db::db_type::KeyDefinition::new(User::native_model_id(),
                            User::native_model_version(), "username",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for User {
            fn native_model_id() -> u32 { 4 }
            fn native_model_id_str() -> &'static str { "4" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 4 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for User {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.username, encoder)?;
                ::bincode::Encode::encode(&self.email, encoder)?;
                ::bincode::Encode::encode(&self.display_name, encoder)?;
                ::bincode::Encode::encode(&self.bio, encoder)?;
                ::bincode::Encode::encode(&self.avatar_url, encoder)?;
                ::bincode::Encode::encode(&self.cover_url, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.created_at),
                        encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.updated_at),
                        encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.birth_timestamp),
                        encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.last_active),
                        encoder)?;
                ::bincode::Encode::encode(&self.followers_count, encoder)?;
                ::bincode::Encode::encode(&self.following_count, encoder)?;
                ::bincode::Encode::encode(&self.posts_count, encoder)?;
                ::bincode::Encode::encode(&self.age, encoder)?;
                ::bincode::Encode::encode(&self.is_verified, encoder)?;
                ::bincode::Encode::encode(&self.is_private, encoder)?;
                ::bincode::Encode::encode(&self.is_active, encoder)?;
                ::bincode::Encode::encode(&self.allow_messages, encoder)?;
                ::bincode::Encode::encode(&self.interests, encoder)?;
                ::bincode::Encode::encode(&self.languages, encoder)?;
                ::bincode::Encode::encode(&self.settings, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for User {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        username: ::bincode::Decode::decode(decoder)?,
                        email: ::bincode::Decode::decode(decoder)?,
                        display_name: ::bincode::Decode::decode(decoder)?,
                        bio: ::bincode::Decode::decode(decoder)?,
                        avatar_url: ::bincode::Decode::decode(decoder)?,
                        cover_url: ::bincode::Decode::decode(decoder)?,
                        created_at: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        updated_at: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        birth_timestamp: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        last_active: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        followers_count: ::bincode::Decode::decode(decoder)?,
                        following_count: ::bincode::Decode::decode(decoder)?,
                        posts_count: ::bincode::Decode::decode(decoder)?,
                        age: ::bincode::Decode::decode(decoder)?,
                        is_verified: ::bincode::Decode::decode(decoder)?,
                        is_private: ::bincode::Decode::decode(decoder)?,
                        is_active: ::bincode::Decode::decode(decoder)?,
                        allow_messages: ::bincode::Decode::decode(decoder)?,
                        interests: ::bincode::Decode::decode(decoder)?,
                        languages: ::bincode::Decode::decode(decoder)?,
                        settings: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            User {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        username: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        email: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        display_name: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        bio: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        avatar_url: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        cover_url: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        created_at: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        updated_at: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        birth_timestamp: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        last_active: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        followers_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        following_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        posts_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        age: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_verified: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_private: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_active: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        allow_messages: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        interests: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        languages: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        settings: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for User {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer, "User",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 +
                                                                                1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "username", &self.username)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "email", &self.email)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "display_name", &self.display_name)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "bio", &self.bio)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "avatar_url", &self.avatar_url)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "cover_url", &self.cover_url)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "created_at", &self.created_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "updated_at", &self.updated_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "birth_timestamp", &self.birth_timestamp)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "last_active", &self.last_active)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "followers_count", &self.followers_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "following_count", &self.following_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "posts_count", &self.posts_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "age", &self.age)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_verified", &self.is_verified)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_private", &self.is_private)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_active", &self.is_active)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "allow_messages", &self.allow_messages)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "interests", &self.interests)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "languages", &self.languages)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "settings", &self.settings)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for User {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __field10,
                            __field11,
                            __field12,
                            __field13,
                            __field14,
                            __field15,
                            __field16,
                            __field17,
                            __field18,
                            __field19,
                            __field20,
                            __field21,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    10u64 => _serde::__private225::Ok(__Field::__field10),
                                    11u64 => _serde::__private225::Ok(__Field::__field11),
                                    12u64 => _serde::__private225::Ok(__Field::__field12),
                                    13u64 => _serde::__private225::Ok(__Field::__field13),
                                    14u64 => _serde::__private225::Ok(__Field::__field14),
                                    15u64 => _serde::__private225::Ok(__Field::__field15),
                                    16u64 => _serde::__private225::Ok(__Field::__field16),
                                    17u64 => _serde::__private225::Ok(__Field::__field17),
                                    18u64 => _serde::__private225::Ok(__Field::__field18),
                                    19u64 => _serde::__private225::Ok(__Field::__field19),
                                    20u64 => _serde::__private225::Ok(__Field::__field20),
                                    21u64 => _serde::__private225::Ok(__Field::__field21),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "username" => _serde::__private225::Ok(__Field::__field1),
                                    "email" => _serde::__private225::Ok(__Field::__field2),
                                    "display_name" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    "bio" => _serde::__private225::Ok(__Field::__field4),
                                    "avatar_url" => _serde::__private225::Ok(__Field::__field5),
                                    "cover_url" => _serde::__private225::Ok(__Field::__field6),
                                    "created_at" => _serde::__private225::Ok(__Field::__field7),
                                    "updated_at" => _serde::__private225::Ok(__Field::__field8),
                                    "birth_timestamp" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    "last_active" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    "followers_count" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    "following_count" =>
                                        _serde::__private225::Ok(__Field::__field12),
                                    "posts_count" =>
                                        _serde::__private225::Ok(__Field::__field13),
                                    "age" => _serde::__private225::Ok(__Field::__field14),
                                    "is_verified" =>
                                        _serde::__private225::Ok(__Field::__field15),
                                    "is_private" =>
                                        _serde::__private225::Ok(__Field::__field16),
                                    "is_active" => _serde::__private225::Ok(__Field::__field17),
                                    "allow_messages" =>
                                        _serde::__private225::Ok(__Field::__field18),
                                    "interests" => _serde::__private225::Ok(__Field::__field19),
                                    "languages" => _serde::__private225::Ok(__Field::__field20),
                                    "settings" => _serde::__private225::Ok(__Field::__field21),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"username" => _serde::__private225::Ok(__Field::__field1),
                                    b"email" => _serde::__private225::Ok(__Field::__field2),
                                    b"display_name" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"bio" => _serde::__private225::Ok(__Field::__field4),
                                    b"avatar_url" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    b"cover_url" => _serde::__private225::Ok(__Field::__field6),
                                    b"created_at" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    b"updated_at" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    b"birth_timestamp" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    b"last_active" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    b"followers_count" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    b"following_count" =>
                                        _serde::__private225::Ok(__Field::__field12),
                                    b"posts_count" =>
                                        _serde::__private225::Ok(__Field::__field13),
                                    b"age" => _serde::__private225::Ok(__Field::__field14),
                                    b"is_verified" =>
                                        _serde::__private225::Ok(__Field::__field15),
                                    b"is_private" =>
                                        _serde::__private225::Ok(__Field::__field16),
                                    b"is_active" =>
                                        _serde::__private225::Ok(__Field::__field17),
                                    b"allow_messages" =>
                                        _serde::__private225::Ok(__Field::__field18),
                                    b"interests" =>
                                        _serde::__private225::Ok(__Field::__field19),
                                    b"languages" =>
                                        _serde::__private225::Ok(__Field::__field20),
                                    b"settings" => _serde::__private225::Ok(__Field::__field21),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<User>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = User;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct User")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<Option<DateTime<Utc>>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field10 =
                                    match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(10usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field11 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(11usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field12 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(12usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field13 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(13usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field14 =
                                    match _serde::de::SeqAccess::next_element::<Option<u8>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(14usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field15 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(15usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field16 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(16usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field17 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(17usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field18 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(18usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field19 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(19usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field20 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(20usize,
                                                        &"struct User with 22 elements")),
                                    };
                                let __field21 =
                                    match _serde::de::SeqAccess::next_element::<HashMap<String,
                                                    String>>(&mut __seq)? {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(21usize,
                                                        &"struct User with 22 elements")),
                                    };
                                _serde::__private225::Ok(User {
                                        id: __field0,
                                        username: __field1,
                                        email: __field2,
                                        display_name: __field3,
                                        bio: __field4,
                                        avatar_url: __field5,
                                        cover_url: __field6,
                                        created_at: __field7,
                                        updated_at: __field8,
                                        birth_timestamp: __field9,
                                        last_active: __field10,
                                        followers_count: __field11,
                                        following_count: __field12,
                                        posts_count: __field13,
                                        age: __field14,
                                        is_verified: __field15,
                                        is_private: __field16,
                                        is_active: __field17,
                                        allow_messages: __field18,
                                        interests: __field19,
                                        languages: __field20,
                                        settings: __field21,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field3:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field4:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field5:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field6:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field7:
                                        _serde::__private225::Option<DateTime<Utc>> =
                                    _serde::__private225::None;
                                let mut __field8:
                                        _serde::__private225::Option<DateTime<Utc>> =
                                    _serde::__private225::None;
                                let mut __field9:
                                        _serde::__private225::Option<Option<DateTime<Utc>>> =
                                    _serde::__private225::None;
                                let mut __field10:
                                        _serde::__private225::Option<DateTime<Utc>> =
                                    _serde::__private225::None;
                                let mut __field11: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field12: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field13: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field14:
                                        _serde::__private225::Option<Option<u8>> =
                                    _serde::__private225::None;
                                let mut __field15: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field16: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field17: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field18: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field19:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                let mut __field20:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                let mut __field21:
                                        _serde::__private225::Option<HashMap<String, String>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("username"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("email"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("display_name"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("bio"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("avatar_url"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("cover_url"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("created_at"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("updated_at"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("birth_timestamp"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<DateTime<Utc>>>(&mut __map)?);
                                        }
                                        __Field::__field10 => {
                                            if _serde::__private225::Option::is_some(&__field10) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("last_active"));
                                            }
                                            __field10 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)?);
                                        }
                                        __Field::__field11 => {
                                            if _serde::__private225::Option::is_some(&__field11) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("followers_count"));
                                            }
                                            __field11 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field12 => {
                                            if _serde::__private225::Option::is_some(&__field12) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("following_count"));
                                            }
                                            __field12 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field13 => {
                                            if _serde::__private225::Option::is_some(&__field13) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("posts_count"));
                                            }
                                            __field13 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field14 => {
                                            if _serde::__private225::Option::is_some(&__field14) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("age"));
                                            }
                                            __field14 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<u8>>(&mut __map)?);
                                        }
                                        __Field::__field15 => {
                                            if _serde::__private225::Option::is_some(&__field15) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_verified"));
                                            }
                                            __field15 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field16 => {
                                            if _serde::__private225::Option::is_some(&__field16) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_private"));
                                            }
                                            __field16 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field17 => {
                                            if _serde::__private225::Option::is_some(&__field17) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_active"));
                                            }
                                            __field17 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field18 => {
                                            if _serde::__private225::Option::is_some(&__field18) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("allow_messages"));
                                            }
                                            __field18 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field19 => {
                                            if _serde::__private225::Option::is_some(&__field19) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("interests"));
                                            }
                                            __field19 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        __Field::__field20 => {
                                            if _serde::__private225::Option::is_some(&__field20) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("languages"));
                                            }
                                            __field20 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        __Field::__field21 => {
                                            if _serde::__private225::Option::is_some(&__field21) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("settings"));
                                            }
                                            __field21 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<HashMap<String,
                                                                String>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("username")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("email")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("display_name")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("bio")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("avatar_url")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("cover_url")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("created_at")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("updated_at")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("birth_timestamp")?,
                                    };
                                let __field10 =
                                    match __field10 {
                                        _serde::__private225::Some(__field10) => __field10,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("last_active")?,
                                    };
                                let __field11 =
                                    match __field11 {
                                        _serde::__private225::Some(__field11) => __field11,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("followers_count")?,
                                    };
                                let __field12 =
                                    match __field12 {
                                        _serde::__private225::Some(__field12) => __field12,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("following_count")?,
                                    };
                                let __field13 =
                                    match __field13 {
                                        _serde::__private225::Some(__field13) => __field13,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("posts_count")?,
                                    };
                                let __field14 =
                                    match __field14 {
                                        _serde::__private225::Some(__field14) => __field14,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("age")?,
                                    };
                                let __field15 =
                                    match __field15 {
                                        _serde::__private225::Some(__field15) => __field15,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_verified")?,
                                    };
                                let __field16 =
                                    match __field16 {
                                        _serde::__private225::Some(__field16) => __field16,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_private")?,
                                    };
                                let __field17 =
                                    match __field17 {
                                        _serde::__private225::Some(__field17) => __field17,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_active")?,
                                    };
                                let __field18 =
                                    match __field18 {
                                        _serde::__private225::Some(__field18) => __field18,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("allow_messages")?,
                                    };
                                let __field19 =
                                    match __field19 {
                                        _serde::__private225::Some(__field19) => __field19,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("interests")?,
                                    };
                                let __field20 =
                                    match __field20 {
                                        _serde::__private225::Some(__field20) => __field20,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("languages")?,
                                    };
                                let __field21 =
                                    match __field21 {
                                        _serde::__private225::Some(__field21) => __field21,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("settings")?,
                                    };
                                _serde::__private225::Ok(User {
                                        id: __field0,
                                        username: __field1,
                                        email: __field2,
                                        display_name: __field3,
                                        bio: __field4,
                                        avatar_url: __field5,
                                        cover_url: __field6,
                                        created_at: __field7,
                                        updated_at: __field8,
                                        birth_timestamp: __field9,
                                        last_active: __field10,
                                        followers_count: __field11,
                                        following_count: __field12,
                                        posts_count: __field13,
                                        age: __field14,
                                        is_verified: __field15,
                                        is_private: __field16,
                                        is_active: __field17,
                                        allow_messages: __field18,
                                        interests: __field19,
                                        languages: __field20,
                                        settings: __field21,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "username", "email", "display_name", "bio",
                                        "avatar_url", "cover_url", "created_at", "updated_at",
                                        "birth_timestamp", "last_active", "followers_count",
                                        "following_count", "posts_count", "age", "is_verified",
                                        "is_private", "is_active", "allow_messages", "interests",
                                        "languages", "settings"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "User", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<User>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for User {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "username", "email", "display_name", "bio",
                                "avatar_url", "cover_url", "created_at", "updated_at",
                                "birth_timestamp", "last_active", "followers_count",
                                "following_count", "posts_count", "age", "is_verified",
                                "is_private", "is_active", "allow_messages", "interests",
                                "languages", "settings"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.username, &self.email, &self.display_name,
                                &self.bio, &self.avatar_url, &self.cover_url,
                                &self.created_at, &self.updated_at, &self.birth_timestamp,
                                &self.last_active, &self.followers_count,
                                &self.following_count, &self.posts_count, &self.age,
                                &self.is_verified, &self.is_private, &self.is_active,
                                &self.allow_messages, &self.interests, &self.languages,
                                &&self.settings];
                ::core::fmt::Formatter::debug_struct_fields_finish(f, "User",
                    names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for User {
            #[inline]
            fn clone(&self) -> User {
                User {
                    id: ::core::clone::Clone::clone(&self.id),
                    username: ::core::clone::Clone::clone(&self.username),
                    email: ::core::clone::Clone::clone(&self.email),
                    display_name: ::core::clone::Clone::clone(&self.display_name),
                    bio: ::core::clone::Clone::clone(&self.bio),
                    avatar_url: ::core::clone::Clone::clone(&self.avatar_url),
                    cover_url: ::core::clone::Clone::clone(&self.cover_url),
                    created_at: ::core::clone::Clone::clone(&self.created_at),
                    updated_at: ::core::clone::Clone::clone(&self.updated_at),
                    birth_timestamp: ::core::clone::Clone::clone(&self.birth_timestamp),
                    last_active: ::core::clone::Clone::clone(&self.last_active),
                    followers_count: ::core::clone::Clone::clone(&self.followers_count),
                    following_count: ::core::clone::Clone::clone(&self.following_count),
                    posts_count: ::core::clone::Clone::clone(&self.posts_count),
                    age: ::core::clone::Clone::clone(&self.age),
                    is_verified: ::core::clone::Clone::clone(&self.is_verified),
                    is_private: ::core::clone::Clone::clone(&self.is_private),
                    is_active: ::core::clone::Clone::clone(&self.is_active),
                    allow_messages: ::core::clone::Clone::clone(&self.allow_messages),
                    interests: ::core::clone::Clone::clone(&self.interests),
                    languages: ::core::clone::Clone::clone(&self.languages),
                    settings: ::core::clone::Clone::clone(&self.settings),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for User { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for User {
            #[inline]
            fn eq(&self, other: &User) -> bool {
                self.followers_count == other.followers_count &&
                                                                                                    self.following_count == other.following_count &&
                                                                                                self.posts_count == other.posts_count &&
                                                                                            self.is_verified == other.is_verified &&
                                                                                        self.is_private == other.is_private &&
                                                                                    self.is_active == other.is_active &&
                                                                                self.allow_messages == other.allow_messages &&
                                                                            self.id == other.id && self.username == other.username &&
                                                                    self.email == other.email &&
                                                                self.display_name == other.display_name &&
                                                            self.bio == other.bio && self.avatar_url == other.avatar_url
                                                    && self.cover_url == other.cover_url &&
                                                self.created_at == other.created_at &&
                                            self.updated_at == other.updated_at &&
                                        self.birth_timestamp == other.birth_timestamp &&
                                    self.last_active == other.last_active &&
                                self.age == other.age && self.interests == other.interests
                        && self.languages == other.languages &&
                    self.settings == other.settings
            }
        }
        pub struct Post {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub created_at: i64,
            pub content: String,
            #[bincode(with_serde)]
            pub updated_at: Option<DateTime<Utc>>,
            pub media_urls: Vec<String>,
            pub hashtags: Vec<String>,
            pub mentions: Vec<String>,
            pub likes_count: u32,
            pub comments_count: u32,
            pub shares_count: u32,
            pub views_count: u64,
            pub is_public: bool,
            pub allow_comments: bool,
            pub allow_shares: bool,
            pub latitude: Option<f64>,
            pub longitude: Option<f64>,
            pub location_name: Option<String>,
        }
        impl native_db::db_type::ToInput for Post {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                        Post::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                        Post::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                        Post::native_model_version(), "id", <String>::key_names(),
                        ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key =
                    (&self.created_at).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                        Post::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key = (&self.user_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                        Post::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum PostKey {

            #[allow(non_camel_case_types, dead_code)]
            created_at,

            #[allow(non_camel_case_types, dead_code)]
            user_id,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for PostKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    PostKey::created_at =>
                        native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                            Post::native_model_version(), "created_at",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    PostKey::user_id =>
                        native_db::db_type::KeyDefinition::new(Post::native_model_id(),
                            Post::native_model_version(), "user_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for Post {
            fn native_model_id() -> u32 { 5 }
            fn native_model_id_str() -> &'static str { "5" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 5 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for Post {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.user_id, encoder)?;
                ::bincode::Encode::encode(&self.created_at, encoder)?;
                ::bincode::Encode::encode(&self.content, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.updated_at),
                        encoder)?;
                ::bincode::Encode::encode(&self.media_urls, encoder)?;
                ::bincode::Encode::encode(&self.hashtags, encoder)?;
                ::bincode::Encode::encode(&self.mentions, encoder)?;
                ::bincode::Encode::encode(&self.likes_count, encoder)?;
                ::bincode::Encode::encode(&self.comments_count, encoder)?;
                ::bincode::Encode::encode(&self.shares_count, encoder)?;
                ::bincode::Encode::encode(&self.views_count, encoder)?;
                ::bincode::Encode::encode(&self.is_public, encoder)?;
                ::bincode::Encode::encode(&self.allow_comments, encoder)?;
                ::bincode::Encode::encode(&self.allow_shares, encoder)?;
                ::bincode::Encode::encode(&self.latitude, encoder)?;
                ::bincode::Encode::encode(&self.longitude, encoder)?;
                ::bincode::Encode::encode(&self.location_name, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for Post {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        user_id: ::bincode::Decode::decode(decoder)?,
                        created_at: ::bincode::Decode::decode(decoder)?,
                        content: ::bincode::Decode::decode(decoder)?,
                        updated_at: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        media_urls: ::bincode::Decode::decode(decoder)?,
                        hashtags: ::bincode::Decode::decode(decoder)?,
                        mentions: ::bincode::Decode::decode(decoder)?,
                        likes_count: ::bincode::Decode::decode(decoder)?,
                        comments_count: ::bincode::Decode::decode(decoder)?,
                        shares_count: ::bincode::Decode::decode(decoder)?,
                        views_count: ::bincode::Decode::decode(decoder)?,
                        is_public: ::bincode::Decode::decode(decoder)?,
                        allow_comments: ::bincode::Decode::decode(decoder)?,
                        allow_shares: ::bincode::Decode::decode(decoder)?,
                        latitude: ::bincode::Decode::decode(decoder)?,
                        longitude: ::bincode::Decode::decode(decoder)?,
                        location_name: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            Post {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        user_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        created_at: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        content: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        updated_at: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        media_urls: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        hashtags: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        mentions: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        likes_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        comments_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        shares_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        views_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_public: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        allow_comments: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        allow_shares: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        latitude: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        longitude: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        location_name: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for Post {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer, "Post",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 +
                                                                1 + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "user_id", &self.user_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "created_at", &self.created_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "content", &self.content)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "updated_at", &self.updated_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "media_urls", &self.media_urls)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "hashtags", &self.hashtags)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "mentions", &self.mentions)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "likes_count", &self.likes_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "comments_count", &self.comments_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "shares_count", &self.shares_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "views_count", &self.views_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_public", &self.is_public)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "allow_comments", &self.allow_comments)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "allow_shares", &self.allow_shares)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "latitude", &self.latitude)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "longitude", &self.longitude)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "location_name", &self.location_name)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for Post {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __field10,
                            __field11,
                            __field12,
                            __field13,
                            __field14,
                            __field15,
                            __field16,
                            __field17,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    10u64 => _serde::__private225::Ok(__Field::__field10),
                                    11u64 => _serde::__private225::Ok(__Field::__field11),
                                    12u64 => _serde::__private225::Ok(__Field::__field12),
                                    13u64 => _serde::__private225::Ok(__Field::__field13),
                                    14u64 => _serde::__private225::Ok(__Field::__field14),
                                    15u64 => _serde::__private225::Ok(__Field::__field15),
                                    16u64 => _serde::__private225::Ok(__Field::__field16),
                                    17u64 => _serde::__private225::Ok(__Field::__field17),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "user_id" => _serde::__private225::Ok(__Field::__field1),
                                    "created_at" => _serde::__private225::Ok(__Field::__field2),
                                    "content" => _serde::__private225::Ok(__Field::__field3),
                                    "updated_at" => _serde::__private225::Ok(__Field::__field4),
                                    "media_urls" => _serde::__private225::Ok(__Field::__field5),
                                    "hashtags" => _serde::__private225::Ok(__Field::__field6),
                                    "mentions" => _serde::__private225::Ok(__Field::__field7),
                                    "likes_count" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    "comments_count" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    "shares_count" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    "views_count" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    "is_public" => _serde::__private225::Ok(__Field::__field12),
                                    "allow_comments" =>
                                        _serde::__private225::Ok(__Field::__field13),
                                    "allow_shares" =>
                                        _serde::__private225::Ok(__Field::__field14),
                                    "latitude" => _serde::__private225::Ok(__Field::__field15),
                                    "longitude" => _serde::__private225::Ok(__Field::__field16),
                                    "location_name" =>
                                        _serde::__private225::Ok(__Field::__field17),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"user_id" => _serde::__private225::Ok(__Field::__field1),
                                    b"created_at" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    b"content" => _serde::__private225::Ok(__Field::__field3),
                                    b"updated_at" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    b"media_urls" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    b"hashtags" => _serde::__private225::Ok(__Field::__field6),
                                    b"mentions" => _serde::__private225::Ok(__Field::__field7),
                                    b"likes_count" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    b"comments_count" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    b"shares_count" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    b"views_count" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    b"is_public" =>
                                        _serde::__private225::Ok(__Field::__field12),
                                    b"allow_comments" =>
                                        _serde::__private225::Ok(__Field::__field13),
                                    b"allow_shares" =>
                                        _serde::__private225::Ok(__Field::__field14),
                                    b"latitude" => _serde::__private225::Ok(__Field::__field15),
                                    b"longitude" =>
                                        _serde::__private225::Ok(__Field::__field16),
                                    b"location_name" =>
                                        _serde::__private225::Ok(__Field::__field17),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<Post>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Post;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct Post")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<Option<DateTime<Utc>>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field10 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(10usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field11 =
                                    match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(11usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field12 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(12usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field13 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(13usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field14 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(14usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field15 =
                                    match _serde::de::SeqAccess::next_element::<Option<f64>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(15usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field16 =
                                    match _serde::de::SeqAccess::next_element::<Option<f64>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(16usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                let __field17 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(17usize,
                                                        &"struct Post with 18 elements")),
                                    };
                                _serde::__private225::Ok(Post {
                                        id: __field0,
                                        user_id: __field1,
                                        created_at: __field2,
                                        content: __field3,
                                        updated_at: __field4,
                                        media_urls: __field5,
                                        hashtags: __field6,
                                        mentions: __field7,
                                        likes_count: __field8,
                                        comments_count: __field9,
                                        shares_count: __field10,
                                        views_count: __field11,
                                        is_public: __field12,
                                        allow_comments: __field13,
                                        allow_shares: __field14,
                                        latitude: __field15,
                                        longitude: __field16,
                                        location_name: __field17,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field4:
                                        _serde::__private225::Option<Option<DateTime<Utc>>> =
                                    _serde::__private225::None;
                                let mut __field5:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                let mut __field6:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                let mut __field7:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                let mut __field8: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field9: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field10: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field11: _serde::__private225::Option<u64> =
                                    _serde::__private225::None;
                                let mut __field12: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field13: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field14: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field15:
                                        _serde::__private225::Option<Option<f64>> =
                                    _serde::__private225::None;
                                let mut __field16:
                                        _serde::__private225::Option<Option<f64>> =
                                    _serde::__private225::None;
                                let mut __field17:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("user_id"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("created_at"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("content"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("updated_at"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<DateTime<Utc>>>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("media_urls"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("hashtags"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("mentions"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("likes_count"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("comments_count"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field10 => {
                                            if _serde::__private225::Option::is_some(&__field10) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("shares_count"));
                                            }
                                            __field10 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field11 => {
                                            if _serde::__private225::Option::is_some(&__field11) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("views_count"));
                                            }
                                            __field11 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u64>(&mut __map)?);
                                        }
                                        __Field::__field12 => {
                                            if _serde::__private225::Option::is_some(&__field12) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_public"));
                                            }
                                            __field12 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field13 => {
                                            if _serde::__private225::Option::is_some(&__field13) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("allow_comments"));
                                            }
                                            __field13 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field14 => {
                                            if _serde::__private225::Option::is_some(&__field14) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("allow_shares"));
                                            }
                                            __field14 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field15 => {
                                            if _serde::__private225::Option::is_some(&__field15) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("latitude"));
                                            }
                                            __field15 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<f64>>(&mut __map)?);
                                        }
                                        __Field::__field16 => {
                                            if _serde::__private225::Option::is_some(&__field16) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("longitude"));
                                            }
                                            __field16 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<f64>>(&mut __map)?);
                                        }
                                        __Field::__field17 => {
                                            if _serde::__private225::Option::is_some(&__field17) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("location_name"));
                                            }
                                            __field17 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("user_id")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("created_at")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("content")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("updated_at")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("media_urls")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("hashtags")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("mentions")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("likes_count")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("comments_count")?,
                                    };
                                let __field10 =
                                    match __field10 {
                                        _serde::__private225::Some(__field10) => __field10,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("shares_count")?,
                                    };
                                let __field11 =
                                    match __field11 {
                                        _serde::__private225::Some(__field11) => __field11,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("views_count")?,
                                    };
                                let __field12 =
                                    match __field12 {
                                        _serde::__private225::Some(__field12) => __field12,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_public")?,
                                    };
                                let __field13 =
                                    match __field13 {
                                        _serde::__private225::Some(__field13) => __field13,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("allow_comments")?,
                                    };
                                let __field14 =
                                    match __field14 {
                                        _serde::__private225::Some(__field14) => __field14,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("allow_shares")?,
                                    };
                                let __field15 =
                                    match __field15 {
                                        _serde::__private225::Some(__field15) => __field15,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("latitude")?,
                                    };
                                let __field16 =
                                    match __field16 {
                                        _serde::__private225::Some(__field16) => __field16,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("longitude")?,
                                    };
                                let __field17 =
                                    match __field17 {
                                        _serde::__private225::Some(__field17) => __field17,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("location_name")?,
                                    };
                                _serde::__private225::Ok(Post {
                                        id: __field0,
                                        user_id: __field1,
                                        created_at: __field2,
                                        content: __field3,
                                        updated_at: __field4,
                                        media_urls: __field5,
                                        hashtags: __field6,
                                        mentions: __field7,
                                        likes_count: __field8,
                                        comments_count: __field9,
                                        shares_count: __field10,
                                        views_count: __field11,
                                        is_public: __field12,
                                        allow_comments: __field13,
                                        allow_shares: __field14,
                                        latitude: __field15,
                                        longitude: __field16,
                                        location_name: __field17,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "user_id", "created_at", "content", "updated_at",
                                        "media_urls", "hashtags", "mentions", "likes_count",
                                        "comments_count", "shares_count", "views_count",
                                        "is_public", "allow_comments", "allow_shares", "latitude",
                                        "longitude", "location_name"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "Post", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<Post>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for Post {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "user_id", "created_at", "content", "updated_at",
                                "media_urls", "hashtags", "mentions", "likes_count",
                                "comments_count", "shares_count", "views_count",
                                "is_public", "allow_comments", "allow_shares", "latitude",
                                "longitude", "location_name"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.user_id, &self.created_at, &self.content,
                                &self.updated_at, &self.media_urls, &self.hashtags,
                                &self.mentions, &self.likes_count, &self.comments_count,
                                &self.shares_count, &self.views_count, &self.is_public,
                                &self.allow_comments, &self.allow_shares, &self.latitude,
                                &self.longitude, &&self.location_name];
                ::core::fmt::Formatter::debug_struct_fields_finish(f, "Post",
                    names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Post {
            #[inline]
            fn clone(&self) -> Post {
                Post {
                    id: ::core::clone::Clone::clone(&self.id),
                    user_id: ::core::clone::Clone::clone(&self.user_id),
                    created_at: ::core::clone::Clone::clone(&self.created_at),
                    content: ::core::clone::Clone::clone(&self.content),
                    updated_at: ::core::clone::Clone::clone(&self.updated_at),
                    media_urls: ::core::clone::Clone::clone(&self.media_urls),
                    hashtags: ::core::clone::Clone::clone(&self.hashtags),
                    mentions: ::core::clone::Clone::clone(&self.mentions),
                    likes_count: ::core::clone::Clone::clone(&self.likes_count),
                    comments_count: ::core::clone::Clone::clone(&self.comments_count),
                    shares_count: ::core::clone::Clone::clone(&self.shares_count),
                    views_count: ::core::clone::Clone::clone(&self.views_count),
                    is_public: ::core::clone::Clone::clone(&self.is_public),
                    allow_comments: ::core::clone::Clone::clone(&self.allow_comments),
                    allow_shares: ::core::clone::Clone::clone(&self.allow_shares),
                    latitude: ::core::clone::Clone::clone(&self.latitude),
                    longitude: ::core::clone::Clone::clone(&self.longitude),
                    location_name: ::core::clone::Clone::clone(&self.location_name),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Post { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Post {
            #[inline]
            fn eq(&self, other: &Post) -> bool {
                self.created_at == other.created_at &&
                                                                                    self.likes_count == other.likes_count &&
                                                                                self.comments_count == other.comments_count &&
                                                                            self.shares_count == other.shares_count &&
                                                                        self.views_count == other.views_count &&
                                                                    self.is_public == other.is_public &&
                                                                self.allow_comments == other.allow_comments &&
                                                            self.allow_shares == other.allow_shares &&
                                                        self.id == other.id && self.user_id == other.user_id &&
                                                self.content == other.content &&
                                            self.updated_at == other.updated_at &&
                                        self.media_urls == other.media_urls &&
                                    self.hashtags == other.hashtags &&
                                self.mentions == other.mentions &&
                            self.latitude == other.latitude &&
                        self.longitude == other.longitude &&
                    self.location_name == other.location_name
            }
        }
        pub struct Comment {
            #[primary_key]
            pub id: u64,
            #[secondary_key]
            pub post_id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub created_at: i64,
            pub content: String,
            pub parent_comment_id: Option<String>,
            pub likes_count: u16,
            pub replies_count: u16,
            pub is_edited: bool,
            #[bincode(with_serde)]
            pub edited_at: Option<DateTime<Utc>>,
        }
        impl native_db::db_type::ToInput for Comment {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "post_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "id", <u64>::key_names(),
                        ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key = (&self.post_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "post_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key =
                    (&self.created_at).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key = (&self.user_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                        Comment::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum CommentKey {

            #[allow(non_camel_case_types, dead_code)]
            post_id,

            #[allow(non_camel_case_types, dead_code)]
            created_at,

            #[allow(non_camel_case_types, dead_code)]
            user_id,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for CommentKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    CommentKey::post_id =>
                        native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                            Comment::native_model_version(), "post_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    CommentKey::created_at =>
                        native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                            Comment::native_model_version(), "created_at",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    CommentKey::user_id =>
                        native_db::db_type::KeyDefinition::new(Comment::native_model_id(),
                            Comment::native_model_version(), "user_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for Comment {
            fn native_model_id() -> u32 { 6 }
            fn native_model_id_str() -> &'static str { "6" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 6 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for Comment {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.post_id, encoder)?;
                ::bincode::Encode::encode(&self.user_id, encoder)?;
                ::bincode::Encode::encode(&self.created_at, encoder)?;
                ::bincode::Encode::encode(&self.content, encoder)?;
                ::bincode::Encode::encode(&self.parent_comment_id, encoder)?;
                ::bincode::Encode::encode(&self.likes_count, encoder)?;
                ::bincode::Encode::encode(&self.replies_count, encoder)?;
                ::bincode::Encode::encode(&self.is_edited, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.edited_at),
                        encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for Comment {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        post_id: ::bincode::Decode::decode(decoder)?,
                        user_id: ::bincode::Decode::decode(decoder)?,
                        created_at: ::bincode::Decode::decode(decoder)?,
                        content: ::bincode::Decode::decode(decoder)?,
                        parent_comment_id: ::bincode::Decode::decode(decoder)?,
                        likes_count: ::bincode::Decode::decode(decoder)?,
                        replies_count: ::bincode::Decode::decode(decoder)?,
                        is_edited: ::bincode::Decode::decode(decoder)?,
                        edited_at: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            Comment {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        post_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        user_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        created_at: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        content: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        parent_comment_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        likes_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        replies_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_edited: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        edited_at: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for Comment {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "Comment",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "post_id", &self.post_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "user_id", &self.user_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "created_at", &self.created_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "content", &self.content)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "parent_comment_id", &self.parent_comment_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "likes_count", &self.likes_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "replies_count", &self.replies_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_edited", &self.is_edited)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "edited_at", &self.edited_at)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for Comment {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "post_id" => _serde::__private225::Ok(__Field::__field1),
                                    "user_id" => _serde::__private225::Ok(__Field::__field2),
                                    "created_at" => _serde::__private225::Ok(__Field::__field3),
                                    "content" => _serde::__private225::Ok(__Field::__field4),
                                    "parent_comment_id" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    "likes_count" =>
                                        _serde::__private225::Ok(__Field::__field6),
                                    "replies_count" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    "is_edited" => _serde::__private225::Ok(__Field::__field8),
                                    "edited_at" => _serde::__private225::Ok(__Field::__field9),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"post_id" => _serde::__private225::Ok(__Field::__field1),
                                    b"user_id" => _serde::__private225::Ok(__Field::__field2),
                                    b"created_at" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"content" => _serde::__private225::Ok(__Field::__field4),
                                    b"parent_comment_id" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    b"likes_count" =>
                                        _serde::__private225::Ok(__Field::__field6),
                                    b"replies_count" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    b"is_edited" => _serde::__private225::Ok(__Field::__field8),
                                    b"edited_at" => _serde::__private225::Ok(__Field::__field9),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<Comment>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Comment;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct Comment")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<Option<DateTime<Utc>>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct Comment with 10 elements")),
                                    };
                                _serde::__private225::Ok(Comment {
                                        id: __field0,
                                        post_id: __field1,
                                        user_id: __field2,
                                        created_at: __field3,
                                        content: __field4,
                                        parent_comment_id: __field5,
                                        likes_count: __field6,
                                        replies_count: __field7,
                                        is_edited: __field8,
                                        edited_at: __field9,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<u64> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field4: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field5:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field6: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field7: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field8: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field9:
                                        _serde::__private225::Option<Option<DateTime<Utc>>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u64>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("post_id"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("user_id"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("created_at"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("content"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("parent_comment_id"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("likes_count"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("replies_count"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_edited"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("edited_at"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<DateTime<Utc>>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("post_id")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("user_id")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("created_at")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("content")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("parent_comment_id")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("likes_count")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("replies_count")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_edited")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("edited_at")?,
                                    };
                                _serde::__private225::Ok(Comment {
                                        id: __field0,
                                        post_id: __field1,
                                        user_id: __field2,
                                        created_at: __field3,
                                        content: __field4,
                                        parent_comment_id: __field5,
                                        likes_count: __field6,
                                        replies_count: __field7,
                                        is_edited: __field8,
                                        edited_at: __field9,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "post_id", "user_id", "created_at", "content",
                                        "parent_comment_id", "likes_count", "replies_count",
                                        "is_edited", "edited_at"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "Comment", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<Comment>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for Comment {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "post_id", "user_id", "created_at", "content",
                                "parent_comment_id", "likes_count", "replies_count",
                                "is_edited", "edited_at"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.post_id, &self.user_id, &self.created_at,
                                &self.content, &self.parent_comment_id, &self.likes_count,
                                &self.replies_count, &self.is_edited, &&self.edited_at];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "Comment", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Comment {
            #[inline]
            fn clone(&self) -> Comment {
                Comment {
                    id: ::core::clone::Clone::clone(&self.id),
                    post_id: ::core::clone::Clone::clone(&self.post_id),
                    user_id: ::core::clone::Clone::clone(&self.user_id),
                    created_at: ::core::clone::Clone::clone(&self.created_at),
                    content: ::core::clone::Clone::clone(&self.content),
                    parent_comment_id: ::core::clone::Clone::clone(&self.parent_comment_id),
                    likes_count: ::core::clone::Clone::clone(&self.likes_count),
                    replies_count: ::core::clone::Clone::clone(&self.replies_count),
                    is_edited: ::core::clone::Clone::clone(&self.is_edited),
                    edited_at: ::core::clone::Clone::clone(&self.edited_at),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Comment { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Comment {
            #[inline]
            fn eq(&self, other: &Comment) -> bool {
                self.id == other.id && self.created_at == other.created_at &&
                                                self.likes_count == other.likes_count &&
                                            self.replies_count == other.replies_count &&
                                        self.is_edited == other.is_edited &&
                                    self.post_id == other.post_id &&
                                self.user_id == other.user_id &&
                            self.content == other.content &&
                        self.parent_comment_id == other.parent_comment_id &&
                    self.edited_at == other.edited_at
            }
        }
        pub struct Media {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub post_id: String,
            #[secondary_key]
            pub uploaded_at: i64,
            pub url: String,
            pub media_type: String,
            pub filename: String,
            pub size_bytes: u64,
            pub width: Option<u32>,
            pub height: Option<u32>,
            pub duration_seconds: Option<f32>,
            pub alt_text: Option<String>,
            pub is_processed: bool,
        }
        impl native_db::db_type::ToInput for Media {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                        Media::native_model_version(), "post_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                        Media::native_model_version(), "uploaded_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                        Media::native_model_version(), "id", <String>::key_names(),
                        ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key = (&self.post_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                        Media::native_model_version(), "post_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key =
                    (&self.uploaded_at).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                        Media::native_model_version(), "uploaded_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum MediaKey {

            #[allow(non_camel_case_types, dead_code)]
            post_id,

            #[allow(non_camel_case_types, dead_code)]
            uploaded_at,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for MediaKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    MediaKey::post_id =>
                        native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                            Media::native_model_version(), "post_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    MediaKey::uploaded_at =>
                        native_db::db_type::KeyDefinition::new(Media::native_model_id(),
                            Media::native_model_version(), "uploaded_at",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for Media {
            fn native_model_id() -> u32 { 7 }
            fn native_model_id_str() -> &'static str { "7" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 7 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for Media {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.post_id, encoder)?;
                ::bincode::Encode::encode(&self.uploaded_at, encoder)?;
                ::bincode::Encode::encode(&self.url, encoder)?;
                ::bincode::Encode::encode(&self.media_type, encoder)?;
                ::bincode::Encode::encode(&self.filename, encoder)?;
                ::bincode::Encode::encode(&self.size_bytes, encoder)?;
                ::bincode::Encode::encode(&self.width, encoder)?;
                ::bincode::Encode::encode(&self.height, encoder)?;
                ::bincode::Encode::encode(&self.duration_seconds, encoder)?;
                ::bincode::Encode::encode(&self.alt_text, encoder)?;
                ::bincode::Encode::encode(&self.is_processed, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for Media {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        post_id: ::bincode::Decode::decode(decoder)?,
                        uploaded_at: ::bincode::Decode::decode(decoder)?,
                        url: ::bincode::Decode::decode(decoder)?,
                        media_type: ::bincode::Decode::decode(decoder)?,
                        filename: ::bincode::Decode::decode(decoder)?,
                        size_bytes: ::bincode::Decode::decode(decoder)?,
                        width: ::bincode::Decode::decode(decoder)?,
                        height: ::bincode::Decode::decode(decoder)?,
                        duration_seconds: ::bincode::Decode::decode(decoder)?,
                        alt_text: ::bincode::Decode::decode(decoder)?,
                        is_processed: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            Media {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        post_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        uploaded_at: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        url: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        media_type: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        filename: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        size_bytes: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        width: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        height: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        duration_seconds: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        alt_text: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_processed: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for Media {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer, "Media",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 +
                                        1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "post_id", &self.post_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "uploaded_at", &self.uploaded_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "url", &self.url)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "media_type", &self.media_type)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "filename", &self.filename)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "size_bytes", &self.size_bytes)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "width", &self.width)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "height", &self.height)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "duration_seconds", &self.duration_seconds)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "alt_text", &self.alt_text)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_processed", &self.is_processed)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for Media {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __field10,
                            __field11,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    10u64 => _serde::__private225::Ok(__Field::__field10),
                                    11u64 => _serde::__private225::Ok(__Field::__field11),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "post_id" => _serde::__private225::Ok(__Field::__field1),
                                    "uploaded_at" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    "url" => _serde::__private225::Ok(__Field::__field3),
                                    "media_type" => _serde::__private225::Ok(__Field::__field4),
                                    "filename" => _serde::__private225::Ok(__Field::__field5),
                                    "size_bytes" => _serde::__private225::Ok(__Field::__field6),
                                    "width" => _serde::__private225::Ok(__Field::__field7),
                                    "height" => _serde::__private225::Ok(__Field::__field8),
                                    "duration_seconds" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    "alt_text" => _serde::__private225::Ok(__Field::__field10),
                                    "is_processed" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"post_id" => _serde::__private225::Ok(__Field::__field1),
                                    b"uploaded_at" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    b"url" => _serde::__private225::Ok(__Field::__field3),
                                    b"media_type" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    b"filename" => _serde::__private225::Ok(__Field::__field5),
                                    b"size_bytes" =>
                                        _serde::__private225::Ok(__Field::__field6),
                                    b"width" => _serde::__private225::Ok(__Field::__field7),
                                    b"height" => _serde::__private225::Ok(__Field::__field8),
                                    b"duration_seconds" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    b"alt_text" => _serde::__private225::Ok(__Field::__field10),
                                    b"is_processed" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<Media>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Media;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct Media")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<Option<u32>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<Option<u32>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<Option<f32>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field10 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(10usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                let __field11 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(11usize,
                                                        &"struct Media with 12 elements")),
                                    };
                                _serde::__private225::Ok(Media {
                                        id: __field0,
                                        post_id: __field1,
                                        uploaded_at: __field2,
                                        url: __field3,
                                        media_type: __field4,
                                        filename: __field5,
                                        size_bytes: __field6,
                                        width: __field7,
                                        height: __field8,
                                        duration_seconds: __field9,
                                        alt_text: __field10,
                                        is_processed: __field11,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field4: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field5: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field6: _serde::__private225::Option<u64> =
                                    _serde::__private225::None;
                                let mut __field7:
                                        _serde::__private225::Option<Option<u32>> =
                                    _serde::__private225::None;
                                let mut __field8:
                                        _serde::__private225::Option<Option<u32>> =
                                    _serde::__private225::None;
                                let mut __field9:
                                        _serde::__private225::Option<Option<f32>> =
                                    _serde::__private225::None;
                                let mut __field10:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field11: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("post_id"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("uploaded_at"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("url"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("media_type"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("filename"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("size_bytes"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u64>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("width"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<u32>>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("height"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<u32>>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("duration_seconds"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<f32>>(&mut __map)?);
                                        }
                                        __Field::__field10 => {
                                            if _serde::__private225::Option::is_some(&__field10) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("alt_text"));
                                            }
                                            __field10 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field11 => {
                                            if _serde::__private225::Option::is_some(&__field11) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_processed"));
                                            }
                                            __field11 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("post_id")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("uploaded_at")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("url")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("media_type")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("filename")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("size_bytes")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("width")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("height")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("duration_seconds")?,
                                    };
                                let __field10 =
                                    match __field10 {
                                        _serde::__private225::Some(__field10) => __field10,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("alt_text")?,
                                    };
                                let __field11 =
                                    match __field11 {
                                        _serde::__private225::Some(__field11) => __field11,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_processed")?,
                                    };
                                _serde::__private225::Ok(Media {
                                        id: __field0,
                                        post_id: __field1,
                                        uploaded_at: __field2,
                                        url: __field3,
                                        media_type: __field4,
                                        filename: __field5,
                                        size_bytes: __field6,
                                        width: __field7,
                                        height: __field8,
                                        duration_seconds: __field9,
                                        alt_text: __field10,
                                        is_processed: __field11,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "post_id", "uploaded_at", "url", "media_type",
                                        "filename", "size_bytes", "width", "height",
                                        "duration_seconds", "alt_text", "is_processed"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "Media", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<Media>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for Media {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "post_id", "uploaded_at", "url", "media_type",
                                "filename", "size_bytes", "width", "height",
                                "duration_seconds", "alt_text", "is_processed"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.post_id, &self.uploaded_at, &self.url,
                                &self.media_type, &self.filename, &self.size_bytes,
                                &self.width, &self.height, &self.duration_seconds,
                                &self.alt_text, &&self.is_processed];
                ::core::fmt::Formatter::debug_struct_fields_finish(f, "Media",
                    names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Media {
            #[inline]
            fn clone(&self) -> Media {
                Media {
                    id: ::core::clone::Clone::clone(&self.id),
                    post_id: ::core::clone::Clone::clone(&self.post_id),
                    uploaded_at: ::core::clone::Clone::clone(&self.uploaded_at),
                    url: ::core::clone::Clone::clone(&self.url),
                    media_type: ::core::clone::Clone::clone(&self.media_type),
                    filename: ::core::clone::Clone::clone(&self.filename),
                    size_bytes: ::core::clone::Clone::clone(&self.size_bytes),
                    width: ::core::clone::Clone::clone(&self.width),
                    height: ::core::clone::Clone::clone(&self.height),
                    duration_seconds: ::core::clone::Clone::clone(&self.duration_seconds),
                    alt_text: ::core::clone::Clone::clone(&self.alt_text),
                    is_processed: ::core::clone::Clone::clone(&self.is_processed),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Media { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Media {
            #[inline]
            fn eq(&self, other: &Media) -> bool {
                self.uploaded_at == other.uploaded_at &&
                                                            self.size_bytes == other.size_bytes &&
                                                        self.is_processed == other.is_processed &&
                                                    self.id == other.id && self.post_id == other.post_id &&
                                            self.url == other.url && self.media_type == other.media_type
                                    && self.filename == other.filename &&
                                self.width == other.width && self.height == other.height &&
                        self.duration_seconds == other.duration_seconds &&
                    self.alt_text == other.alt_text
            }
        }
        pub struct Reaction {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub target_id: String,
            #[secondary_key]
            pub created_at: i64,
            pub reaction_type: String,
            pub target_type: String,
        }
        impl native_db::db_type::ToInput for Reaction {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "target_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "id",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key = (&self.user_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key =
                    (&self.created_at).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key =
                    (&self.target_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                        Reaction::native_model_version(), "target_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum ReactionKey {

            #[allow(non_camel_case_types, dead_code)]
            user_id,

            #[allow(non_camel_case_types, dead_code)]
            created_at,

            #[allow(non_camel_case_types, dead_code)]
            target_id,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for ReactionKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    ReactionKey::user_id =>
                        native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                            Reaction::native_model_version(), "user_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    ReactionKey::created_at =>
                        native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                            Reaction::native_model_version(), "created_at",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    ReactionKey::target_id =>
                        native_db::db_type::KeyDefinition::new(Reaction::native_model_id(),
                            Reaction::native_model_version(), "target_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for Reaction {
            fn native_model_id() -> u32 { 8 }
            fn native_model_id_str() -> &'static str { "8" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 8 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for Reaction {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.user_id, encoder)?;
                ::bincode::Encode::encode(&self.target_id, encoder)?;
                ::bincode::Encode::encode(&self.created_at, encoder)?;
                ::bincode::Encode::encode(&self.reaction_type, encoder)?;
                ::bincode::Encode::encode(&self.target_type, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for Reaction {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        user_id: ::bincode::Decode::decode(decoder)?,
                        target_id: ::bincode::Decode::decode(decoder)?,
                        created_at: ::bincode::Decode::decode(decoder)?,
                        reaction_type: ::bincode::Decode::decode(decoder)?,
                        target_type: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            Reaction {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        user_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        target_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        created_at: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        reaction_type: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        target_type: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for Reaction {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "Reaction", false as usize + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "user_id", &self.user_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "target_id", &self.target_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "created_at", &self.created_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "reaction_type", &self.reaction_type)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "target_type", &self.target_type)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for Reaction {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "user_id" => _serde::__private225::Ok(__Field::__field1),
                                    "target_id" => _serde::__private225::Ok(__Field::__field2),
                                    "created_at" => _serde::__private225::Ok(__Field::__field3),
                                    "reaction_type" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    "target_type" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"user_id" => _serde::__private225::Ok(__Field::__field1),
                                    b"target_id" => _serde::__private225::Ok(__Field::__field2),
                                    b"created_at" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"reaction_type" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    b"target_type" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<Reaction>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Reaction;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct Reaction")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct Reaction with 6 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct Reaction with 6 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct Reaction with 6 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct Reaction with 6 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct Reaction with 6 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct Reaction with 6 elements")),
                                    };
                                _serde::__private225::Ok(Reaction {
                                        id: __field0,
                                        user_id: __field1,
                                        target_id: __field2,
                                        created_at: __field3,
                                        reaction_type: __field4,
                                        target_type: __field5,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field4: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field5: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("user_id"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("target_id"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("created_at"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("reaction_type"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("target_type"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("user_id")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("target_id")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("created_at")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("reaction_type")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("target_type")?,
                                    };
                                _serde::__private225::Ok(Reaction {
                                        id: __field0,
                                        user_id: __field1,
                                        target_id: __field2,
                                        created_at: __field3,
                                        reaction_type: __field4,
                                        target_type: __field5,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "user_id", "target_id", "created_at",
                                        "reaction_type", "target_type"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "Reaction", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<Reaction>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for Reaction {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "user_id", "target_id", "created_at",
                                "reaction_type", "target_type"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.user_id, &self.target_id,
                                &self.created_at, &self.reaction_type, &&self.target_type];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "Reaction", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Reaction {
            #[inline]
            fn clone(&self) -> Reaction {
                Reaction {
                    id: ::core::clone::Clone::clone(&self.id),
                    user_id: ::core::clone::Clone::clone(&self.user_id),
                    target_id: ::core::clone::Clone::clone(&self.target_id),
                    created_at: ::core::clone::Clone::clone(&self.created_at),
                    reaction_type: ::core::clone::Clone::clone(&self.reaction_type),
                    target_type: ::core::clone::Clone::clone(&self.target_type),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Reaction { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Reaction {
            #[inline]
            fn eq(&self, other: &Reaction) -> bool {
                self.created_at == other.created_at && self.id == other.id &&
                                self.user_id == other.user_id &&
                            self.target_id == other.target_id &&
                        self.reaction_type == other.reaction_type &&
                    self.target_type == other.target_type
            }
        }
        pub struct Notification {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub created_at: i64,
            pub notification_type: String,
            pub title: String,
            pub message: String,
            pub is_read: bool,
            #[bincode(with_serde)]
            pub read_at: Option<DateTime<Utc>>,
            pub related_user_id: Option<String>,
            pub related_post_id: Option<String>,
            pub related_comment_id: Option<String>,
            pub action_url: Option<String>,
            pub priority: u8,
        }
        impl native_db::db_type::ToInput for Notification {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                        Notification::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                        Notification::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                        Notification::native_model_version(), "id",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key =
                    (&self.created_at).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                        Notification::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                let value: native_db::db_type::Key = (&self.user_id).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                        Notification::native_model_version(), "user_id",
                        <String>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum NotificationKey {

            #[allow(non_camel_case_types, dead_code)]
            created_at,

            #[allow(non_camel_case_types, dead_code)]
            user_id,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for NotificationKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    NotificationKey::created_at =>
                        native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                            Notification::native_model_version(), "created_at",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    NotificationKey::user_id =>
                        native_db::db_type::KeyDefinition::new(Notification::native_model_id(),
                            Notification::native_model_version(), "user_id",
                            <String>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for Notification {
            fn native_model_id() -> u32 { 9 }
            fn native_model_id_str() -> &'static str { "9" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 9 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for Notification {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.id, encoder)?;
                ::bincode::Encode::encode(&self.user_id, encoder)?;
                ::bincode::Encode::encode(&self.created_at, encoder)?;
                ::bincode::Encode::encode(&self.notification_type, encoder)?;
                ::bincode::Encode::encode(&self.title, encoder)?;
                ::bincode::Encode::encode(&self.message, encoder)?;
                ::bincode::Encode::encode(&self.is_read, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.read_at),
                        encoder)?;
                ::bincode::Encode::encode(&self.related_user_id, encoder)?;
                ::bincode::Encode::encode(&self.related_post_id, encoder)?;
                ::bincode::Encode::encode(&self.related_comment_id, encoder)?;
                ::bincode::Encode::encode(&self.action_url, encoder)?;
                ::bincode::Encode::encode(&self.priority, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for Notification {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::Decode::decode(decoder)?,
                        user_id: ::bincode::Decode::decode(decoder)?,
                        created_at: ::bincode::Decode::decode(decoder)?,
                        notification_type: ::bincode::Decode::decode(decoder)?,
                        title: ::bincode::Decode::decode(decoder)?,
                        message: ::bincode::Decode::decode(decoder)?,
                        is_read: ::bincode::Decode::decode(decoder)?,
                        read_at: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        related_user_id: ::bincode::Decode::decode(decoder)?,
                        related_post_id: ::bincode::Decode::decode(decoder)?,
                        related_comment_id: ::bincode::Decode::decode(decoder)?,
                        action_url: ::bincode::Decode::decode(decoder)?,
                        priority: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            Notification {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        user_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        created_at: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        notification_type: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        title: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        message: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        is_read: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        read_at: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        related_user_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        related_post_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        related_comment_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        action_url: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        priority: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for Notification {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "Notification",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 +
                                            1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "id", &self.id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "user_id", &self.user_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "created_at", &self.created_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "notification_type", &self.notification_type)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "title", &self.title)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "message", &self.message)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_read", &self.is_read)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "read_at", &self.read_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "related_user_id", &self.related_user_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "related_post_id", &self.related_post_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "related_comment_id", &self.related_comment_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "action_url", &self.action_url)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "priority", &self.priority)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for Notification {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __field10,
                            __field11,
                            __field12,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    10u64 => _serde::__private225::Ok(__Field::__field10),
                                    11u64 => _serde::__private225::Ok(__Field::__field11),
                                    12u64 => _serde::__private225::Ok(__Field::__field12),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "id" => _serde::__private225::Ok(__Field::__field0),
                                    "user_id" => _serde::__private225::Ok(__Field::__field1),
                                    "created_at" => _serde::__private225::Ok(__Field::__field2),
                                    "notification_type" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    "title" => _serde::__private225::Ok(__Field::__field4),
                                    "message" => _serde::__private225::Ok(__Field::__field5),
                                    "is_read" => _serde::__private225::Ok(__Field::__field6),
                                    "read_at" => _serde::__private225::Ok(__Field::__field7),
                                    "related_user_id" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    "related_post_id" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    "related_comment_id" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    "action_url" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    "priority" => _serde::__private225::Ok(__Field::__field12),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"id" => _serde::__private225::Ok(__Field::__field0),
                                    b"user_id" => _serde::__private225::Ok(__Field::__field1),
                                    b"created_at" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    b"notification_type" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"title" => _serde::__private225::Ok(__Field::__field4),
                                    b"message" => _serde::__private225::Ok(__Field::__field5),
                                    b"is_read" => _serde::__private225::Ok(__Field::__field6),
                                    b"read_at" => _serde::__private225::Ok(__Field::__field7),
                                    b"related_user_id" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    b"related_post_id" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    b"related_comment_id" =>
                                        _serde::__private225::Ok(__Field::__field10),
                                    b"action_url" =>
                                        _serde::__private225::Ok(__Field::__field11),
                                    b"priority" => _serde::__private225::Ok(__Field::__field12),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<Notification>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Notification;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct Notification")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<Option<DateTime<Utc>>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field10 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(10usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field11 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(11usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                let __field12 =
                                    match _serde::de::SeqAccess::next_element::<u8>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(12usize,
                                                        &"struct Notification with 13 elements")),
                                    };
                                _serde::__private225::Ok(Notification {
                                        id: __field0,
                                        user_id: __field1,
                                        created_at: __field2,
                                        notification_type: __field3,
                                        title: __field4,
                                        message: __field5,
                                        is_read: __field6,
                                        read_at: __field7,
                                        related_user_id: __field8,
                                        related_post_id: __field9,
                                        related_comment_id: __field10,
                                        action_url: __field11,
                                        priority: __field12,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field4: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field5: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field6: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field7:
                                        _serde::__private225::Option<Option<DateTime<Utc>>> =
                                    _serde::__private225::None;
                                let mut __field8:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field9:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field10:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field11:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field12: _serde::__private225::Option<u8> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("user_id"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("created_at"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("notification_type"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("title"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("message"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_read"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("read_at"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<DateTime<Utc>>>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("related_user_id"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("related_post_id"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field10 => {
                                            if _serde::__private225::Option::is_some(&__field10) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("related_comment_id"));
                                            }
                                            __field10 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field11 => {
                                            if _serde::__private225::Option::is_some(&__field11) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("action_url"));
                                            }
                                            __field11 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field12 => {
                                            if _serde::__private225::Option::is_some(&__field12) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("priority"));
                                            }
                                            __field12 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u8>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("user_id")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("created_at")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("notification_type")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("title")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("message")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_read")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("read_at")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("related_user_id")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("related_post_id")?,
                                    };
                                let __field10 =
                                    match __field10 {
                                        _serde::__private225::Some(__field10) => __field10,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("related_comment_id")?,
                                    };
                                let __field11 =
                                    match __field11 {
                                        _serde::__private225::Some(__field11) => __field11,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("action_url")?,
                                    };
                                let __field12 =
                                    match __field12 {
                                        _serde::__private225::Some(__field12) => __field12,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("priority")?,
                                    };
                                _serde::__private225::Ok(Notification {
                                        id: __field0,
                                        user_id: __field1,
                                        created_at: __field2,
                                        notification_type: __field3,
                                        title: __field4,
                                        message: __field5,
                                        is_read: __field6,
                                        read_at: __field7,
                                        related_user_id: __field8,
                                        related_post_id: __field9,
                                        related_comment_id: __field10,
                                        action_url: __field11,
                                        priority: __field12,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["id", "user_id", "created_at", "notification_type",
                                        "title", "message", "is_read", "read_at", "related_user_id",
                                        "related_post_id", "related_comment_id", "action_url",
                                        "priority"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "Notification", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<Notification>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for Notification {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["id", "user_id", "created_at", "notification_type",
                                "title", "message", "is_read", "read_at", "related_user_id",
                                "related_post_id", "related_comment_id", "action_url",
                                "priority"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.id, &self.user_id, &self.created_at,
                                &self.notification_type, &self.title, &self.message,
                                &self.is_read, &self.read_at, &self.related_user_id,
                                &self.related_post_id, &self.related_comment_id,
                                &self.action_url, &&self.priority];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "Notification", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Notification {
            #[inline]
            fn clone(&self) -> Notification {
                Notification {
                    id: ::core::clone::Clone::clone(&self.id),
                    user_id: ::core::clone::Clone::clone(&self.user_id),
                    created_at: ::core::clone::Clone::clone(&self.created_at),
                    notification_type: ::core::clone::Clone::clone(&self.notification_type),
                    title: ::core::clone::Clone::clone(&self.title),
                    message: ::core::clone::Clone::clone(&self.message),
                    is_read: ::core::clone::Clone::clone(&self.is_read),
                    read_at: ::core::clone::Clone::clone(&self.read_at),
                    related_user_id: ::core::clone::Clone::clone(&self.related_user_id),
                    related_post_id: ::core::clone::Clone::clone(&self.related_post_id),
                    related_comment_id: ::core::clone::Clone::clone(&self.related_comment_id),
                    action_url: ::core::clone::Clone::clone(&self.action_url),
                    priority: ::core::clone::Clone::clone(&self.priority),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Notification { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Notification {
            #[inline]
            fn eq(&self, other: &Notification) -> bool {
                self.created_at == other.created_at &&
                                                                self.is_read == other.is_read &&
                                                            self.priority == other.priority && self.id == other.id &&
                                                    self.user_id == other.user_id &&
                                                self.notification_type == other.notification_type &&
                                            self.title == other.title && self.message == other.message
                                    && self.read_at == other.read_at &&
                                self.related_user_id == other.related_user_id &&
                            self.related_post_id == other.related_post_id &&
                        self.related_comment_id == other.related_comment_id &&
                    self.action_url == other.action_url
            }
        }
        pub struct UserStats {
            #[primary_key]
            pub user_id: String,
            #[secondary_key]
            pub date_timestamp: i64,
            pub posts_created: u16,
            pub comments_made: u16,
            pub likes_given: u16,
            pub likes_received: u16,
            pub profile_views: u32,
            pub time_spent_minutes: u32,
            pub login_count: u8,
            pub avg_session_duration: f32,
        }
        impl native_db::db_type::ToInput for UserStats {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(UserStats::native_model_id(),
                        UserStats::native_model_version(), "date_timestamp",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(UserStats::native_model_id(),
                        UserStats::native_model_version(), "user_id",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.user_id).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key =
                    (&self.date_timestamp).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(UserStats::native_model_id(),
                        UserStats::native_model_version(), "date_timestamp",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum UserStatsKey {

            #[allow(non_camel_case_types, dead_code)]
            date_timestamp,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for UserStatsKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    UserStatsKey::date_timestamp =>
                        native_db::db_type::KeyDefinition::new(UserStats::native_model_id(),
                            UserStats::native_model_version(), "date_timestamp",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for UserStats {
            fn native_model_id() -> u32 { 10 }
            fn native_model_id_str() -> &'static str { "10" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 10 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for UserStats {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.user_id, encoder)?;
                ::bincode::Encode::encode(&self.date_timestamp, encoder)?;
                ::bincode::Encode::encode(&self.posts_created, encoder)?;
                ::bincode::Encode::encode(&self.comments_made, encoder)?;
                ::bincode::Encode::encode(&self.likes_given, encoder)?;
                ::bincode::Encode::encode(&self.likes_received, encoder)?;
                ::bincode::Encode::encode(&self.profile_views, encoder)?;
                ::bincode::Encode::encode(&self.time_spent_minutes, encoder)?;
                ::bincode::Encode::encode(&self.login_count, encoder)?;
                ::bincode::Encode::encode(&self.avg_session_duration,
                        encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for UserStats {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        user_id: ::bincode::Decode::decode(decoder)?,
                        date_timestamp: ::bincode::Decode::decode(decoder)?,
                        posts_created: ::bincode::Decode::decode(decoder)?,
                        comments_made: ::bincode::Decode::decode(decoder)?,
                        likes_given: ::bincode::Decode::decode(decoder)?,
                        likes_received: ::bincode::Decode::decode(decoder)?,
                        profile_views: ::bincode::Decode::decode(decoder)?,
                        time_spent_minutes: ::bincode::Decode::decode(decoder)?,
                        login_count: ::bincode::Decode::decode(decoder)?,
                        avg_session_duration: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            UserStats {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        user_id: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        date_timestamp: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        posts_created: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        comments_made: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        likes_given: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        likes_received: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        profile_views: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        time_spent_minutes: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        login_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        avg_session_duration: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for UserStats {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "UserStats",
                                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "user_id", &self.user_id)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "date_timestamp", &self.date_timestamp)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "posts_created", &self.posts_created)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "comments_made", &self.comments_made)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "likes_given", &self.likes_given)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "likes_received", &self.likes_received)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "profile_views", &self.profile_views)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "time_spent_minutes", &self.time_spent_minutes)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "login_count", &self.login_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "avg_session_duration", &self.avg_session_duration)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for UserStats {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __field8,
                            __field9,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    8u64 => _serde::__private225::Ok(__Field::__field8),
                                    9u64 => _serde::__private225::Ok(__Field::__field9),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "user_id" => _serde::__private225::Ok(__Field::__field0),
                                    "date_timestamp" =>
                                        _serde::__private225::Ok(__Field::__field1),
                                    "posts_created" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    "comments_made" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    "likes_given" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    "likes_received" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    "profile_views" =>
                                        _serde::__private225::Ok(__Field::__field6),
                                    "time_spent_minutes" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    "login_count" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    "avg_session_duration" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"user_id" => _serde::__private225::Ok(__Field::__field0),
                                    b"date_timestamp" =>
                                        _serde::__private225::Ok(__Field::__field1),
                                    b"posts_created" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    b"comments_made" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"likes_given" =>
                                        _serde::__private225::Ok(__Field::__field4),
                                    b"likes_received" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    b"profile_views" =>
                                        _serde::__private225::Ok(__Field::__field6),
                                    b"time_spent_minutes" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    b"login_count" =>
                                        _serde::__private225::Ok(__Field::__field8),
                                    b"avg_session_duration" =>
                                        _serde::__private225::Ok(__Field::__field9),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<UserStats>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = UserStats;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct UserStats")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<u16>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field8 =
                                    match _serde::de::SeqAccess::next_element::<u8>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(8usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                let __field9 =
                                    match _serde::de::SeqAccess::next_element::<f32>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(9usize,
                                                        &"struct UserStats with 10 elements")),
                                    };
                                _serde::__private225::Ok(UserStats {
                                        user_id: __field0,
                                        date_timestamp: __field1,
                                        posts_created: __field2,
                                        comments_made: __field3,
                                        likes_given: __field4,
                                        likes_received: __field5,
                                        profile_views: __field6,
                                        time_spent_minutes: __field7,
                                        login_count: __field8,
                                        avg_session_duration: __field9,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field4: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field5: _serde::__private225::Option<u16> =
                                    _serde::__private225::None;
                                let mut __field6: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field7: _serde::__private225::Option<u32> =
                                    _serde::__private225::None;
                                let mut __field8: _serde::__private225::Option<u8> =
                                    _serde::__private225::None;
                                let mut __field9: _serde::__private225::Option<f32> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("user_id"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("date_timestamp"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("posts_created"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("comments_made"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("likes_given"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("likes_received"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u16>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("profile_views"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("time_spent_minutes"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u32>(&mut __map)?);
                                        }
                                        __Field::__field8 => {
                                            if _serde::__private225::Option::is_some(&__field8) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("login_count"));
                                            }
                                            __field8 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u8>(&mut __map)?);
                                        }
                                        __Field::__field9 => {
                                            if _serde::__private225::Option::is_some(&__field9) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("avg_session_duration"));
                                            }
                                            __field9 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<f32>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("user_id")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("date_timestamp")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("posts_created")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("comments_made")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("likes_given")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("likes_received")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("profile_views")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("time_spent_minutes")?,
                                    };
                                let __field8 =
                                    match __field8 {
                                        _serde::__private225::Some(__field8) => __field8,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("login_count")?,
                                    };
                                let __field9 =
                                    match __field9 {
                                        _serde::__private225::Some(__field9) => __field9,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("avg_session_duration")?,
                                    };
                                _serde::__private225::Ok(UserStats {
                                        user_id: __field0,
                                        date_timestamp: __field1,
                                        posts_created: __field2,
                                        comments_made: __field3,
                                        likes_given: __field4,
                                        likes_received: __field5,
                                        profile_views: __field6,
                                        time_spent_minutes: __field7,
                                        login_count: __field8,
                                        avg_session_duration: __field9,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["user_id", "date_timestamp", "posts_created",
                                        "comments_made", "likes_given", "likes_received",
                                        "profile_views", "time_spent_minutes", "login_count",
                                        "avg_session_duration"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "UserStats", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<UserStats>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for UserStats {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["user_id", "date_timestamp", "posts_created",
                                "comments_made", "likes_given", "likes_received",
                                "profile_views", "time_spent_minutes", "login_count",
                                "avg_session_duration"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.user_id, &self.date_timestamp, &self.posts_created,
                                &self.comments_made, &self.likes_given,
                                &self.likes_received, &self.profile_views,
                                &self.time_spent_minutes, &self.login_count,
                                &&self.avg_session_duration];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "UserStats", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UserStats {
            #[inline]
            fn clone(&self) -> UserStats {
                UserStats {
                    user_id: ::core::clone::Clone::clone(&self.user_id),
                    date_timestamp: ::core::clone::Clone::clone(&self.date_timestamp),
                    posts_created: ::core::clone::Clone::clone(&self.posts_created),
                    comments_made: ::core::clone::Clone::clone(&self.comments_made),
                    likes_given: ::core::clone::Clone::clone(&self.likes_given),
                    likes_received: ::core::clone::Clone::clone(&self.likes_received),
                    profile_views: ::core::clone::Clone::clone(&self.profile_views),
                    time_spent_minutes: ::core::clone::Clone::clone(&self.time_spent_minutes),
                    login_count: ::core::clone::Clone::clone(&self.login_count),
                    avg_session_duration: ::core::clone::Clone::clone(&self.avg_session_duration),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UserStats { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UserStats {
            #[inline]
            fn eq(&self, other: &UserStats) -> bool {
                self.date_timestamp == other.date_timestamp &&
                                                    self.posts_created == other.posts_created &&
                                                self.comments_made == other.comments_made &&
                                            self.likes_given == other.likes_given &&
                                        self.likes_received == other.likes_received &&
                                    self.profile_views == other.profile_views &&
                                self.time_spent_minutes == other.time_spent_minutes &&
                            self.login_count == other.login_count &&
                        self.avg_session_duration == other.avg_session_duration &&
                    self.user_id == other.user_id
            }
        }
        pub struct HashTag {
            #[primary_key]
            pub tag: String,
            #[secondary_key]
            pub created_at: i64,
            pub usage_count: u64,
            pub trending_score: f64,
            #[bincode(with_serde)]
            pub last_used: DateTime<Utc>,
            pub is_trending: bool,
            pub category: Option<String>,
            pub related_tags: Vec<String>,
        }
        impl native_db::db_type::ToInput for HashTag {
            fn native_db_bincode_encode_to_vec(&self)
                -> native_db::db_type::Result<Vec<u8>> {
                native_db::bincode_encode_to_vec(self)
            }
            fn native_db_bincode_decode_from_slice(slice: &[u8])
                -> native_db::db_type::Result<Self> {
                Ok(native_db::bincode_decode_from_slice(slice)?.0)
            }
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name =
                    std::collections::HashSet::new();
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(HashTag::native_model_id(),
                        HashTag::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }));
                native_db::Model {
                    primary_key: native_db::db_type::KeyDefinition::new(HashTag::native_model_id(),
                        HashTag::native_model_version(), "tag",
                        <String>::key_names(), ()),
                    secondary_keys: secondary_tables_name,
                }
            }
            fn native_db_primary_key(&self) -> native_db::db_type::Key {
                (&self.tag).to_key()
            }
            fn native_db_secondary_keys(&self)
                ->
                    std::collections::HashMap<native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
                    native_db::db_type::KeyEntry> {
                let mut secondary_tables_name =
                    std::collections::HashMap::new();
                let value: native_db::db_type::Key =
                    (&self.created_at).to_key();
                let value = native_db::db_type::KeyEntry::Default(value);
                secondary_tables_name.insert(native_db::db_type::KeyDefinition::new(HashTag::native_model_id(),
                        HashTag::native_model_version(), "created_at",
                        <i64>::key_names(),
                        native_db::db_type::KeyOptions {
                            unique: false,
                            optional: false,
                        }), value);
                secondary_tables_name
            }
        }
        #[allow(non_camel_case_types)]
        pub(crate) enum HashTagKey {

            #[allow(non_camel_case_types, dead_code)]
            created_at,
        }
        impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
            for HashTagKey {
            fn key_definition(&self)
                ->
                    native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
                match self {
                    HashTagKey::created_at =>
                        native_db::db_type::KeyDefinition::new(HashTag::native_model_id(),
                            HashTag::native_model_version(), "created_at",
                            <i64>::key_names(),
                            native_db::db_type::KeyOptions {
                                unique: false,
                                optional: false,
                            }),
                    _ => { ::std::rt::begin_panic("Unknown key"); }
                }
            }
        }
        impl native_model::Model for HashTag {
            fn native_model_id() -> u32 { 11 }
            fn native_model_id_str() -> &'static str { "11" }
            fn native_model_version() -> u32 { 1 }
            fn native_model_version_str() -> &'static str { "1" }
            fn native_model_encode_body(&self)
                ->
                    std::result::Result<Vec<u8>,
                    native_model::EncodeBodyError> {
                use native_model::Encode;
                native_model::bincode_1_3::Bincode::encode(self).map_err(|e|
                        native_model::EncodeBodyError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_encode_downgrade_body(self, version: u32)
                -> native_model::Result<Vec<u8>> {
                if version == Self::native_model_version() {
                    let result = self.native_model_encode_body()?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::DowngradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
            fn native_model_decode_body(data: Vec<u8>, id: u32)
                -> std::result::Result<Self, native_model::DecodeBodyError> {
                if id != 11 {
                    return Err(native_model::DecodeBodyError::MismatchedModelId);
                }
                use native_model::Decode;
                native_model::bincode_1_3::Bincode::decode(data).map_err(|e|
                        native_model::DecodeBodyError::DecodeError {
                            msg: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0}", e))
                                }),
                            source: e.into(),
                        })
            }
            fn native_model_decode_upgrade_body(data: Vec<u8>, id: u32,
                version: u32) -> native_model::Result<Self> {
                if version == Self::native_model_version() {
                    let result = Self::native_model_decode_body(data, id)?;
                    Ok(result)
                } else if version < Self::native_model_version() {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                } else {
                    Err(native_model::Error::UpgradeNotSupported {
                            from: version,
                            to: Self::native_model_version(),
                        })
                }
            }
        }
        impl ::bincode::Encode for HashTag {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
                -> core::result::Result<(), ::bincode::error::EncodeError> {
                ::bincode::Encode::encode(&self.tag, encoder)?;
                ::bincode::Encode::encode(&self.created_at, encoder)?;
                ::bincode::Encode::encode(&self.usage_count, encoder)?;
                ::bincode::Encode::encode(&self.trending_score, encoder)?;
                ::bincode::Encode::encode(&::bincode::serde::Compat(&self.last_used),
                        encoder)?;
                ::bincode::Encode::encode(&self.is_trending, encoder)?;
                ::bincode::Encode::encode(&self.category, encoder)?;
                ::bincode::Encode::encode(&self.related_tags, encoder)?;
                core::result::Result::Ok(())
            }
        }
        impl<__Context> ::bincode::Decode<__Context> for HashTag {
            fn decode<__D: ::bincode::de::Decoder<Context =
                __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        tag: ::bincode::Decode::decode(decoder)?,
                        created_at: ::bincode::Decode::decode(decoder)?,
                        usage_count: ::bincode::Decode::decode(decoder)?,
                        trending_score: ::bincode::Decode::decode(decoder)?,
                        last_used: (<::bincode::serde::Compat<_> as
                                            ::bincode::Decode<__Context>>::decode(decoder)?).0,
                        is_trending: ::bincode::Decode::decode(decoder)?,
                        category: ::bincode::Decode::decode(decoder)?,
                        related_tags: ::bincode::Decode::decode(decoder)?,
                    })
            }
        }
        impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
            HashTag {
            fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context
                = __Context>>(decoder: &mut __D)
                -> core::result::Result<Self, ::bincode::error::DecodeError> {
                core::result::Result::Ok(Self {
                        tag: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        created_at: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        usage_count: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        trending_score: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        last_used: (<::bincode::serde::BorrowCompat<_> as
                                            ::bincode::BorrowDecode<'_,
                                            __Context>>::borrow_decode(decoder)?).0,
                        is_trending: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        category: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                        related_tags: ::bincode::BorrowDecode::<'_,
                                    __Context>::borrow_decode(decoder)?,
                    })
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl _serde::Serialize for HashTag {
                    fn serialize<__S>(&self, __serializer: __S)
                        -> _serde::__private225::Result<__S::Ok, __S::Error> where
                        __S: _serde::Serializer {
                        let mut __serde_state =
                            _serde::Serializer::serialize_struct(__serializer,
                                    "HashTag", false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "tag", &self.tag)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "created_at", &self.created_at)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "usage_count", &self.usage_count)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "trending_score", &self.trending_score)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "last_used", &self.last_used)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "is_trending", &self.is_trending)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "category", &self.category)?;
                        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                "related_tags", &self.related_tags)?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes,
        unused_qualifications, clippy :: absolute_paths,)]
        const _: () =
            {
                #[allow(unused_extern_crates, clippy :: useless_attribute)]
                extern crate serde as _serde;
                ;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for HashTag {
                    fn deserialize<__D>(__deserializer: __D)
                        -> _serde::__private225::Result<Self, __D::Error> where
                        __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __field5,
                            __field6,
                            __field7,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    0u64 => _serde::__private225::Ok(__Field::__field0),
                                    1u64 => _serde::__private225::Ok(__Field::__field1),
                                    2u64 => _serde::__private225::Ok(__Field::__field2),
                                    3u64 => _serde::__private225::Ok(__Field::__field3),
                                    4u64 => _serde::__private225::Ok(__Field::__field4),
                                    5u64 => _serde::__private225::Ok(__Field::__field5),
                                    6u64 => _serde::__private225::Ok(__Field::__field6),
                                    7u64 => _serde::__private225::Ok(__Field::__field7),
                                    _ => _serde::__private225::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    "tag" => _serde::__private225::Ok(__Field::__field0),
                                    "created_at" => _serde::__private225::Ok(__Field::__field1),
                                    "usage_count" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    "trending_score" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    "last_used" => _serde::__private225::Ok(__Field::__field4),
                                    "is_trending" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    "category" => _serde::__private225::Ok(__Field::__field6),
                                    "related_tags" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                                -> _serde::__private225::Result<Self::Value, __E> where
                                __E: _serde::de::Error {
                                match __value {
                                    b"tag" => _serde::__private225::Ok(__Field::__field0),
                                    b"created_at" =>
                                        _serde::__private225::Ok(__Field::__field1),
                                    b"usage_count" =>
                                        _serde::__private225::Ok(__Field::__field2),
                                    b"trending_score" =>
                                        _serde::__private225::Ok(__Field::__field3),
                                    b"last_used" => _serde::__private225::Ok(__Field::__field4),
                                    b"is_trending" =>
                                        _serde::__private225::Ok(__Field::__field5),
                                    b"category" => _serde::__private225::Ok(__Field::__field6),
                                    b"related_tags" =>
                                        _serde::__private225::Ok(__Field::__field7),
                                    _ => { _serde::__private225::Ok(__Field::__ignore) }
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                                -> _serde::__private225::Result<Self, __D::Error> where
                                __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                    __FieldVisitor)
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private225::PhantomData<HashTag>,
                            lifetime: _serde::__private225::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = HashTag;
                            fn expecting(&self,
                                __formatter: &mut _serde::__private225::Formatter)
                                -> _serde::__private225::fmt::Result {
                                _serde::__private225::Formatter::write_str(__formatter,
                                    "struct HashTag")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(0usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field1 =
                                    match _serde::de::SeqAccess::next_element::<i64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(1usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field2 =
                                    match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(2usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field3 =
                                    match _serde::de::SeqAccess::next_element::<f64>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(3usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field4 =
                                    match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(4usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field5 =
                                    match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(5usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field6 =
                                    match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(6usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                let __field7 =
                                    match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)?
                                        {
                                        _serde::__private225::Some(__value) => __value,
                                        _serde::__private225::None =>
                                            return _serde::__private225::Err(_serde::de::Error::invalid_length(7usize,
                                                        &"struct HashTag with 8 elements")),
                                    };
                                _serde::__private225::Ok(HashTag {
                                        tag: __field0,
                                        created_at: __field1,
                                        usage_count: __field2,
                                        trending_score: __field3,
                                        last_used: __field4,
                                        is_trending: __field5,
                                        category: __field6,
                                        related_tags: __field7,
                                    })
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                                -> _serde::__private225::Result<Self::Value, __A::Error>
                                where __A: _serde::de::MapAccess<'de> {
                                let mut __field0: _serde::__private225::Option<String> =
                                    _serde::__private225::None;
                                let mut __field1: _serde::__private225::Option<i64> =
                                    _serde::__private225::None;
                                let mut __field2: _serde::__private225::Option<u64> =
                                    _serde::__private225::None;
                                let mut __field3: _serde::__private225::Option<f64> =
                                    _serde::__private225::None;
                                let mut __field4:
                                        _serde::__private225::Option<DateTime<Utc>> =
                                    _serde::__private225::None;
                                let mut __field5: _serde::__private225::Option<bool> =
                                    _serde::__private225::None;
                                let mut __field6:
                                        _serde::__private225::Option<Option<String>> =
                                    _serde::__private225::None;
                                let mut __field7:
                                        _serde::__private225::Option<Vec<String>> =
                                    _serde::__private225::None;
                                while let _serde::__private225::Some(__key) =
                                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private225::Option::is_some(&__field0) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("tag"));
                                            }
                                            __field0 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<String>(&mut __map)?);
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private225::Option::is_some(&__field1) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("created_at"));
                                            }
                                            __field1 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<i64>(&mut __map)?);
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private225::Option::is_some(&__field2) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("usage_count"));
                                            }
                                            __field2 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<u64>(&mut __map)?);
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private225::Option::is_some(&__field3) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("trending_score"));
                                            }
                                            __field3 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<f64>(&mut __map)?);
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private225::Option::is_some(&__field4) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("last_used"));
                                            }
                                            __field4 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)?);
                                        }
                                        __Field::__field5 => {
                                            if _serde::__private225::Option::is_some(&__field5) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("is_trending"));
                                            }
                                            __field5 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<bool>(&mut __map)?);
                                        }
                                        __Field::__field6 => {
                                            if _serde::__private225::Option::is_some(&__field6) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("category"));
                                            }
                                            __field6 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Option<String>>(&mut __map)?);
                                        }
                                        __Field::__field7 => {
                                            if _serde::__private225::Option::is_some(&__field7) {
                                                return _serde::__private225::Err(<__A::Error as
                                                                _serde::de::Error>::duplicate_field("related_tags"));
                                            }
                                            __field7 =
                                                _serde::__private225::Some(_serde::de::MapAccess::next_value::<Vec<String>>(&mut __map)?);
                                        }
                                        _ => {
                                            let _ =
                                                _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::__private225::Some(__field0) => __field0,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("tag")?,
                                    };
                                let __field1 =
                                    match __field1 {
                                        _serde::__private225::Some(__field1) => __field1,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("created_at")?,
                                    };
                                let __field2 =
                                    match __field2 {
                                        _serde::__private225::Some(__field2) => __field2,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("usage_count")?,
                                    };
                                let __field3 =
                                    match __field3 {
                                        _serde::__private225::Some(__field3) => __field3,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("trending_score")?,
                                    };
                                let __field4 =
                                    match __field4 {
                                        _serde::__private225::Some(__field4) => __field4,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("last_used")?,
                                    };
                                let __field5 =
                                    match __field5 {
                                        _serde::__private225::Some(__field5) => __field5,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("is_trending")?,
                                    };
                                let __field6 =
                                    match __field6 {
                                        _serde::__private225::Some(__field6) => __field6,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("category")?,
                                    };
                                let __field7 =
                                    match __field7 {
                                        _serde::__private225::Some(__field7) => __field7,
                                        _serde::__private225::None =>
                                            _serde::__private225::de::missing_field("related_tags")?,
                                    };
                                _serde::__private225::Ok(HashTag {
                                        tag: __field0,
                                        created_at: __field1,
                                        usage_count: __field2,
                                        trending_score: __field3,
                                        last_used: __field4,
                                        is_trending: __field5,
                                        category: __field6,
                                        related_tags: __field7,
                                    })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] =
                            &["tag", "created_at", "usage_count", "trending_score",
                                        "last_used", "is_trending", "category", "related_tags"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                            "HashTag", FIELDS,
                            __Visitor {
                                marker: _serde::__private225::PhantomData::<HashTag>,
                                lifetime: _serde::__private225::PhantomData,
                            })
                    }
                }
            };
        #[automatically_derived]
        impl ::core::fmt::Debug for HashTag {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["tag", "created_at", "usage_count", "trending_score",
                                "last_used", "is_trending", "category", "related_tags"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.tag, &self.created_at, &self.usage_count,
                                &self.trending_score, &self.last_used, &self.is_trending,
                                &self.category, &&self.related_tags];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "HashTag", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for HashTag {
            #[inline]
            fn clone(&self) -> HashTag {
                HashTag {
                    tag: ::core::clone::Clone::clone(&self.tag),
                    created_at: ::core::clone::Clone::clone(&self.created_at),
                    usage_count: ::core::clone::Clone::clone(&self.usage_count),
                    trending_score: ::core::clone::Clone::clone(&self.trending_score),
                    last_used: ::core::clone::Clone::clone(&self.last_used),
                    is_trending: ::core::clone::Clone::clone(&self.is_trending),
                    category: ::core::clone::Clone::clone(&self.category),
                    related_tags: ::core::clone::Clone::clone(&self.related_tags),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for HashTag { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for HashTag {
            #[inline]
            fn eq(&self, other: &HashTag) -> bool {
                self.created_at == other.created_at &&
                                            self.usage_count == other.usage_count &&
                                        self.trending_score == other.trending_score &&
                                    self.is_trending == other.is_trending &&
                                self.tag == other.tag && self.last_used == other.last_used
                        && self.category == other.category &&
                    self.related_tags == other.related_tags
            }
        }
    }
}
pub enum SocialMediaSchema {
    PrimitiveTest(v1::PrimitiveTest),
    TestUnit(v1::TestUnit),
    TestTuple(v1::TestTuple),
    User(v1::User),
    Post(v1::Post),
    Comment(v1::Comment),
    Media(v1::Media),
    Reaction(v1::Reaction),
    Notification(v1::Notification),
    UserStats(v1::UserStats),
    HashTag(v1::HashTag),
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::PrimitiveTest)> for
    SocialMediaSchema {
    #[inline]
    fn from(value: (v1::PrimitiveTest)) -> Self {
        SocialMediaSchema::PrimitiveTest(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::TestUnit)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::TestUnit)) -> Self {
        SocialMediaSchema::TestUnit(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::TestTuple)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::TestTuple)) -> Self {
        SocialMediaSchema::TestTuple(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::User)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::User)) -> Self { SocialMediaSchema::User(value) }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::Post)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::Post)) -> Self { SocialMediaSchema::Post(value) }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::Comment)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::Comment)) -> Self {
        SocialMediaSchema::Comment(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::Media)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::Media)) -> Self { SocialMediaSchema::Media(value) }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::Reaction)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::Reaction)) -> Self {
        SocialMediaSchema::Reaction(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::Notification)> for
    SocialMediaSchema {
    #[inline]
    fn from(value: (v1::Notification)) -> Self {
        SocialMediaSchema::Notification(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::UserStats)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::UserStats)) -> Self {
        SocialMediaSchema::UserStats(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl derive_more::core::convert::From<(v1::HashTag)> for SocialMediaSchema {
    #[inline]
    fn from(value: (v1::HashTag)) -> Self {
        SocialMediaSchema::HashTag(value)
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::Comment)
    {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::Comment(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Comment", "v1 :: Comment")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::User) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::User(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "User", "v1 :: User")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::Post) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::Post(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Post", "v1 :: Post")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::Media) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::Media(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Media", "v1 :: Media")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::TestUnit)
    {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::TestUnit(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "TestUnit", "v1 :: TestUnit")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for
    (v1::PrimitiveTest) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::PrimitiveTest(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "PrimitiveTest", "v1 :: PrimitiveTest")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::Reaction)
    {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::Reaction(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Reaction", "v1 :: Reaction")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for
    (v1::Notification) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::Notification(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Notification", "v1 :: Notification")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for
    (v1::TestTuple) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::TestTuple(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "TestTuple", "v1 :: TestTuple")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for
    (v1::UserStats) {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::UserStats(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "UserStats", "v1 :: UserStats")),
        }
    }
}
#[automatically_derived]
impl derive_more::core::convert::TryFrom<SocialMediaSchema> for (v1::HashTag)
    {
    type Error = derive_more::TryIntoError<SocialMediaSchema>;
    #[inline]
    fn try_from(value: SocialMediaSchema)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchema>> {
        match value {
            SocialMediaSchema::HashTag(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "HashTag", "v1 :: HashTag")),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for SocialMediaSchema {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            SocialMediaSchema::PrimitiveTest(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "PrimitiveTest", &__self_0),
            SocialMediaSchema::TestUnit(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "TestUnit", &__self_0),
            SocialMediaSchema::TestTuple(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "TestTuple", &__self_0),
            SocialMediaSchema::User(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "User",
                    &__self_0),
            SocialMediaSchema::Post(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Post",
                    &__self_0),
            SocialMediaSchema::Comment(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Comment", &__self_0),
            SocialMediaSchema::Media(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Media",
                    &__self_0),
            SocialMediaSchema::Reaction(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Reaction", &__self_0),
            SocialMediaSchema::Notification(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Notification", &__self_0),
            SocialMediaSchema::UserStats(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "UserStats", &__self_0),
            SocialMediaSchema::HashTag(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "HashTag", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SocialMediaSchema {
    #[inline]
    fn clone(&self) -> SocialMediaSchema {
        match self {
            SocialMediaSchema::PrimitiveTest(__self_0) =>
                SocialMediaSchema::PrimitiveTest(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::TestUnit(__self_0) =>
                SocialMediaSchema::TestUnit(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::TestTuple(__self_0) =>
                SocialMediaSchema::TestTuple(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::User(__self_0) =>
                SocialMediaSchema::User(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::Post(__self_0) =>
                SocialMediaSchema::Post(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::Comment(__self_0) =>
                SocialMediaSchema::Comment(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::Media(__self_0) =>
                SocialMediaSchema::Media(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::Reaction(__self_0) =>
                SocialMediaSchema::Reaction(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::Notification(__self_0) =>
                SocialMediaSchema::Notification(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::UserStats(__self_0) =>
                SocialMediaSchema::UserStats(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchema::HashTag(__self_0) =>
                SocialMediaSchema::HashTag(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for SocialMediaSchema {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::PrimitiveTest(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::TestUnit(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::TestTuple(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(2u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::User(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(3u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Post(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(4u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Comment(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(5u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Media(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(6u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Reaction(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(7u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Notification(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(8u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::UserStats(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(9u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::HashTag(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(10u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for SocialMediaSchema {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::PrimitiveTest {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::TestUnit {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            2u32 =>
                core::result::Result::Ok(Self::TestTuple {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            3u32 =>
                core::result::Result::Ok(Self::User {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            4u32 =>
                core::result::Result::Ok(Self::Post {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            5u32 =>
                core::result::Result::Ok(Self::Comment {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            6u32 =>
                core::result::Result::Ok(Self::Media {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            7u32 =>
                core::result::Result::Ok(Self::Reaction {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            8u32 =>
                core::result::Result::Ok(Self::Notification {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            9u32 =>
                core::result::Result::Ok(Self::UserStats {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            10u32 =>
                core::result::Result::Ok(Self::HashTag {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "SocialMediaSchema",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 10,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    SocialMediaSchema {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::PrimitiveTest {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::TestUnit {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            2u32 =>
                core::result::Result::Ok(Self::TestTuple {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            3u32 =>
                core::result::Result::Ok(Self::User {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            4u32 =>
                core::result::Result::Ok(Self::Post {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            5u32 =>
                core::result::Result::Ok(Self::Comment {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            6u32 =>
                core::result::Result::Ok(Self::Media {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            7u32 =>
                core::result::Result::Ok(Self::Reaction {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            8u32 =>
                core::result::Result::Ok(Self::Notification {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            9u32 =>
                core::result::Result::Ok(Self::UserStats {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            10u32 =>
                core::result::Result::Ok(Self::HashTag {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "SocialMediaSchema",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 10,
                            },
                    }),
        }
    }
}
pub enum SocialMediaSchemaRef<'a> {
    PrimitiveTest(&'a v1::PrimitiveTest),
    TestUnit(&'a v1::TestUnit),
    TestTuple(&'a v1::TestTuple),
    User(&'a v1::User),
    Post(&'a v1::Post),
    Comment(&'a v1::Comment),
    Media(&'a v1::Media),
    Reaction(&'a v1::Reaction),
    Notification(&'a v1::Notification),
    UserStats(&'a v1::UserStats),
    HashTag(&'a v1::HashTag),
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::PrimitiveTest)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::PrimitiveTest)) -> Self {
        SocialMediaSchemaRef::PrimitiveTest(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::TestUnit)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::TestUnit)) -> Self {
        SocialMediaSchemaRef::TestUnit(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::TestTuple)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::TestTuple)) -> Self {
        SocialMediaSchemaRef::TestTuple(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::User)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::User)) -> Self {
        SocialMediaSchemaRef::User(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::Post)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::Post)) -> Self {
        SocialMediaSchemaRef::Post(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::Comment)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::Comment)) -> Self {
        SocialMediaSchemaRef::Comment(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::Media)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::Media)) -> Self {
        SocialMediaSchemaRef::Media(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::Reaction)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::Reaction)) -> Self {
        SocialMediaSchemaRef::Reaction(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::Notification)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::Notification)) -> Self {
        SocialMediaSchemaRef::Notification(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::UserStats)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::UserStats)) -> Self {
        SocialMediaSchemaRef::UserStats(value)
    }
}
#[allow(unreachable_code)]
#[automatically_derived]
impl<'a> derive_more::core::convert::From<(&'a v1::HashTag)> for
    SocialMediaSchemaRef<'a> {
    #[inline]
    fn from(value: (&'a v1::HashTag)) -> Self {
        SocialMediaSchemaRef::HashTag(value)
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::Reaction) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::Reaction(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Reaction", "& 'a v1 :: Reaction")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::Notification) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::Notification(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Notification", "& 'a v1 :: Notification")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::TestUnit) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::TestUnit(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "TestUnit", "& 'a v1 :: TestUnit")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::TestTuple) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::TestTuple(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "TestTuple", "& 'a v1 :: TestTuple")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::Media) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::Media(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Media", "& 'a v1 :: Media")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::User) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::User(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "User", "& 'a v1 :: User")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::PrimitiveTest) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::PrimitiveTest(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "PrimitiveTest", "& 'a v1 :: PrimitiveTest")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::Post) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::Post(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Post", "& 'a v1 :: Post")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::Comment) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::Comment(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "Comment", "& 'a v1 :: Comment")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::HashTag) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::HashTag(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "HashTag", "& 'a v1 :: HashTag")),
        }
    }
}
#[automatically_derived]
impl<'a> derive_more::core::convert::TryFrom<SocialMediaSchemaRef<'a>> for
    (&'a v1::UserStats) {
    type Error = derive_more::TryIntoError<SocialMediaSchemaRef<'a>>;
    #[inline]
    fn try_from(value: SocialMediaSchemaRef<'a>)
        ->
            derive_more::core::result::Result<Self,
            derive_more::TryIntoError<SocialMediaSchemaRef<'a>>> {
        match value {
            SocialMediaSchemaRef::UserStats(__0) =>
                derive_more::core::result::Result::Ok(__0),
            _ =>
                derive_more::core::result::Result::Err(derive_more::TryIntoError::new(value,
                        "UserStats", "& 'a v1 :: UserStats")),
        }
    }
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for SocialMediaSchemaRef<'a> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            SocialMediaSchemaRef::PrimitiveTest(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "PrimitiveTest", &__self_0),
            SocialMediaSchemaRef::TestUnit(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "TestUnit", &__self_0),
            SocialMediaSchemaRef::TestTuple(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "TestTuple", &__self_0),
            SocialMediaSchemaRef::User(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "User",
                    &__self_0),
            SocialMediaSchemaRef::Post(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Post",
                    &__self_0),
            SocialMediaSchemaRef::Comment(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Comment", &__self_0),
            SocialMediaSchemaRef::Media(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Media",
                    &__self_0),
            SocialMediaSchemaRef::Reaction(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Reaction", &__self_0),
            SocialMediaSchemaRef::Notification(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Notification", &__self_0),
            SocialMediaSchemaRef::UserStats(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "UserStats", &__self_0),
            SocialMediaSchemaRef::HashTag(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "HashTag", &__self_0),
        }
    }
}
#[automatically_derived]
impl<'a> ::core::clone::Clone for SocialMediaSchemaRef<'a> {
    #[inline]
    fn clone(&self) -> SocialMediaSchemaRef<'a> {
        let _: ::core::clone::AssertParamIsClone<&'a v1::PrimitiveTest>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::TestUnit>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::TestTuple>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::User>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::Post>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::Comment>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::Media>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::Reaction>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::Notification>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::UserStats>;
        let _: ::core::clone::AssertParamIsClone<&'a v1::HashTag>;
        *self
    }
}
#[automatically_derived]
impl<'a> ::core::marker::Copy for SocialMediaSchemaRef<'a> { }
pub struct SocialMediaSchemaDBIter<'stack_db: 'db, 'db> {
    database: &'stack_db native_db::Database<'db>,
    r_scan: native_db::transaction::query::RScan<'db, 'stack_db>,
}
impl<'stack_db: 'db, 'db> SocialMediaSchemaDBIter<'stack_db, 'db> {
    pub fn new(database: &'stack_db native_db::Database<'db>)
        -> native_db::db_type::Result<Self> {
        let r_scan = database.r_transaction()?.scan::<'stack_db>();
        Ok(Self { database, r_scan })
    }
    pub fn scan_type_0(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::PrimitiveTest>> {
        self.r_scan.primary::<v1::PrimitiveTest>()?.all()
    }
    pub fn scan_type_1(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::TestUnit>> {
        self.r_scan.primary::<v1::TestUnit>()?.all()
    }
    pub fn scan_type_2(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::TestTuple>> {
        self.r_scan.primary::<v1::TestTuple>()?.all()
    }
    pub fn scan_type_3(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::User>> {
        self.r_scan.primary::<v1::User>()?.all()
    }
    pub fn scan_type_4(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::Post>> {
        self.r_scan.primary::<v1::Post>()?.all()
    }
    pub fn scan_type_5(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::Comment>> {
        self.r_scan.primary::<v1::Comment>()?.all()
    }
    pub fn scan_type_6(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::Media>> {
        self.r_scan.primary::<v1::Media>()?.all()
    }
    pub fn scan_type_7(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::Reaction>> {
        self.r_scan.primary::<v1::Reaction>()?.all()
    }
    pub fn scan_type_8(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::Notification>> {
        self.r_scan.primary::<v1::Notification>()?.all()
    }
    pub fn scan_type_9(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::UserStats>> {
        self.r_scan.primary::<v1::UserStats>()?.all()
    }
    pub fn scan_type_10(&'stack_db self)
        ->
            native_db::db_type::Result<native_db::transaction::query::PrimaryScanIterator<'db,
            v1::HashTag>> {
        self.r_scan.primary::<v1::HashTag>()?.all()
    }
}
pub enum PrimitiveTestKeys { Primary(String), }
#[automatically_derived]
impl ::core::fmt::Debug for PrimitiveTestKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PrimitiveTestKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PrimitiveTestKeys {
    #[inline]
    fn clone(&self) -> PrimitiveTestKeys {
        match self {
            PrimitiveTestKeys::Primary(__self_0) =>
                PrimitiveTestKeys::Primary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for PrimitiveTestKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for PrimitiveTestKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "PrimitiveTestKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    PrimitiveTestKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "PrimitiveTestKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
pub enum TestUnitKeys { Primary(String), }
#[automatically_derived]
impl ::core::fmt::Debug for TestUnitKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            TestUnitKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TestUnitKeys {
    #[inline]
    fn clone(&self) -> TestUnitKeys {
        match self {
            TestUnitKeys::Primary(__self_0) =>
                TestUnitKeys::Primary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for TestUnitKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for TestUnitKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "TestUnitKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    TestUnitKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "TestUnitKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
pub enum TestTupleKeys { Primary(String), }
#[automatically_derived]
impl ::core::fmt::Debug for TestTupleKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            TestTupleKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TestTupleKeys {
    #[inline]
    fn clone(&self) -> TestTupleKeys {
        match self {
            TestTupleKeys::Primary(__self_0) =>
                TestTupleKeys::Primary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for TestTupleKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for TestTupleKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "TestTupleKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    TestTupleKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "TestTupleKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
pub enum UserSecondaryKeys { username(String), email(String), }
#[automatically_derived]
impl ::core::fmt::Debug for UserSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            UserSecondaryKeys::username(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "username", &__self_0),
            UserSecondaryKeys::email(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "email",
                    &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UserSecondaryKeys {
    #[inline]
    fn clone(&self) -> UserSecondaryKeys {
        match self {
            UserSecondaryKeys::username(__self_0) =>
                UserSecondaryKeys::username(::core::clone::Clone::clone(__self_0)),
            UserSecondaryKeys::email(__self_0) =>
                UserSecondaryKeys::email(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for UserSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::username(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::email(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for UserSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::username {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::email {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    UserSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::username {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::email {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum UserKeys { Primary(String), Secondary(UserSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for UserKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            UserKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            UserKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UserKeys {
    #[inline]
    fn clone(&self) -> UserKeys {
        match self {
            UserKeys::Primary(__self_0) =>
                UserKeys::Primary(::core::clone::Clone::clone(__self_0)),
            UserKeys::Secondary(__self_0) =>
                UserKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for UserKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for UserKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for UserKeys
    {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum PostSecondaryKeys { user_id(String), created_at(i64), }
#[automatically_derived]
impl ::core::fmt::Debug for PostSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PostSecondaryKeys::user_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "user_id", &__self_0),
            PostSecondaryKeys::created_at(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "created_at", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PostSecondaryKeys {
    #[inline]
    fn clone(&self) -> PostSecondaryKeys {
        match self {
            PostSecondaryKeys::user_id(__self_0) =>
                PostSecondaryKeys::user_id(::core::clone::Clone::clone(__self_0)),
            PostSecondaryKeys::created_at(__self_0) =>
                PostSecondaryKeys::created_at(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for PostSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::user_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::created_at(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for PostSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "PostSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    PostSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "PostSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum PostKeys { Primary(String), Secondary(PostSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for PostKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PostKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            PostKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PostKeys {
    #[inline]
    fn clone(&self) -> PostKeys {
        match self {
            PostKeys::Primary(__self_0) =>
                PostKeys::Primary(::core::clone::Clone::clone(__self_0)),
            PostKeys::Secondary(__self_0) =>
                PostKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for PostKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for PostKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "PostKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for PostKeys
    {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "PostKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum CommentSecondaryKeys {
    post_id(String),
    user_id(String),
    created_at(i64),
}
#[automatically_derived]
impl ::core::fmt::Debug for CommentSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            CommentSecondaryKeys::post_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "post_id", &__self_0),
            CommentSecondaryKeys::user_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "user_id", &__self_0),
            CommentSecondaryKeys::created_at(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "created_at", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CommentSecondaryKeys {
    #[inline]
    fn clone(&self) -> CommentSecondaryKeys {
        match self {
            CommentSecondaryKeys::post_id(__self_0) =>
                CommentSecondaryKeys::post_id(::core::clone::Clone::clone(__self_0)),
            CommentSecondaryKeys::user_id(__self_0) =>
                CommentSecondaryKeys::user_id(::core::clone::Clone::clone(__self_0)),
            CommentSecondaryKeys::created_at(__self_0) =>
                CommentSecondaryKeys::created_at(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for CommentSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::post_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::user_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::created_at(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(2u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for CommentSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::post_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            2u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "CommentSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 2,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    CommentSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::post_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            2u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "CommentSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 2,
                            },
                    }),
        }
    }
}
pub enum CommentKeys { Primary(u64), Secondary(CommentSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for CommentKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            CommentKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            CommentKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CommentKeys {
    #[inline]
    fn clone(&self) -> CommentKeys {
        match self {
            CommentKeys::Primary(__self_0) =>
                CommentKeys::Primary(::core::clone::Clone::clone(__self_0)),
            CommentKeys::Secondary(__self_0) =>
                CommentKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for CommentKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for CommentKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "CommentKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    CommentKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "CommentKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum MediaSecondaryKeys { post_id(String), uploaded_at(i64), }
#[automatically_derived]
impl ::core::fmt::Debug for MediaSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            MediaSecondaryKeys::post_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "post_id", &__self_0),
            MediaSecondaryKeys::uploaded_at(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "uploaded_at", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for MediaSecondaryKeys {
    #[inline]
    fn clone(&self) -> MediaSecondaryKeys {
        match self {
            MediaSecondaryKeys::post_id(__self_0) =>
                MediaSecondaryKeys::post_id(::core::clone::Clone::clone(__self_0)),
            MediaSecondaryKeys::uploaded_at(__self_0) =>
                MediaSecondaryKeys::uploaded_at(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for MediaSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::post_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::uploaded_at(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for MediaSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::post_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::uploaded_at {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "MediaSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    MediaSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::post_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::uploaded_at {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "MediaSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum MediaKeys { Primary(String), Secondary(MediaSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for MediaKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            MediaKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            MediaKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for MediaKeys {
    #[inline]
    fn clone(&self) -> MediaKeys {
        match self {
            MediaKeys::Primary(__self_0) =>
                MediaKeys::Primary(::core::clone::Clone::clone(__self_0)),
            MediaKeys::Secondary(__self_0) =>
                MediaKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for MediaKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for MediaKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "MediaKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for MediaKeys
    {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "MediaKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum ReactionSecondaryKeys {
    user_id(String),
    target_id(String),
    created_at(i64),
}
#[automatically_derived]
impl ::core::fmt::Debug for ReactionSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ReactionSecondaryKeys::user_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "user_id", &__self_0),
            ReactionSecondaryKeys::target_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "target_id", &__self_0),
            ReactionSecondaryKeys::created_at(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "created_at", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ReactionSecondaryKeys {
    #[inline]
    fn clone(&self) -> ReactionSecondaryKeys {
        match self {
            ReactionSecondaryKeys::user_id(__self_0) =>
                ReactionSecondaryKeys::user_id(::core::clone::Clone::clone(__self_0)),
            ReactionSecondaryKeys::target_id(__self_0) =>
                ReactionSecondaryKeys::target_id(::core::clone::Clone::clone(__self_0)),
            ReactionSecondaryKeys::created_at(__self_0) =>
                ReactionSecondaryKeys::created_at(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for ReactionSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::user_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::target_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::created_at(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(2u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for ReactionSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::target_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            2u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "ReactionSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 2,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    ReactionSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::target_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            2u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "ReactionSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 2,
                            },
                    }),
        }
    }
}
pub enum ReactionKeys { Primary(String), Secondary(ReactionSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for ReactionKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ReactionKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            ReactionKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ReactionKeys {
    #[inline]
    fn clone(&self) -> ReactionKeys {
        match self {
            ReactionKeys::Primary(__self_0) =>
                ReactionKeys::Primary(::core::clone::Clone::clone(__self_0)),
            ReactionKeys::Secondary(__self_0) =>
                ReactionKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for ReactionKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for ReactionKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "ReactionKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    ReactionKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "ReactionKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum NotificationSecondaryKeys { user_id(String), created_at(i64), }
#[automatically_derived]
impl ::core::fmt::Debug for NotificationSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            NotificationSecondaryKeys::user_id(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "user_id", &__self_0),
            NotificationSecondaryKeys::created_at(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "created_at", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NotificationSecondaryKeys {
    #[inline]
    fn clone(&self) -> NotificationSecondaryKeys {
        match self {
            NotificationSecondaryKeys::user_id(__self_0) =>
                NotificationSecondaryKeys::user_id(::core::clone::Clone::clone(__self_0)),
            NotificationSecondaryKeys::created_at(__self_0) =>
                NotificationSecondaryKeys::created_at(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for NotificationSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::user_id(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::created_at(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for NotificationSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "NotificationSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    NotificationSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::user_id {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "NotificationSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum NotificationKeys {
    Primary(String),
    Secondary(NotificationSecondaryKeys),
}
#[automatically_derived]
impl ::core::fmt::Debug for NotificationKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            NotificationKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            NotificationKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NotificationKeys {
    #[inline]
    fn clone(&self) -> NotificationKeys {
        match self {
            NotificationKeys::Primary(__self_0) =>
                NotificationKeys::Primary(::core::clone::Clone::clone(__self_0)),
            NotificationKeys::Secondary(__self_0) =>
                NotificationKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for NotificationKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for NotificationKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "NotificationKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    NotificationKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "NotificationKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum UserStatsSecondaryKeys { date_timestamp(i64), }
#[automatically_derived]
impl ::core::fmt::Debug for UserStatsSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            UserStatsSecondaryKeys::date_timestamp(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "date_timestamp", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UserStatsSecondaryKeys {
    #[inline]
    fn clone(&self) -> UserStatsSecondaryKeys {
        match self {
            UserStatsSecondaryKeys::date_timestamp(__self_0) =>
                UserStatsSecondaryKeys::date_timestamp(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for UserStatsSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::date_timestamp(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for UserStatsSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::date_timestamp {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserStatsSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    UserStatsSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::date_timestamp {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserStatsSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
pub enum UserStatsKeys { Primary(String), Secondary(UserStatsSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for UserStatsKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            UserStatsKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            UserStatsKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UserStatsKeys {
    #[inline]
    fn clone(&self) -> UserStatsKeys {
        match self {
            UserStatsKeys::Primary(__self_0) =>
                UserStatsKeys::Primary(::core::clone::Clone::clone(__self_0)),
            UserStatsKeys::Secondary(__self_0) =>
                UserStatsKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for UserStatsKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for UserStatsKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserStatsKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    UserStatsKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "UserStatsKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum HashTagSecondaryKeys { created_at(i64), }
#[automatically_derived]
impl ::core::fmt::Debug for HashTagSecondaryKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            HashTagSecondaryKeys::created_at(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "created_at", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for HashTagSecondaryKeys {
    #[inline]
    fn clone(&self) -> HashTagSecondaryKeys {
        match self {
            HashTagSecondaryKeys::created_at(__self_0) =>
                HashTagSecondaryKeys::created_at(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for HashTagSecondaryKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::created_at(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for HashTagSecondaryKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "HashTagSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    HashTagSecondaryKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::created_at {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "HashTagSecondaryKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 0,
                            },
                    }),
        }
    }
}
pub enum HashTagKeys { Primary(String), Secondary(HashTagSecondaryKeys), }
#[automatically_derived]
impl ::core::fmt::Debug for HashTagKeys {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            HashTagKeys::Primary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Primary", &__self_0),
            HashTagKeys::Secondary(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Secondary", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for HashTagKeys {
    #[inline]
    fn clone(&self) -> HashTagKeys {
        match self {
            HashTagKeys::Primary(__self_0) =>
                HashTagKeys::Primary(::core::clone::Clone::clone(__self_0)),
            HashTagKeys::Secondary(__self_0) =>
                HashTagKeys::Secondary(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::bincode::Encode for HashTagKeys {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Primary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
            Self::Secondary(field_0) => {
                <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                ::bincode::Encode::encode(field_0, encoder)?;
                core::result::Result::Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for HashTagKeys {
    fn decode<__D: ::bincode::de::Decoder<Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "HashTagKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for
    HashTagKeys {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context =
        __Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let variant_index =
            <u32 as ::bincode::Decode<__D::Context>>::decode(decoder)?;
        match variant_index {
            0u32 =>
                core::result::Result::Ok(Self::Primary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            1u32 =>
                core::result::Result::Ok(Self::Secondary {
                        0: ::bincode::BorrowDecode::<__D::Context>::borrow_decode(decoder)?,
                    }),
            variant =>
                core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "HashTagKeys",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                                min: 0,
                                max: 1,
                            },
                    }),
        }
    }
}
pub enum SocialMediaSchemaKey {
    PrimitiveTest(PrimitiveTestKeys),
    TestUnit(TestUnitKeys),
    TestTuple(TestTupleKeys),
    User(UserKeys),
    Post(PostKeys),
    Comment(CommentKeys),
    Media(MediaKeys),
    Reaction(ReactionKeys),
    Notification(NotificationKeys),
    UserStats(UserStatsKeys),
    HashTag(HashTagKeys),
}
#[automatically_derived]
impl ::core::fmt::Debug for SocialMediaSchemaKey {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            SocialMediaSchemaKey::PrimitiveTest(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "PrimitiveTest", &__self_0),
            SocialMediaSchemaKey::TestUnit(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "TestUnit", &__self_0),
            SocialMediaSchemaKey::TestTuple(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "TestTuple", &__self_0),
            SocialMediaSchemaKey::User(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "User",
                    &__self_0),
            SocialMediaSchemaKey::Post(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Post",
                    &__self_0),
            SocialMediaSchemaKey::Comment(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Comment", &__self_0),
            SocialMediaSchemaKey::Media(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Media",
                    &__self_0),
            SocialMediaSchemaKey::Reaction(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Reaction", &__self_0),
            SocialMediaSchemaKey::Notification(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "Notification", &__self_0),
            SocialMediaSchemaKey::UserStats(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "UserStats", &__self_0),
            SocialMediaSchemaKey::HashTag(__self_0) =>
                ::core::fmt::Formatter::debug_tuple_field1_finish(f,
                    "HashTag", &__self_0),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SocialMediaSchemaKey {
    #[inline]
    fn clone(&self) -> SocialMediaSchemaKey {
        match self {
            SocialMediaSchemaKey::PrimitiveTest(__self_0) =>
                SocialMediaSchemaKey::PrimitiveTest(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::TestUnit(__self_0) =>
                SocialMediaSchemaKey::TestUnit(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::TestTuple(__self_0) =>
                SocialMediaSchemaKey::TestTuple(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::User(__self_0) =>
                SocialMediaSchemaKey::User(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::Post(__self_0) =>
                SocialMediaSchemaKey::Post(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::Comment(__self_0) =>
                SocialMediaSchemaKey::Comment(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::Media(__self_0) =>
                SocialMediaSchemaKey::Media(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::Reaction(__self_0) =>
                SocialMediaSchemaKey::Reaction(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::Notification(__self_0) =>
                SocialMediaSchemaKey::Notification(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::UserStats(__self_0) =>
                SocialMediaSchemaKey::UserStats(::core::clone::Clone::clone(__self_0)),
            SocialMediaSchemaKey::HashTag(__self_0) =>
                SocialMediaSchemaKey::HashTag(::core::clone::Clone::clone(__self_0)),
        }
    }
}
impl ::netabase::GetKey for v1::PrimitiveTest {
    type KeyType = PrimitiveTestKeys;
    fn key(&self) -> Self::KeyType {
        PrimitiveTestKeys::Primary(self.id.clone())
    }
}
impl ::netabase::RecordConversion for v1::PrimitiveTest {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            PrimitiveTestKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (PrimitiveTestKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::TestUnit {
    type KeyType = TestUnitKeys;
    fn key(&self) -> Self::KeyType { TestUnitKeys::Primary(self.id.clone()) }
}
impl ::netabase::RecordConversion for v1::TestUnit {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            TestUnitKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (TestUnitKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::TestTuple {
    type KeyType = TestTupleKeys;
    fn key(&self) -> Self::KeyType {
        TestTupleKeys::Primary(self.field_0.clone())
    }
}
impl ::netabase::RecordConversion for v1::TestTuple {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            TestTupleKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (TestTupleKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::User {
    type KeyType = UserKeys;
    fn key(&self) -> Self::KeyType { UserKeys::Primary(self.id.clone()) }
}
impl ::netabase::RecordConversion for v1::User {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            UserKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            UserKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (UserKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::Post {
    type KeyType = PostKeys;
    fn key(&self) -> Self::KeyType { PostKeys::Primary(self.id.clone()) }
}
impl ::netabase::RecordConversion for v1::Post {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            PostKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            PostKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (PostKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::Comment {
    type KeyType = CommentKeys;
    fn key(&self) -> Self::KeyType { CommentKeys::Primary(self.id.clone()) }
}
impl ::netabase::RecordConversion for v1::Comment {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            CommentKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            CommentKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (CommentKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::Media {
    type KeyType = MediaKeys;
    fn key(&self) -> Self::KeyType { MediaKeys::Primary(self.id.clone()) }
}
impl ::netabase::RecordConversion for v1::Media {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            MediaKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            MediaKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (MediaKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::Reaction {
    type KeyType = ReactionKeys;
    fn key(&self) -> Self::KeyType { ReactionKeys::Primary(self.id.clone()) }
}
impl ::netabase::RecordConversion for v1::Reaction {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            ReactionKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            ReactionKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (ReactionKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::Notification {
    type KeyType = NotificationKeys;
    fn key(&self) -> Self::KeyType {
        NotificationKeys::Primary(self.id.clone())
    }
}
impl ::netabase::RecordConversion for v1::Notification {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            NotificationKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            NotificationKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (NotificationKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::UserStats {
    type KeyType = UserStatsKeys;
    fn key(&self) -> Self::KeyType {
        UserStatsKeys::Primary(self.user_id.clone())
    }
}
impl ::netabase::RecordConversion for v1::UserStats {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            UserStatsKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            UserStatsKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (UserStatsKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for v1::HashTag {
    type KeyType = HashTagKeys;
    fn key(&self) -> Self::KeyType { HashTagKeys::Primary(self.tag.clone()) }
}
impl ::netabase::RecordConversion for v1::HashTag {
    fn calculate_expiry(&self) -> Option<std::time::Instant> { None }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            HashTagKeys::Primary(k) => {
                use native_db::ToKey;
                let native_key = k.to_key();
                ::netabase::native_db_key_to_bytes(&native_key)
            }
            HashTagKeys::Secondary(sk_enum) => {
                {
                    ::core::panicking::panic_fmt(format_args!("not yet implemented: {0}",
                            format_args!("Secondary key conversion not yet implemented")));
                }
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (HashTagKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(key)
    }
}
impl ::netabase::GetKey for SocialMediaSchema {
    type KeyType = SocialMediaSchemaKey;
    fn key(&self) -> Self::KeyType {
        match self {
            SocialMediaSchema::PrimitiveTest(data) => {
                SocialMediaSchemaKey::PrimitiveTest(data.key())
            }
            SocialMediaSchema::TestUnit(data) => {
                SocialMediaSchemaKey::TestUnit(data.key())
            }
            SocialMediaSchema::TestTuple(data) => {
                SocialMediaSchemaKey::TestTuple(data.key())
            }
            SocialMediaSchema::User(data) => {
                SocialMediaSchemaKey::User(data.key())
            }
            SocialMediaSchema::Post(data) => {
                SocialMediaSchemaKey::Post(data.key())
            }
            SocialMediaSchema::Comment(data) => {
                SocialMediaSchemaKey::Comment(data.key())
            }
            SocialMediaSchema::Media(data) => {
                SocialMediaSchemaKey::Media(data.key())
            }
            SocialMediaSchema::Reaction(data) => {
                SocialMediaSchemaKey::Reaction(data.key())
            }
            SocialMediaSchema::Notification(data) => {
                SocialMediaSchemaKey::Notification(data.key())
            }
            SocialMediaSchema::UserStats(data) => {
                SocialMediaSchemaKey::UserStats(data.key())
            }
            SocialMediaSchema::HashTag(data) => {
                SocialMediaSchemaKey::HashTag(data.key())
            }
        }
    }
}
impl ::netabase::RecordConversion for SocialMediaSchema {
    fn calculate_expiry(&self) -> Option<std::time::Instant> {
        match self {
            SocialMediaSchema::PrimitiveTest(data) => data.calculate_expiry(),
            SocialMediaSchema::TestUnit(data) => data.calculate_expiry(),
            SocialMediaSchema::TestTuple(data) => data.calculate_expiry(),
            SocialMediaSchema::User(data) => data.calculate_expiry(),
            SocialMediaSchema::Post(data) => data.calculate_expiry(),
            SocialMediaSchema::Comment(data) => data.calculate_expiry(),
            SocialMediaSchema::Media(data) => data.calculate_expiry(),
            SocialMediaSchema::Reaction(data) => data.calculate_expiry(),
            SocialMediaSchema::Notification(data) => data.calculate_expiry(),
            SocialMediaSchema::UserStats(data) => data.calculate_expiry(),
            SocialMediaSchema::HashTag(data) => data.calculate_expiry(),
        }
    }
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
        match key {
            SocialMediaSchemaKey::PrimitiveTest(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::TestUnit(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::TestTuple(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::User(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::Post(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::Comment(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::Media(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::Reaction(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::Notification(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::UserStats(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
            SocialMediaSchemaKey::HashTag(k) => {
                use ::bincode::Encode;
                ::bincode::encode_to_vec(k,
                        ::bincode::config::standard()).unwrap_or_default()
            }
        }
    }
    fn bytes_to_key(bytes: &[u8])
        -> Result<Self::KeyType, Box<dyn std::error::Error>> {
        use ::bincode::Decode;
        let (key, _): (PrimitiveTestKeys, usize) =
            ::bincode::decode_from_slice(bytes,
                    ::bincode::config::standard())?;
        Ok(SocialMediaSchemaKey::PrimitiveTest(key))
    }
}
impl ::bincode::Encode for SocialMediaSchemaKey {
    fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E)
        -> core::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            SocialMediaSchemaKey::PrimitiveTest(key) => {
                ::bincode::Encode::encode(&0u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::TestUnit(key) => {
                ::bincode::Encode::encode(&1u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::TestTuple(key) => {
                ::bincode::Encode::encode(&2u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::User(key) => {
                ::bincode::Encode::encode(&3u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::Post(key) => {
                ::bincode::Encode::encode(&4u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::Comment(key) => {
                ::bincode::Encode::encode(&5u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::Media(key) => {
                ::bincode::Encode::encode(&6u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::Reaction(key) => {
                ::bincode::Encode::encode(&7u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::Notification(key) => {
                ::bincode::Encode::encode(&8u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::UserStats(key) => {
                ::bincode::Encode::encode(&9u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
            SocialMediaSchemaKey::HashTag(key) => {
                ::bincode::Encode::encode(&10u8, encoder)?;
                ::bincode::Encode::encode(key, encoder)
            }
        }
    }
}
impl<Context> ::bincode::Decode<Context> for SocialMediaSchemaKey {
    fn decode<__D: ::bincode::de::Decoder<Context =
        Context>>(decoder: &mut __D)
        -> core::result::Result<Self, ::bincode::error::DecodeError> {
        let discriminant: u8 = ::bincode::Decode::decode(decoder)?;
        match discriminant {
            0u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::PrimitiveTest(key))
            }
            1u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::TestUnit(key))
            }
            2u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::TestTuple(key))
            }
            3u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::User(key))
            }
            4u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::Post(key))
            }
            5u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::Comment(key))
            }
            6u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::Media(key))
            }
            7u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::Reaction(key))
            }
            8u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::Notification(key))
            }
            9u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::UserStats(key))
            }
            10u8 => {
                let key = ::bincode::Decode::decode(decoder)?;
                Ok(SocialMediaSchemaKey::HashTag(key))
            }
            _ =>
                Err(::bincode::error::DecodeError::UnexpectedEnd {
                        additional: 0,
                    }),
        }
    }
}
impl<'a> From<&'a SocialMediaSchema> for SocialMediaSchemaRef<'a> {
    fn from(owned: &'a SocialMediaSchema) -> Self {
        match owned {
            SocialMediaSchema::PrimitiveTest(data) =>
                SocialMediaSchemaRef::PrimitiveTest(data),
            SocialMediaSchema::TestUnit(data) =>
                SocialMediaSchemaRef::TestUnit(data),
            SocialMediaSchema::TestTuple(data) =>
                SocialMediaSchemaRef::TestTuple(data),
            SocialMediaSchema::User(data) => SocialMediaSchemaRef::User(data),
            SocialMediaSchema::Post(data) => SocialMediaSchemaRef::Post(data),
            SocialMediaSchema::Comment(data) =>
                SocialMediaSchemaRef::Comment(data),
            SocialMediaSchema::Media(data) =>
                SocialMediaSchemaRef::Media(data),
            SocialMediaSchema::Reaction(data) =>
                SocialMediaSchemaRef::Reaction(data),
            SocialMediaSchema::Notification(data) =>
                SocialMediaSchemaRef::Notification(data),
            SocialMediaSchema::UserStats(data) =>
                SocialMediaSchemaRef::UserStats(data),
            SocialMediaSchema::HashTag(data) =>
                SocialMediaSchemaRef::HashTag(data),
        }
    }
}
impl<'a> From<SocialMediaSchemaRef<'a>> for
    ::netabase::Record<SocialMediaSchema> {
    fn from(ref_enum: SocialMediaSchemaRef<'a>) -> Self {
        match ref_enum {
            SocialMediaSchemaRef::PrimitiveTest(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::PrimitiveTest as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data =
                    SocialMediaSchema::PrimitiveTest(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::TestUnit(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::TestUnit as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::TestUnit(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::TestTuple(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::TestTuple as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::TestTuple(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::User(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::User as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::User(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::Post(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::Post as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::Post(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::Comment(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::Comment as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::Comment(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::Media(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::Media as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::Media(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::Reaction(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::Reaction as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::Reaction(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::Notification(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::Notification as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data =
                    SocialMediaSchema::Notification(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::UserStats(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::UserStats as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::UserStats(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
            SocialMediaSchemaRef::HashTag(data) => {
                use ::netabase::GetKey;
                let key = data.key();
                let key_bytes =
                    <v1::HashTag as
                            ::netabase::RecordConversion>::key_to_bytes(&key);
                let catalog_data = SocialMediaSchema::HashTag(data.clone());
                ::netabase::Record::new(key_bytes, catalog_data)
            }
        }
    }
}
impl<'a> ::netabase::FromNativeDb<'a> for SocialMediaSchemaRef<'a> {
    fn try_from_native_db<T: ::native_db::ToInput + 'a>(data: &'a T)
        -> Option<Self> where T: ::std::any::Any {
        let any_data = data as &dyn ::std::any::Any;
        if let Some(typed_data) = any_data.downcast_ref::<v1::PrimitiveTest>()
            {
            return Some(SocialMediaSchemaRef::PrimitiveTest(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::TestUnit>() {
            return Some(SocialMediaSchemaRef::TestUnit(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::TestTuple>() {
            return Some(SocialMediaSchemaRef::TestTuple(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::User>() {
            return Some(SocialMediaSchemaRef::User(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::Post>() {
            return Some(SocialMediaSchemaRef::Post(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::Comment>() {
            return Some(SocialMediaSchemaRef::Comment(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::Media>() {
            return Some(SocialMediaSchemaRef::Media(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::Reaction>() {
            return Some(SocialMediaSchemaRef::Reaction(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::Notification>()
            {
            return Some(SocialMediaSchemaRef::Notification(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::UserStats>() {
            return Some(SocialMediaSchemaRef::UserStats(typed_data));
        }
        if let Some(typed_data) = any_data.downcast_ref::<v1::HashTag>() {
            return Some(SocialMediaSchemaRef::HashTag(typed_data));
        }
        None
    }
}
impl ::netabase::CatalogConstructor<v1::PrimitiveTest> for SocialMediaSchema {
    fn from_native_db(data: v1::PrimitiveTest) -> Self {
        SocialMediaSchema::PrimitiveTest(data)
    }
    fn to_native_db(self) -> v1::PrimitiveTest {
        match self {
            SocialMediaSchema::PrimitiveTest(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: PrimitiveTest"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::TestUnit> for SocialMediaSchema {
    fn from_native_db(data: v1::TestUnit) -> Self {
        SocialMediaSchema::TestUnit(data)
    }
    fn to_native_db(self) -> v1::TestUnit {
        match self {
            SocialMediaSchema::TestUnit(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: TestUnit"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::TestTuple> for SocialMediaSchema {
    fn from_native_db(data: v1::TestTuple) -> Self {
        SocialMediaSchema::TestTuple(data)
    }
    fn to_native_db(self) -> v1::TestTuple {
        match self {
            SocialMediaSchema::TestTuple(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: TestTuple"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::User> for SocialMediaSchema {
    fn from_native_db(data: v1::User) -> Self {
        SocialMediaSchema::User(data)
    }
    fn to_native_db(self) -> v1::User {
        match self {
            SocialMediaSchema::User(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: User"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::Post> for SocialMediaSchema {
    fn from_native_db(data: v1::Post) -> Self {
        SocialMediaSchema::Post(data)
    }
    fn to_native_db(self) -> v1::Post {
        match self {
            SocialMediaSchema::Post(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: Post"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::Comment> for SocialMediaSchema {
    fn from_native_db(data: v1::Comment) -> Self {
        SocialMediaSchema::Comment(data)
    }
    fn to_native_db(self) -> v1::Comment {
        match self {
            SocialMediaSchema::Comment(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: Comment"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::Media> for SocialMediaSchema {
    fn from_native_db(data: v1::Media) -> Self {
        SocialMediaSchema::Media(data)
    }
    fn to_native_db(self) -> v1::Media {
        match self {
            SocialMediaSchema::Media(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: Media"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::Reaction> for SocialMediaSchema {
    fn from_native_db(data: v1::Reaction) -> Self {
        SocialMediaSchema::Reaction(data)
    }
    fn to_native_db(self) -> v1::Reaction {
        match self {
            SocialMediaSchema::Reaction(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: Reaction"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::Notification> for SocialMediaSchema {
    fn from_native_db(data: v1::Notification) -> Self {
        SocialMediaSchema::Notification(data)
    }
    fn to_native_db(self) -> v1::Notification {
        match self {
            SocialMediaSchema::Notification(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: Notification"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::UserStats> for SocialMediaSchema {
    fn from_native_db(data: v1::UserStats) -> Self {
        SocialMediaSchema::UserStats(data)
    }
    fn to_native_db(self) -> v1::UserStats {
        match self {
            SocialMediaSchema::UserStats(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: UserStats"));
            }
        }
    }
}
impl ::netabase::CatalogConstructor<v1::HashTag> for SocialMediaSchema {
    fn from_native_db(data: v1::HashTag) -> Self {
        SocialMediaSchema::HashTag(data)
    }
    fn to_native_db(self) -> v1::HashTag {
        match self {
            SocialMediaSchema::HashTag(data) => data,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Cannot convert {0} variant to {1}",
                        "SocialMediaSchema", "v1 :: HashTag"));
            }
        }
    }
}
