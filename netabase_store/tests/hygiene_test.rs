//! Test to verify that Netabase macros are hygienic and don't require manual imports.
//!
//! This test intentionally does NOT import serde, bincode, strum, derive_more, or sled
//! to ensure that the macros work without requiring users to manually import dependencies.

use netabase_deps::{bincode, serde};
use netabase_macros::{NetabaseModel, netabase_schema_module}; // Re-exported for convenience

// Import necessary traits for the test
use netabase_store::traits::{
    NetabaseModel as NetabaseModelTrait, NetabaseModelKey, NetabaseSchema,
};

/// Test that basic NetabaseModel works with re-exported dependencies
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    PartialEq,
    bincode::Encode,
    bincode::Decode,
    serde::Serialize,
    serde::Deserialize,
)]
#[key_name(UserKey)]
pub struct User {
    #[key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub email: String,
    #[secondary_key]
    pub department: String,
    pub age: u32,
}

/// Test that schema modules work with re-exported dependencies
#[netabase_schema_module(TestSchema, TestKeys)]
mod test_schema {
    use super::*;
    // Import traits needed within the module
    use netabase_store::traits::{NetabaseModel as NetabaseModelTrait, NetabaseSchema};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(PersonKey)]
    pub struct Person {
        #[key]
        pub id: String,
        pub first_name: String,
        pub last_name: String,
        #[secondary_key]
        pub age: u32,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(CompanyKey)]
    pub struct Company {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub industry: String,
        pub founded_year: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase_store::traits::{NetabaseModel, NetabaseModelKey, NetabaseSchema};

    #[test]
    fn test_user_model_hygiene() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            department: "Engineering".to_string(),
            age: 30,
        };

        // Test that the generated key method works
        let key = user.key();
        match key {
            UserKey::Primary(primary_key) => {
                assert_eq!(primary_key.0, 1);
            }
            _ => panic!("Expected primary key"),
        }

        // Test that secondary keys work
        let secondary_keys = User::secondary_keys();
        assert_eq!(secondary_keys.len(), 2);

        // Test that tree name generation works
        assert_eq!(User::tree_name(), "User");
    }

    #[test]
    fn test_schema_module_hygiene() {
        use test_schema::*;

        let person = Person {
            id: "person1".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            age: 25,
        };

        let company = Company {
            id: 100,
            name: "Tech Corp".to_string(),
            industry: "Technology".to_string(),
            founded_year: 2010,
        };

        // Test that both models work in the schema
        let person_key = person.key();
        let company_key = company.key();

        // Test schema conversion
        let person_schema = TestSchema::Person(person.clone());
        let company_schema = TestSchema::Company(company.clone());

        // Verify discriminants work
        assert_ne!(person_schema.discriminant(), company_schema.discriminant());

        // Test that we can convert between schema and model types
        match person_schema {
            TestSchema::Person(p) => assert_eq!(p, person),
            _ => panic!("Expected person variant"),
        }

        match company_schema {
            TestSchema::Company(c) => assert_eq!(c, company),
            _ => panic!("Expected company variant"),
        }
    }

    #[test]
    fn test_serialization_hygiene() {
        // Test that serialization works with re-exported dependencies
        let user = User {
            id: 42,
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            department: "Sales".to_string(),
            age: 35,
        };

        // This should work because bincode serialization uses re-exported dependencies
        let key = user.key();

        // Test IVec conversion (which uses bincode internally)
        let ivec_result = std::convert::TryInto::<netabase_deps::sled::IVec>::try_into(key.clone());
        assert!(ivec_result.is_ok());

        let ivec = ivec_result.unwrap();
        let key_back_result = std::convert::TryInto::<UserKey>::try_into(ivec);
        assert!(key_back_result.is_ok());

        let key_back = key_back_result.unwrap();
        assert_eq!(key, key_back);
    }

    #[test]
    fn test_secondary_key_enum_hygiene() {
        // Test that secondary key enums work with re-exported strum
        let email_key = UserSecondaryKeys::EmailKey("test@example.com".to_string());
        let dept_key = UserSecondaryKeys::DepartmentKey("HR".to_string());

        // Test that enum iteration works (uses strum internally)
        let discriminants = UserKey::secondary_key_discriminants();
        assert!(!discriminants.is_empty());

        // Test that the enum variants can be created and used
        match email_key {
            UserSecondaryKeys::EmailKey(email) => assert_eq!(email, "test@example.com"),
            _ => panic!("Expected EmailKey variant"),
        }

        match dept_key {
            UserSecondaryKeys::DepartmentKey(dept) => assert_eq!(dept, "HR"),
            _ => panic!("Expected DepartmentKey variant"),
        }
    }

    #[test]
    fn test_no_secondary_keys_hygiene() {
        // Test a model with no secondary keys to ensure placeholder works
        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            bincode::Encode,
            bincode::Decode,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[key_name(SimpleKey)]
        pub struct SimpleModel {
            #[key]
            pub id: u32,
            pub data: String,
        }

        let model = SimpleModel {
            id: 123,
            data: "test".to_string(),
        };

        let key = model.key();
        match key {
            SimpleKey::Primary(pk) => assert_eq!(pk.0, 123),
            _ => panic!("Expected primary key"),
        }

        // Should have empty secondary keys
        let secondary_keys = SimpleModel::secondary_keys();
        assert!(secondary_keys.is_empty());
    }
}
