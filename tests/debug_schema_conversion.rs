//! Debug test to isolate the schema conversion issue
//!
//! This test recreates the exact conversion flow that happens during
//! network communication to identify where the conversion is failing.

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::{NetabaseModel as NetabaseModelTrait, NetabaseSchema};
use serde::{Deserialize, Serialize};

/// Test schema identical to MultiProcessSchema
#[netabase_schema_module(DebugSchema, DebugKeys)]
pub mod debug_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestDataKey)]
    pub struct TestData {
        #[key]
        pub id: u64,
        pub content: String,
    }
}

use debug_schema::{DebugSchema, TestData};

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn test_schema_conversion_debug() {
        println!("🔍 Debug Schema Conversion Test");
        println!("================================");

        // Step 1: Create test data
        let test_data = TestData {
            id: 123,
            content: "Hello World".to_string(),
        };
        println!("✅ Created test data: {:?}", test_data);

        // Step 2: Convert to schema (this happens in put_record)
        let schema = DebugSchema::from(test_data.clone());
        println!("✅ Converted to schema: {:?}", schema);

        // Step 3: Convert schema to record (this happens in to_record)
        let record = schema.to_record().expect("Failed to convert to record");
        println!("✅ Converted to record:");
        println!("   - Key: {:?}", record.key);
        println!("   - Value length: {} bytes", record.value.len());
        println!(
            "   - Value (hex): {}",
            hex::encode(&record.value[..std::cmp::min(32, record.value.len())])
        );

        // Step 4: Try to convert back from record (this is where it fails)
        println!("\n🔄 Testing conversion back from record...");
        match DebugSchema::from_record(record.clone()) {
            Ok(recovered_schema) => {
                println!("✅ Successfully converted back: {:?}", recovered_schema);

                // Verify data integrity
                if let DebugSchema::TestData(recovered_data) = recovered_schema {
                    if recovered_data == test_data {
                        println!("✅ Data integrity verified!");
                    } else {
                        println!("❌ Data mismatch!");
                        println!("   Original: {:?}", test_data);
                        println!("   Recovered: {:?}", recovered_data);
                    }
                } else {
                    println!("❌ Wrong schema variant!");
                }
            }
            Err(e) => {
                println!("❌ Conversion failed: {:?}", e);

                // Debug the raw data
                println!("\n🔍 Debug Information:");
                println!("   - Record value length: {}", record.value.len());
                println!(
                    "   - First 64 bytes (hex): {}",
                    hex::encode(&record.value[..std::cmp::min(64, record.value.len())])
                );

                // Try to decode as just the inner type
                println!("\n🧪 Testing direct TestData decode...");
                match bincode::decode_from_slice::<TestData, _>(
                    &record.value,
                    bincode::config::standard(),
                ) {
                    Ok((decoded_data, _)) => {
                        println!("✅ Direct TestData decode succeeded: {:?}", decoded_data);
                        println!("❌ This means the record contains TestData, not DebugSchema!");
                    }
                    Err(e) => {
                        println!("❌ Direct TestData decode also failed: {:?}", e);
                    }
                }

                // Try to decode as schema enum manually
                println!("\n🧪 Testing manual DebugSchema decode...");
                match bincode::decode_from_slice::<DebugSchema, _>(
                    &record.value,
                    bincode::config::standard(),
                ) {
                    Ok((decoded_schema, _)) => {
                        println!(
                            "✅ Manual DebugSchema decode succeeded: {:?}",
                            decoded_schema
                        );
                    }
                    Err(e) => {
                        println!("❌ Manual DebugSchema decode failed: {:?}", e);
                    }
                }
                panic!("Schema conversion failed: {:?}", e);
            }
        }

        // Step 5: Test what happens if we manually create a record with TestData
        println!("\n🧪 Testing what happens with raw TestData in record...");
        let raw_test_data_bytes = bincode::encode_to_vec(&test_data, bincode::config::standard())
            .expect("Failed to encode");
        let raw_record = libp2p::kad::Record {
            key: record.key.clone(),
            value: raw_test_data_bytes.clone(),
            publisher: None,
            expires: None,
        };

        println!("Raw TestData record:");
        println!("   - Value length: {} bytes", raw_record.value.len());
        println!(
            "   - Value (hex): {}",
            hex::encode(&raw_record.value[..std::cmp::min(32, raw_record.value.len())])
        );

        match DebugSchema::from_record(raw_record) {
            Ok(recovered) => println!(
                "✅ Raw TestData record converted to schema: {:?}",
                recovered
            ),
            Err(e) => println!("❌ Raw TestData record conversion failed: {:?}", e),
        }

        // Step 6: Compare the byte representations
        println!("\n📊 Byte Comparison:");
        let schema_bytes = bincode::encode_to_vec(&schema, bincode::config::standard())
            .expect("Failed to encode schema");
        println!("Schema bytes length: {}", schema_bytes.len());
        println!("TestData bytes length: {}", raw_test_data_bytes.len());
        println!(
            "Schema first 32 bytes: {}",
            hex::encode(&schema_bytes[..std::cmp::min(32, schema_bytes.len())])
        );
        println!(
            "TestData first 32 bytes: {}",
            hex::encode(&raw_test_data_bytes[..std::cmp::min(32, raw_test_data_bytes.len())])
        );

        if schema_bytes == raw_test_data_bytes {
            println!("❌ PROBLEM: Schema and TestData have identical serialization!");
            panic!("Schema and TestData should have different serialization!");
        } else {
            println!("✅ Schema and TestData have different serialization (as expected)");
        }
    }

    #[test]
    fn test_actual_multiprocess_record_flow() {
        println!("\n🔍 Testing Actual Multi-Process Record Flow");
        println!("===========================================");

        // Test the exact same schema used in multiprocess tests
        use crate::debug_schema::{DebugSchema, TestData};

        // Create data similar to SenderData
        let test_data = TestData {
            id: 456,
            content: "Multi-process test".to_string(),
        };

        println!("✅ Created test data: {:?}", test_data);

        // Step 1: Simulate put_record conversion (individual model -> schema)
        let schema = DebugSchema::from(test_data.clone());
        println!("✅ Converted individual model to schema: {:?}", schema);

        // Step 2: Simulate to_record conversion (schema -> Record)
        let record = schema
            .to_record()
            .expect("Failed to convert schema to record");
        println!("✅ Converted schema to record");
        println!("   - Key bytes: {}", hex::encode(record.key.to_vec()));
        println!("   - Value bytes: {}", hex::encode(&record.value));

        // Step 3: Simulate what happens in RecordStore::put (Record -> schema)
        // This is where the error occurs in the multiprocess tests
        println!("\n🔄 Simulating RecordStore::put conversion...");
        match DebugSchema::from_record(record.clone()) {
            Ok(recovered_schema) => {
                println!(
                    "✅ RecordStore conversion succeeded: {:?}",
                    recovered_schema
                );

                // Verify the data matches
                if let DebugSchema::TestData(recovered_data) = recovered_schema {
                    if recovered_data == test_data {
                        println!("✅ Data integrity maintained through full cycle!");
                    } else {
                        println!("❌ Data corruption detected!");
                        println!("   Original: {:?}", test_data);
                        println!("   Recovered: {:?}", recovered_data);
                    }
                } else {
                    println!("❌ Wrong schema variant recovered!");
                }
            }
            Err(e) => {
                println!("❌ RecordStore conversion failed: {:?}", e);

                // This would be the same error seen in multiprocess tests
                println!("\n🔍 Analyzing the failure...");
                println!("   - Error type: {:?}", e);
                println!("   - Record value length: {}", record.value.len());
                println!("   - Record value hex: {}", hex::encode(&record.value));

                // Check if it's a bincode deserialization issue
                println!("\n🧪 Testing bincode deserialization directly...");
                match bincode::decode_from_slice::<DebugSchema, _>(
                    &record.value,
                    bincode::config::standard(),
                ) {
                    Ok((decoded, _)) => {
                        println!("✅ Direct bincode decode worked: {:?}", decoded);
                        println!("❌ This means the trait implementation has an issue!");
                    }
                    Err(bincode_err) => {
                        println!("❌ Direct bincode decode failed: {:?}", bincode_err);
                        println!("❌ This confirms a serialization format mismatch!");
                    }
                }

                panic!("RecordStore conversion failed in test: {:?}", e);
            }
        }
    }

    #[test]
    fn test_record_store_trait_implementation() {
        println!("\n🔍 Testing RecordStore Trait Implementation");
        println!("==========================================");

        use crate::debug_schema::{DebugSchema, TestData};
        use netabase_store::database::sled::NetabaseSledDatabase;
        use netabase_store::traits::{NetabaseRecordStoreQuery, NetabaseSchema};

        // Create test data
        let test_data = TestData {
            id: 789,
            content: "RecordStore test".to_string(),
        };

        let schema = DebugSchema::from(test_data.clone());
        println!("✅ Created schema: {:?}", schema);

        // Test schema_to_record (used in RecordStore::get)
        println!("\n🔄 Testing schema_to_record...");
        match NetabaseSledDatabase::<DebugSchema>::schema_to_record(&schema) {
            Ok(record) => {
                println!("✅ schema_to_record succeeded");
                println!("   - Key: {}", hex::encode(record.key.to_vec()));
                println!("   - Value: {}", hex::encode(&record.value));

                // Test record_to_schema (used in RecordStore::put)
                println!("\n🔄 Testing record_to_schema...");
                match NetabaseSledDatabase::<DebugSchema>::record_to_schema(&record) {
                    Ok(recovered_schema) => {
                        println!("✅ record_to_schema succeeded: {:?}", recovered_schema);

                        // Verify roundtrip integrity
                        if let DebugSchema::TestData(recovered_data) = recovered_schema {
                            if recovered_data == test_data {
                                println!("✅ Full RecordStore roundtrip successful!");
                            } else {
                                println!("❌ Data corruption in roundtrip!");
                            }
                        } else {
                            println!("❌ Wrong schema variant in roundtrip!");
                        }
                    }
                    Err(e) => {
                        println!("❌ record_to_schema failed: {:?}", e);

                        // This would be the exact same error as in multiprocess tests
                        println!("\n🔍 This is the EXACT error from multiprocess tests!");
                        println!("   - Error: {:?}", e);

                        panic!("record_to_schema failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ schema_to_record failed: {:?}", e);
                panic!("schema_to_record failed: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_conversion_roundtrip() {
        let test_data = TestData {
            id: 456,
            content: "Test".to_string(),
        };

        // Convert to schema
        let schema = DebugSchema::from(test_data.clone());

        // Convert to record
        let record = schema.to_record().expect("Failed to convert to record");

        // Convert back from record
        let recovered_schema =
            DebugSchema::from_record(record).expect("Failed to convert from record");

        // Verify
        if let DebugSchema::TestData(recovered_data) = recovered_schema {
            assert_eq!(recovered_data, test_data);
        } else {
            panic!("Wrong schema variant");
        }
    }
}
