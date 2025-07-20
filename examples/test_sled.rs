//! Simple test to verify sled database creation and persistence

use sled::Db;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing sled database creation...");

    let test_dir = "./test_sled_db";

    // Remove test directory if it exists
    if Path::new(test_dir).exists() {
        std::fs::remove_dir_all(test_dir)?;
        println!("Removed existing test directory");
    }

    // Create directory
    std::fs::create_dir_all(test_dir)?;
    println!("Created test directory: {}", test_dir);

    // Test 1: Create database with directory path
    {
        println!("\nTest 1: Creating database in directory");
        let db_path = format!("{}/db1", test_dir);
        let db = sled::open(&db_path)?;

        // Insert some data
        db.insert("test_key", "test_value")?;
        println!("Inserted test data");

        // Flush to disk
        db.flush()?;
        println!("Flushed database");

        // Check if database files exist
        if Path::new(&db_path).exists() {
            println!("Database directory exists: {}", db_path);
            for entry in std::fs::read_dir(&db_path)? {
                let entry = entry?;
                println!("  File: {}", entry.file_name().to_string_lossy());
            }
        } else {
            println!("Database directory does NOT exist: {}", db_path);
        }

        drop(db);
        println!("Database dropped");
    }

    // Test 2: Reopen database and check persistence
    {
        println!("\nTest 2: Reopening database to check persistence");
        let db_path = format!("{}/db1", test_dir);
        let db = sled::open(&db_path)?;

        match db.get("test_key")? {
            Some(value) => {
                println!("Found persisted value: {}", String::from_utf8_lossy(&value));
            }
            None => {
                println!("No persisted value found!");
            }
        }

        drop(db);
    }

    // Test 3: Test with file path instead of directory
    {
        println!("\nTest 3: Creating database as file");
        let db_path = format!("{}/db2.sled", test_dir);
        let db = sled::open(&db_path)?;

        db.insert("file_key", "file_value")?;
        db.flush()?;

        if Path::new(&db_path).exists() {
            println!("Database file exists: {}", db_path);
        } else {
            println!("Database file does NOT exist: {}", db_path);
        }

        drop(db);

        // Reopen and check
        let db = sled::open(&db_path)?;
        match db.get("file_key")? {
            Some(value) => {
                println!(
                    "Found persisted file value: {}",
                    String::from_utf8_lossy(&value)
                );
            }
            None => {
                println!("No persisted file value found!");
            }
        }
    }

    println!("\nAll tests completed successfully!");
    Ok(())
}
