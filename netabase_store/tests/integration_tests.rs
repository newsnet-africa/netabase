use bincode::{Decode, Encode};
use log::{debug, error, info, warn};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseModel, NetabaseSchema},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

// Test schema module with proper relational data
#[netabase_schema_module(BlogSchema, BlogSchemaKey)]
pub mod blog_schema {
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
        pub username: String,
        pub created_at: u64,
        // No direct relational fields - posts and profile reference user via foreign keys
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub category: String,
        #[secondary_key]
        pub published: bool,
        pub created_at: u64,
        pub tags: Vec<String>,
        // Relational fields - the macro will transform these
        pub author: UserLink,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CommentKey)]
    pub struct Comment {
        #[key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub post_id: u64,
        #[secondary_key]
        pub author_id: u64,
        pub created_at: u64,
        pub likes: u32,
        // Relational fields
        pub post: PostLink,
        pub author: UserLink,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ProfileKey)]
    pub struct Profile {
        #[key]
        pub id: u64,
        pub bio: String,
        #[secondary_key]
        pub user_id: u64,
        pub avatar_url: Option<String>,
        pub social_links: HashMap<String, String>,
        // Relational field
        pub user: UserLink,
    }
}

// E-commerce schema for testing complex relationships
#[netabase_schema_module(EcommerceSchema, EcommerceSchemaKey)]
pub mod ecommerce_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CustomerKey)]
    pub struct Customer {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub created_at: u64,
        // No direct relational fields - orders and addresses reference customer via foreign keys
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(OrderKey)]
    pub struct Order {
        #[key]
        pub id: u64,
        #[secondary_key]
        pub customer_id: u64,
        #[secondary_key]
        pub status: String,
        pub total_amount: f64,
        pub created_at: u64,
        // Relational fields
        pub customer: CustomerLink,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(OrderItemKey)]
    pub struct OrderItem {
        #[key]
        pub id: u64,
        #[secondary_key]
        pub order_id: u64,
        #[secondary_key]
        pub product_id: u64,
        pub quantity: u32,
        pub price: f64,
        // Relational fields
        pub order: OrderLink,
        pub product: ProductLink,
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
        // This model has no relations to test empty relations enum
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(AddressKey)]
    pub struct Address {
        #[key]
        pub id: u64,
        #[secondary_key]
        pub customer_id: u64,
        pub street: String,
        pub city: String,
        pub state: String,
        pub zip_code: String,
        pub country: String,
        // Relational field
        pub customer: CustomerLink,
    }
}

// Re-export for easier access in tests
use blog_schema::*;
use ecommerce_schema::*;

// Integration test suite
#[cfg(test)]
mod integration_tests {
    use super::*;
    use anyhow::Result;

    fn create_blog_database() -> Result<(NetabaseSledDatabase<BlogSchema>, TempDir)> {
        init_logger();
        info!("Creating blog database in temporary directory");

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("blog_test_db");
        debug!("Blog database path: {}", db_path.display());

        let db = NetabaseSledDatabase::new_with_path(&db_path)?;
        info!("Blog database created successfully");

        Ok((db, temp_dir))
    }

    fn create_ecommerce_database() -> Result<(NetabaseSledDatabase<EcommerceSchema>, TempDir)> {
        init_logger();
        info!("Creating ecommerce database in temporary directory");

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("ecommerce_test_db");
        debug!("Ecommerce database path: {}", db_path.display());

        let db = NetabaseSledDatabase::new_with_path(&db_path)?;
        info!("Ecommerce database created successfully");

        Ok((db, temp_dir))
    }

    fn create_sample_user(id: u64) -> User {
        User {
            id,
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
            username: format!("user_{}", id),
            created_at: 1234567890,
        }
    }

    fn create_sample_post(id: u64, author_id: u64) -> Post {
        Post {
            id,
            title: format!("Post {}", id),
            content: format!("This is the content of post {}", id),
            author_id,
            category: "tech".to_string(),
            published: true,
            created_at: 1234567891,
            tags: vec!["rust".to_string(), "database".to_string()],
            // Using generated type alias
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(author_id))),
        }
    }

    fn create_sample_comment(id: u64, post_id: u64, author_id: u64) -> Comment {
        Comment {
            id,
            content: format!("This is comment {}", id),
            post_id,
            author_id,
            created_at: 1234567892,
            likes: 0,
            // Using generated type aliases
            post: PostLink::from_key(PostKey::Primary(PostPrimaryKey(post_id))),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(author_id))),
        }
    }

    fn create_sample_profile(id: u64, user_id: u64) -> Profile {
        Profile {
            id,
            bio: format!("Biography for user {}", user_id),
            user_id,
            avatar_url: Some(format!("https://avatar.example.com/{}.jpg", user_id)),
            social_links: HashMap::from([
                ("twitter".to_string(), format!("@user_{}", user_id)),
                ("github".to_string(), format!("user_{}", user_id)),
            ]),
            // Using generated type alias
            user: UserLink::from_key(UserKey::Primary(UserPrimaryKey(user_id))),
        }
    }

    #[test]
    fn test_database_initialization() -> Result<()> {
        info!("Starting test_database_initialization");

        let (db, _temp_dir) = create_blog_database()?;
        info!("Database created successfully for initialization test");

        // Test that database was created successfully
        let was_recovered = db.db().was_recovered();
        debug!("Database was_recovered: {}", was_recovered);
        assert!(!was_recovered);
        info!("✓ Database initialization check passed");

        // Test tree name generation
        let tree_names = db.tree_names();
        debug!("Tree names: {:?}", tree_names);
        assert!(!tree_names.is_empty());
        info!("✓ Tree name generation check passed");

        info!("test_database_initialization completed successfully");
        Ok(())
    }

    #[test]
    fn test_model_specific_trees() -> Result<()> {
        info!("Starting test_model_specific_trees");

        let (db, _temp_dir) = create_blog_database()?;
        info!("Database created successfully for model specific trees test");

        // Test that each model can create its own tree using the new type-safe API
        debug!("Creating User tree");
        let user_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::User,
            blog_schema::UserKey,
        > = db.get_main_tree()?;
        assert_eq!(user_tree.len(), 0);
        info!("✓ User tree created and verified empty");

        debug!("Creating Post tree");
        let post_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Post,
            blog_schema::PostKey,
        > = db.get_main_tree()?;
        assert_eq!(post_tree.len(), 0);
        info!("✓ Post tree created and verified empty");

        debug!("Creating Comment tree");
        let comment_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Comment,
            blog_schema::CommentKey,
        > = db.get_main_tree()?;
        assert_eq!(comment_tree.len(), 0);
        info!("✓ Comment tree created and verified empty");

        debug!("Creating Profile tree");
        let profile_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Profile,
            blog_schema::ProfileKey,
        > = db.get_main_tree()?;
        assert_eq!(profile_tree.len(), 0);
        info!("✓ Profile tree created and verified empty");

        info!("test_model_specific_trees completed successfully");
        Ok(())
    }

    #[test]
    fn test_relational_link_functionality() -> Result<()> {
        info!("Starting test_relational_link_functionality");

        let user = create_sample_user(1);
        debug!("Created sample user with id: {}", user.id);

        let post = create_sample_post(1, user.id);
        debug!(
            "Created sample post with id: {} and author_id: {}",
            post.id, post.author_id
        );

        // Test that the macro transformed the fields correctly
        debug!("Testing initial relational link state");
        assert!(post.author.is_unresolved());
        info!("✓ Author link is initially unresolved as expected");

        assert_eq!(post.author.key(), Some(&user.key()));
        info!("✓ Author link key matches user key");

        // Test resolving the author link
        debug!("Testing relational link resolution");
        let resolved_author = post.author.clone().resolve(user.clone());
        assert!(resolved_author.is_resolved());
        info!("✓ Author link resolved successfully");

        assert_eq!(resolved_author.object().unwrap().id, user.id);
        info!("✓ Resolved author object matches expected user");

        info!("test_relational_link_functionality completed successfully");
        Ok(())
    }

    #[test]
    fn test_storing_and_loading_relational_data() -> Result<()> {
        info!("Starting test_storing_and_loading_relational_data");

        let (db, _temp_dir) = create_blog_database()?;
        info!("Database created successfully for storing and loading test");

        // Create trees using the new type-safe API
        debug!("Creating type-safe trees");
        let user_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::User,
            blog_schema::UserKey,
        > = db.get_main_tree()?;
        let post_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Post,
            blog_schema::PostKey,
        > = db.get_main_tree()?;
        let profile_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Profile,
            blog_schema::ProfileKey,
        > = db.get_main_tree()?;
        info!("✓ All trees created successfully");

        // Create sample data
        debug!("Creating sample data entities");
        let user = create_sample_user(1);
        let profile = create_sample_profile(1, user.id);
        let post1 = create_sample_post(1, user.id);
        let post2 = create_sample_post(2, user.id);
        info!("✓ Sample data created (1 user, 1 profile, 2 posts)");

        // Store all entities
        debug!("Storing entities in database");
        user_tree.insert(user.key(), user.clone())?;
        debug!("User stored with id: {}", user.id);

        profile_tree.insert(profile.key(), profile.clone())?;
        debug!("Profile stored with id: {}", profile.id);

        post_tree.insert(post1.key(), post1.clone())?;
        debug!("Post1 stored with id: {}", post1.id);

        post_tree.insert(post2.key(), post2.clone())?;
        debug!("Post2 stored with id: {}", post2.id);
        info!("✓ All entities stored successfully");

        // Load and verify data storage
        debug!("Loading and verifying stored data");
        let loaded_user = user_tree.get(user.key())?.unwrap();
        assert_eq!(loaded_user.id, user.id);
        assert_eq!(loaded_user.name, user.name);
        info!("✓ User loaded and verified successfully");

        let loaded_profile = profile_tree.get(profile.key())?.unwrap();
        assert_eq!(loaded_profile.id, profile.id);
        assert_eq!(loaded_profile.user_id, user.id);
        info!("✓ Profile loaded and verified successfully");

        let loaded_post1 = post_tree.get(post1.key())?.unwrap();
        assert_eq!(loaded_post1.id, post1.id);
        assert_eq!(loaded_post1.author_id, user.id);
        info!("✓ Post1 loaded and verified successfully");

        info!("test_storing_and_loading_relational_data completed successfully");
        Ok(())
    }

    #[test]
    fn test_resolving_relational_links() -> Result<()> {
        info!("Starting test_resolving_relational_links");

        let (db, _temp_dir) = create_blog_database()?;
        info!("Database created successfully for resolving relational links test");

        // Create trees using the new type-safe API
        debug!("Creating type-safe trees for resolution test");
        let user_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::User,
            blog_schema::UserKey,
        > = db.get_main_tree()?;
        let post_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Post,
            blog_schema::PostKey,
        > = db.get_main_tree()?;
        let comment_tree: netabase_store::database::NetabaseSledTree<
            blog_schema::Comment,
            blog_schema::CommentKey,
        > = db.get_main_tree()?;
        info!("✓ All trees created successfully");

        // Create and store test data
        debug!("Creating test entities for resolution");
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);
        let comment = create_sample_comment(1, post.id, user.id);
        info!("✓ Test entities created (user, post, comment)");

        debug!("Storing test entities in database");
        user_tree.insert(user.key(), user.clone())?;
        debug!("User stored with id: {}", user.id);

        post_tree.insert(post.key(), post.clone())?;
        debug!("Post stored with id: {}", post.id);

        comment_tree.insert(comment.key(), comment.clone())?;
        debug!("Comment stored with id: {}", comment.id);
        info!("✓ All entities stored successfully");

        // Load post and resolve its author relation
        debug!("Loading post for author resolution");
        let mut loaded_post = post_tree.get(post.key())?.unwrap();
        assert!(loaded_post.author.is_unresolved());
        info!("✓ Post loaded with unresolved author link as expected");

        // Resolve the author link in-place (improved pattern)
        debug!("Resolving post author relation");
        let author_key = loaded_post.author.key().unwrap();
        let author = user_tree.get(author_key.clone())?.unwrap();
        {
            let resolved_author_ref = loaded_post.author.resolve_mut(author.clone());
            // Verify reference points to correct data
            assert_eq!(resolved_author_ref.id, user.id);
            debug!("Author resolved with id: {}", resolved_author_ref.id);
        }

        // Verify the post's author field is now resolved (after reference is out of scope)
        assert!(loaded_post.author.is_resolved());
        assert_eq!(loaded_post.author.object().unwrap().id, user.id);
        info!("✓ Post author relation resolved and verified successfully");

        // Load comment and resolve its relations
        debug!("Loading comment for multiple relation resolution");
        let mut loaded_comment = comment_tree.get(comment.key())?.unwrap();

        // Resolve comment's post relation in-place (improved pattern)
        debug!("Resolving comment post relation");
        let comment_post_key = loaded_comment.post.key().unwrap();
        let comment_post = post_tree.get(comment_post_key.clone())?.unwrap();
        {
            let resolved_post_ref = loaded_comment.post.resolve_mut(comment_post);
            // Verify reference points to correct data
            assert_eq!(resolved_post_ref.id, post.id);
            debug!("Comment post resolved with id: {}", resolved_post_ref.id);
        }

        // Verify the comment's post field is now resolved (after reference is out of scope)
        assert!(loaded_comment.post.is_resolved());
        assert_eq!(loaded_comment.post.object().unwrap().id, post.id);
        info!("✓ Comment post relation resolved and verified successfully");

        info!("test_resolving_relational_links completed successfully");
        Ok(())
    }

    #[test]
    fn test_empty_relations() -> Result<()> {
        info!("Starting test_empty_relations");
        // Test that relation discriminants are properly generated

        // Test User relations - User has no direct relational fields
        debug!("Testing User relations");
        let user_relations: Vec<&str> = User::relations();
        assert_eq!(user_relations.len(), 0);
        info!("✓ User relations correctly empty");

        // Test Post relations - Post has relational fields but relations() returns empty (new behavior)
        debug!("Testing Post relations");
        let post_relations: Vec<&str> = Post::relations();
        assert_eq!(post_relations.len(), 0);
        info!("✓ Post relations correctly empty (new behavior)");

        // Test Comment relations - Comment has relational fields but relations() returns empty (new behavior)
        debug!("Testing Comment relations");
        let comment_relations: Vec<&str> = Comment::relations();
        assert_eq!(comment_relations.len(), 0);
        info!("✓ Comment relations correctly empty (new behavior)");

        // Test Profile relations - Profile has relational fields but relations() returns empty (new behavior)
        debug!("Testing Profile relations");
        let profile_relations: Vec<&str> = Profile::relations();
        assert_eq!(profile_relations.len(), 0);
        info!("✓ Profile relations correctly empty (new behavior)");

        // Test ecommerce relations - Customer has no direct relational fields
        debug!("Testing Customer relations");
        let customer_relations: Vec<&str> = Customer::relations();
        assert_eq!(customer_relations.len(), 0);
        info!("✓ Customer relations correctly empty");

        info!("test_empty_relations completed successfully");
        Ok(())
    }

    #[test]
    fn test_empty_relations_with_schema() -> Result<()> {
        // Test models with no relations (like Product)
        let product_relations: Vec<&str> = Product::relations();
        assert!(product_relations.is_empty());

        // Test that we can still create the model without issues
        let product = Product {
            id: 1,
            name: "Test Product".to_string(),
            category: "Test".to_string(),
            in_stock: true,
            price: 10.0,
            description: "Test description".to_string(),
        };

        // Should be able to encode/decode without problems
        let encoded = bincode::encode_to_vec(&product, bincode::config::standard())?;
        let (decoded, _): (Product, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard())?;

        assert_eq!(product, decoded);

        Ok(())
    }

    #[test]
    fn test_complex_ecommerce_relations() -> Result<()> {
        let (db, _temp_dir) = create_ecommerce_database()?;

        // Create trees using the new type-safe API
        let customer_tree: netabase_store::database::NetabaseSledTree<
            ecommerce_schema::Customer,
            ecommerce_schema::CustomerKey,
        > = db.get_main_tree()?;
        let order_tree: netabase_store::database::NetabaseSledTree<
            ecommerce_schema::Order,
            ecommerce_schema::OrderKey,
        > = db.get_main_tree()?;
        let order_item_tree: netabase_store::database::NetabaseSledTree<
            ecommerce_schema::OrderItem,
            ecommerce_schema::OrderItemKey,
        > = db.get_main_tree()?;
        let product_tree: NetabaseSledTree<
            ecommerce_schema::Product,
            ecommerce_schema::ProductKey,
        > = db.get_main_tree()?;
        let address_tree: NetabaseSledTree<
            ecommerce_schema::Address,
            ecommerce_schema::AddressKey,
        > = db.get_main_tree()?;

        // Create test data
        let customer = Customer {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: 1234567890,
        };

        let product1 = Product {
            id: 1,
            name: "Laptop".to_string(),
            category: "Electronics".to_string(),
            in_stock: true,
            price: 999.99,
            description: "High-performance laptop".to_string(),
        };

        let product2 = Product {
            id: 2,
            name: "Mouse".to_string(),
            category: "Electronics".to_string(),
            in_stock: true,
            price: 29.99,
            description: "Wireless mouse".to_string(),
        };

        let order = Order {
            id: 1,
            customer_id: customer.id,
            status: "pending".to_string(),
            total_amount: 1059.97,
            created_at: 1234567891,
            customer: CustomerLink::from_key(customer.key()),
        };

        let order_item1 = OrderItem {
            id: 1,
            order_id: order.id,
            product_id: product1.id,
            quantity: 1,
            price: product1.price,
            order: OrderLink::from_key(order.key()),
            product: ProductLink::from_key(product1.key()),
        };

        let order_item2 = OrderItem {
            id: 2,
            order_id: order.id,
            product_id: product2.id,
            quantity: 2,
            price: product2.price,
            order: OrderLink::from_key(order.key()),
            product: ProductLink::from_key(product2.key()),
        };

        let address = Address {
            id: 1,
            customer_id: customer.id,
            street: "123 Main St".to_string(),
            city: "Anytown".to_string(),
            state: "CA".to_string(),
            zip_code: "12345".to_string(),
            country: "USA".to_string(),
            customer: CustomerLink::from_key(customer.key()),
        };

        // Store all entities
        customer_tree.insert(customer.key(), customer.clone())?;
        order_tree.insert(order.key(), order.clone())?;
        order_item_tree.insert(order_item1.key(), order_item1.clone())?;
        order_item_tree.insert(order_item2.key(), order_item2.clone())?;
        product_tree.insert(product1.key(), product1.clone())?;
        product_tree.insert(product2.key(), product2.clone())?;
        address_tree.insert(address.key(), address.clone())?;

        // Load and verify the complex relationship
        let loaded_customer = customer_tree.get(customer.key())?.unwrap();
        assert_eq!(loaded_customer.id, 1);
        assert_eq!(loaded_customer.name, "John Doe");

        // Load order and verify customer relation
        let loaded_order = order_tree.get(order.key())?.unwrap();
        assert_eq!(loaded_order.customer_id, customer.id);
        assert!(loaded_order.customer.is_unresolved());

        // Resolve customer relation
        let customer_key = loaded_order.customer.key().unwrap();
        let order_customer = customer_tree.get(customer_key.clone())?.unwrap();
        let resolved_customer = loaded_order.customer.clone().resolve(order_customer);
        assert!(resolved_customer.is_resolved());
        assert_eq!(resolved_customer.object().unwrap().name, "John Doe");

        // Load order items and verify their relations
        let loaded_item1 = order_item_tree.get(order_item1.key())?.unwrap();
        assert_eq!(loaded_item1.order_id, order.id);
        assert_eq!(loaded_item1.product_id, product1.id);

        // Resolve item's product relation
        let product_key = loaded_item1.product.key().unwrap();
        let item_product = product_tree.get(product_key.clone())?.unwrap();
        let resolved_product = loaded_item1.product.clone().resolve(item_product);
        assert!(resolved_product.is_resolved());
        assert_eq!(resolved_product.object().unwrap().name, "Laptop");

        // Load address and verify customer relation
        let loaded_address = address_tree.get(address.key())?.unwrap();
        assert_eq!(loaded_address.customer_id, customer.id);
        assert!(loaded_address.customer.is_unresolved());

        Ok(())
    }

    #[test]
    fn test_bidirectional_relations() -> Result<()> {
        let (db, _temp_dir) = create_blog_database()?;

        // Create trees using the new type-safe API
        let user_tree: NetabaseSledTree<blog_schema::User, blog_schema::UserKey> =
            db.get_main_tree()?;
        let post_tree: NetabaseSledTree<blog_schema::Post, blog_schema::PostKey> =
            db.get_main_tree()?;

        // Create bidirectional relationship: User <-> Post
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // post.author is already set in create_sample_post

        // Store both entities
        user_tree.insert(user.key(), user.clone())?;
        post_tree.insert(post.key(), post.clone())?;

        // Load user and verify it exists
        let loaded_user = user_tree.get(user.key())?.unwrap();
        assert_eq!(loaded_user.id, user.id);

        // Load post and verify author relation
        let loaded_post = post_tree.get(post.key())?.unwrap();
        let post_author_key = loaded_post.author.key().unwrap();
        let post_author = user_tree.get(post_author_key.clone())?.unwrap();
        assert_eq!(post_author.id, user.id);

        Ok(())
    }

    #[test]
    fn test_relational_tree_operations() -> Result<()> {
        let (db, _temp_dir) = create_blog_database()?;

        // Create trees using the new type-safe API
        let user_tree: NetabaseSledTree<blog_schema::User, blog_schema::UserKey> =
            db.get_main_tree()?;
        let post_tree: NetabaseSledTree<blog_schema::Post, blog_schema::PostKey> =
            db.get_main_tree()?;

        // Test basic relational functionality through the main trees
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // Store entities
        user_tree.insert(user.key(), user.clone())?;
        post_tree.insert(post.key(), post.clone())?;

        // Test that relations work properly
        let loaded_post = post_tree.get(post.key())?.unwrap();
        assert!(loaded_post.author.is_unresolved());

        // Resolve the author relation
        let author_key = loaded_post.author.key().unwrap();
        let author = user_tree.get(author_key.clone())?.unwrap();
        let resolved_author = loaded_post.author.clone().resolve(author);

        assert!(resolved_author.is_resolved());
        assert_eq!(resolved_author.object().unwrap().id, user.id);

        Ok(())
    }

    #[test]
    fn test_secondary_key_operations_with_relations() -> Result<()> {
        let (db, _temp_dir) = create_blog_database()?;

        // Create trees using the new type-safe API
        let user_tree: NetabaseSledTree<blog_schema::User, blog_schema::UserKey> =
            db.get_main_tree()?;
        let post_tree: NetabaseSledTree<blog_schema::Post, blog_schema::PostKey> =
            db.get_main_tree()?;

        // Create secondary trees for testing
        let _user_secondary_tree: NetabaseSledTree<blog_schema::User, blog_schema::UserKey> =
            db.get_secondary_tree()?;
        let _post_secondary_tree: NetabaseSledTree<blog_schema::Post, blog_schema::PostKey> =
            db.get_secondary_tree()?;

        // Create user and post with relations
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // Store main entities
        user_tree.insert(user.key(), user.clone())?;
        post_tree.insert(post.key(), post.clone())?;

        // Test that secondary key operations work
        assert_eq!(user_tree.len(), 1);
        assert_eq!(post_tree.len(), 1);

        // Test loading and verify relations are intact
        let loaded_user = user_tree.get(user.key())?.unwrap();
        assert_eq!(loaded_user.id, user.id);
        assert_eq!(loaded_user.email, user.email);

        let loaded_post = post_tree.get(post.key())?.unwrap();
        assert_eq!(loaded_post.id, post.id);
        assert_eq!(loaded_post.author_id, user.id);

        Ok(())
    }

    #[test]
    fn test_relation_macro_type_transformation() -> Result<()> {
        // This test verifies that the #[relation] macro properly transforms types

        // Create instances to test the transformed field types
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);
        let comment = create_sample_comment(1, post.id, user.id);

        // Verify that relational fields are now RelationalLink types
        // post.author should be RelationalLink<UserKey, User>
        assert!(post.author.is_unresolved());

        // Post has author relational link

        // comment.post should be RelationalLink<PostKey, Post>
        assert!(comment.post.is_unresolved());

        // comment.author should be RelationalLink<UserKey, User>
        assert!(comment.author.is_unresolved());

        // Test that we can resolve these links using the improved pattern
        let mut post_copy = post.clone();
        let mut comment_copy = comment.clone();

        {
            let resolved_author_ref = post_copy.author.resolve_mut(user.clone());
            assert_eq!(resolved_author_ref.id, user.id);
        }
        assert!(post_copy.author.is_resolved());

        {
            let resolved_post_ref = comment_copy.post.resolve_mut(post.clone());
            assert_eq!(resolved_post_ref.id, post.id);
        }
        assert!(comment_copy.post.is_resolved());

        Ok(())
    }
}
