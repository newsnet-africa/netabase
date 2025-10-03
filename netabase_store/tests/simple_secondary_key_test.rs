use bincode::{Decode, Encode};
use log::{debug, info};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseAdvancedQuery, NetabaseModel, NetabaseSecondaryKeyQuery},
};
use serde::{Deserialize, Serialize};
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

#[netabase_schema_module(SimpleTestSchema, SimpleTestSchemaKey)]
pub mod test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
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
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ProductKey)]
    pub struct Product {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub category: String,
        #[secondary_key]
        pub in_stock: bool,
        pub price: f64,
        pub description: String,
    }
}

use test_schema::*;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_test_database() -> TestResult<(NetabaseSledDatabase<SimpleTestSchema>, TempDir)> {
    init_logger();
    info!("Creating test database for simple secondary key tests");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("simple_secondary_key_test_db");
    debug!("Test database path: {}", db_path.display());

    let db = NetabaseSledDatabase::new_with_path(&db_path.to_string_lossy())?;
    info!("Test database created successfully");

    Ok((db, temp_dir))
}

fn create_sample_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@tech.com".to_string(),
            department: "Engineering".to_string(),
            age: 28,
            created_at: 1234567890,
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@tech.com".to_string(),
            department: "Marketing".to_string(),
            age: 32,
            created_at: 1234567891,
        },
        User {
            id: 3,
            name: "Carol Davis".to_string(),
            email: "carol@tech.com".to_string(),
            department: "Engineering".to_string(),
            age: 29,
            created_at: 1234567892,
        },
        User {
            id: 4,
            name: "David Wilson".to_string(),
            email: "david@tech.com".to_string(),
            department: "Sales".to_string(),
            age: 35,
            created_at: 1234567893,
        },
    ]
}

fn create_sample_products() -> Vec<Product> {
    vec![
        Product {
            id: 1,
            name: "Laptop".to_string(),
            category: "Electronics".to_string(),
            in_stock: true,
            price: 999.99,
            description: "High-performance laptop".to_string(),
        },
        Product {
            id: 2,
            name: "Mouse".to_string(),
            category: "Electronics".to_string(),
            in_stock: false,
            price: 29.99,
            description: "Wireless mouse".to_string(),
        },
        Product {
            id: 3,
            name: "Book".to_string(),
            category: "Books".to_string(),
            in_stock: true,
            price: 19.99,
            description: "Programming guide".to_string(),
        },
        Product {
            id: 4,
            name: "Desk".to_string(),
            category: "Furniture".to_string(),
            in_stock: true,
            price: 299.99,
            description: "Standing desk".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_secondary_key_querying() -> TestResult<()> {
        info!("Starting test_basic_secondary_key_querying");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ {} users inserted", users.len());

        // Test querying by department (secondary key)
        debug!("Testing secondary key query by department");
        let engineering_users = user_tree
            .query_by_secondary_key(UserSecondaryKeys::DepartmentKey("Engineering".to_string()))?;

        debug!("Found {} engineering users", engineering_users.len());
        assert_eq!(engineering_users.len(), 2); // Alice and Carol

        for user in &engineering_users {
            assert_eq!(user.department, "Engineering");
            debug!("Engineering user: {} ({})", user.name, user.email);
        }
        info!("✓ Secondary key query by department successful");

        // Test querying by email (secondary key)
        debug!("Testing secondary key query by email");
        let alice_users = user_tree
            .query_by_secondary_key(UserSecondaryKeys::EmailKey("alice@tech.com".to_string()))?;

        assert_eq!(alice_users.len(), 1);
        assert_eq!(alice_users[0].name, "Alice Johnson");
        debug!("Found user by email: {}", alice_users[0].name);
        info!("✓ Secondary key query by email successful");

        info!("test_basic_secondary_key_querying completed successfully");
        Ok(())
    }

    #[test]
    fn test_product_secondary_keys() -> TestResult<()> {
        info!("Starting test_product_secondary_keys");

        let (db, _temp_dir) = create_test_database()?;
        let product_tree: NetabaseSledTree<Product, ProductKey> = db.get_main_tree()?;

        // Insert sample products
        let products = create_sample_products();
        for product in &products {
            product_tree.insert(product.key(), product.clone())?;
        }
        info!("✓ {} products inserted", products.len());

        // Test querying by category (secondary key)
        debug!("Testing secondary key query by category");
        let electronics = product_tree
            .query_by_secondary_key(ProductSecondaryKeys::CategoryKey("Electronics".to_string()))?;

        debug!("Found {} electronics", electronics.len());
        assert_eq!(electronics.len(), 2); // Laptop and Mouse

        for product in &electronics {
            assert_eq!(product.category, "Electronics");
            debug!(
                "Electronics product: {} (${:.2})",
                product.name, product.price
            );
        }
        info!("✓ Secondary key query by category successful");

        // Test querying by stock status (boolean secondary key)
        debug!("Testing secondary key query by stock status");
        let in_stock_products =
            product_tree.query_by_secondary_key(ProductSecondaryKeys::In_stockKey(true))?;

        debug!("Found {} in-stock products", in_stock_products.len());
        assert_eq!(in_stock_products.len(), 3); // All except Mouse

        for product in &in_stock_products {
            assert!(product.in_stock);
            debug!("In-stock product: {}", product.name);
        }
        info!("✓ Secondary key query by stock status successful");

        info!("test_product_secondary_keys completed successfully");
        Ok(())
    }

    #[test]
    fn test_advanced_querying_without_relations() -> TestResult<()> {
        info!("Starting test_advanced_querying_without_relations");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ {} users inserted", users.len());

        // Test custom filter query
        debug!("Testing custom filter query");
        let senior_users = user_tree.query_with_filter(|user| user.age >= 30)?;

        debug!("Found {} senior users", senior_users.len());
        assert_eq!(senior_users.len(), 2); // Bob and David

        for (_key, user) in &senior_users {
            assert!(user.age >= 30);
            debug!("Senior user: {} (age: {})", user.name, user.age);
        }
        info!("✓ Custom filter query successful");

        // Test count with condition
        debug!("Testing count with condition");
        let engineering_count = user_tree.count_where(|user| user.department == "Engineering")?;
        assert_eq!(engineering_count, 2);
        debug!("Engineering users count: {}", engineering_count);
        info!("✓ Count with condition successful");

        // Test range query by prefix
        debug!("Testing range query by prefix");
        let prefix = b""; // Empty prefix to get all
        let all_users = user_tree.range_by_prefix(prefix)?;

        assert_eq!(all_users.len(), users.len());
        debug!("Range query returned {} users", all_users.len());
        info!("✓ Range query by prefix successful");

        info!("test_advanced_querying_without_relations completed successfully");
        Ok(())
    }

    #[test]
    fn test_secondary_key_values_extraction() -> TestResult<()> {
        info!("Starting test_secondary_key_values_extraction");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ {} users inserted", users.len());

        // Test getting all secondary key values
        debug!("Testing secondary key value extraction");
        let department_values = user_tree.get_secondary_key_values("department")?;
        debug!("Found {} department values", department_values.len());
        assert!(!department_values.is_empty());
        info!("✓ Secondary key value extraction successful");

        info!("test_secondary_key_values_extraction completed successfully");
        Ok(())
    }

    #[test]
    fn test_batch_operations() -> TestResult<()> {
        info!("Starting test_batch_operations");

        let (db, _temp_dir) = create_test_database()?;
        let product_tree: NetabaseSledTree<Product, ProductKey> = db.get_main_tree()?;

        // Test batch insert with indexing
        debug!("Testing batch insert with indexing");
        let new_products = vec![
            (
                ProductKey::Primary(ProductPrimaryKey(5)),
                Product {
                    id: 5,
                    name: "Keyboard".to_string(),
                    category: "Electronics".to_string(),
                    in_stock: true,
                    price: 79.99,
                    description: "Mechanical keyboard".to_string(),
                },
            ),
            (
                ProductKey::Primary(ProductPrimaryKey(6)),
                Product {
                    id: 6,
                    name: "Chair".to_string(),
                    category: "Furniture".to_string(),
                    in_stock: false,
                    price: 199.99,
                    description: "Ergonomic chair".to_string(),
                },
            ),
        ];

        product_tree.batch_insert_with_indexing(new_products)?;
        assert_eq!(product_tree.len(), 2);
        debug!("Products after batch insert: {}", product_tree.len());
        info!("✓ Batch insert with indexing successful");

        // Test querying the newly inserted items
        debug!("Testing query after batch insert");
        let furniture_products = product_tree
            .query_by_secondary_key(ProductSecondaryKeys::CategoryKey("Furniture".to_string()))?;

        assert_eq!(furniture_products.len(), 1); // Just the chair
        assert_eq!(furniture_products[0].name, "Chair");
        debug!("Found furniture product: {}", furniture_products[0].name);
        info!("✓ Query after batch insert successful");

        info!("test_batch_operations completed successfully");
        Ok(())
    }

    #[test]
    fn test_database_level_operations() -> TestResult<()> {
        info!("Starting test_database_level_operations");

        let (db, _temp_dir) = create_test_database()?;

        // Test database-level secondary key indexing
        debug!("Testing database-level secondary key indexing");
        db.create_secondary_key_index::<User, UserKey, UserSecondaryKeys>("email")?;
        db.create_secondary_key_index::<User, UserKey, UserSecondaryKeys>("department")?;
        info!("✓ Database-level secondary key indexes created");

        // Insert sample data through the database
        let users = create_sample_users();
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }

        // Test database-level secondary key querying
        debug!("Testing database-level secondary key querying");
        let engineering_users = db.query_by_secondary_key::<User, UserKey, UserSecondaryKeys>(
            UserSecondaryKeys::DepartmentKey("Engineering".to_string()),
        )?;

        assert_eq!(engineering_users.len(), 2);
        debug!(
            "Database-level query found {} engineering users",
            engineering_users.len()
        );
        info!("✓ Database-level secondary key querying successful");

        info!("test_database_level_operations completed successfully");
        Ok(())
    }

    #[test]
    fn test_empty_secondary_key_results() -> TestResult<()> {
        info!("Starting test_empty_secondary_key_results");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ {} users inserted", users.len());

        // Test querying for non-existent secondary key values
        debug!("Testing query for non-existent department");
        let hr_users =
            user_tree.query_by_secondary_key(UserSecondaryKeys::DepartmentKey("HR".to_string()))?;

        assert!(hr_users.is_empty());
        debug!("Found {} HR users (expected 0)", hr_users.len());
        info!("✓ Empty secondary key query result successful");

        // Test querying for non-existent email
        debug!("Testing query for non-existent email");
        let ghost_users = user_tree
            .query_by_secondary_key(UserSecondaryKeys::EmailKey("ghost@nowhere.com".to_string()))?;

        assert!(ghost_users.is_empty());
        debug!("Found {} ghost users (expected 0)", ghost_users.len());
        info!("✓ Empty email query result successful");

        info!("test_empty_secondary_key_results completed successfully");
        Ok(())
    }
}
