use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use serde::{Deserialize, Serialize};

#[netabase_schema_module(TestSchema, TestSchemaKey)]
pub mod test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub department: String,
        #[secondary_key]
        pub email: String,
    }
}

use test_schema::*;

fn debug_string_matching<SK>(model: &User, query_key: &SK) -> bool
where
    SK: std::fmt::Debug + PartialEq,
{
    let model_debug = format!("{:?}", model);
    let query_debug = format!("{:?}", query_key);

    println!("Model debug: {}", model_debug);
    println!("Query debug: {}", query_debug);

    // Extract the value from the secondary key enum variant
    if let Some(value_start) = query_debug.find('(') {
        if let Some(value_end) = query_debug.rfind(')') {
            let query_value = &query_debug[value_start + 1..value_end];
            println!("Raw extracted query value: '{}'", query_value);

            // Remove quotes if present
            let clean_query_value = query_value.trim_matches('"');
            println!("Clean query value: '{}'", clean_query_value);

            // Check if the model's debug representation contains this value
            let contains_result = model_debug.contains(clean_query_value);
            println!("Model contains clean query value: {}", contains_result);

            return contains_result;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_string_matching() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            department: "Engineering".to_string(),
            email: "alice@tech.com".to_string(),
        };

        // Test with the actual generated secondary key types
        let dept_key = UserSecondaryKeys::DepartmentKey("Engineering".to_string());
        let email_key = UserSecondaryKeys::EmailKey("alice@tech.com".to_string());
        let wrong_dept_key = UserSecondaryKeys::DepartmentKey("Marketing".to_string());

        println!("\n=== Testing Department Key ===");
        let dept_match = debug_string_matching(&user, &dept_key);
        println!("Department match result: {}\n", dept_match);

        println!("=== Testing Email Key ===");
        let email_match = debug_string_matching(&user, &email_key);
        println!("Email match result: {}\n", email_match);

        println!("=== Testing Wrong Department Key ===");
        let wrong_dept_match = debug_string_matching(&user, &wrong_dept_key);
        println!("Wrong department match result: {}\n", wrong_dept_match);

        // The test should pass if the logic is working correctly
        assert!(dept_match, "Should match department 'Engineering'");
        assert!(email_match, "Should match email 'alice@tech.com'");
        assert!(
            !wrong_dept_match,
            "Should not match wrong department 'Marketing'"
        );
    }

    #[test]
    fn test_manual_string_parsing() {
        // Test the string parsing logic manually
        let query_debug = r#"DepartmentKey("Engineering")"#;
        println!("Query debug string: {}", query_debug);

        if let Some(value_start) = query_debug.find('(') {
            if let Some(value_end) = query_debug.rfind(')') {
                let query_value = &query_debug[value_start + 1..value_end];
                println!("Raw extracted: '{}'", query_value);

                let clean_query_value = query_value.trim_matches('"');
                println!("After trim_matches: '{}'", clean_query_value);

                assert_eq!(clean_query_value, "Engineering");
            }
        }
    }
}
