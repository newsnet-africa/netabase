use bincode::{Decode, Encode};
use netabase::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    relational::RelationalLink,
    traits::{NetabaseModel, NetabaseModelKey, NetabaseSchema},
};
use netabase_macros::{NetabaseModel, NetabaseModelKey, netabase_schema_module};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use tempfile::TempDir;

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
        let temp_dir = TempDir::new()?;
        let db = NetabaseSledDatabase::new_with_name("blog_test_db")?;
        Ok((db, temp_dir))
    }

    fn create_ecommerce_database() -> Result<(NetabaseSledDatabase<EcommerceSchema>, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db = NetabaseSledDatabase::new_with_name("ecommerce_test_db")?;
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
        let (db, _temp_dir) = create_blog_database()?;

        // Test that database was created successfully
        assert!(!db.db().was_recovered());

        // Test tree name generation
        let tree_names = db.tree_names();
        assert!(!tree_names.is_empty());

        Ok(())
    }

    #[test]
    fn test_relation_discriminants() -> Result<()> {
        let (mut db, _temp_dir) = create_blog_database()?;

        // Test that schema discriminants are available
        let blog_discriminants = BlogSchema::all_schema_discriminants();
        assert!(!blog_discriminants.is_empty());

        // Initialize trees using discriminants
        db.initialize_trees_from_discriminants(&blog_discriminants)?;

        // Test that all model trees can be accessed using discriminants
        // Note: The discriminants are generated as variants like UserKey, PostKey, etc.
        // We need to iterate through the available discriminants
        for discriminant in &blog_discriminants {
            let tree = db.get_main_tree(discriminant)?;
            assert_eq!(tree.len(), 0);
        }

        Ok(())
    }

    #[test]
    fn test_relational_link_functionality() -> Result<()> {
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // Test that the macro transformed the fields correctly
        assert!(post.author.is_unresolved());
        assert_eq!(post.author.key(), Some(&user.key()));
        assert!(post.comments.is_empty());

        // Test resolving the author link
        let resolved_author = post.author.clone().resolve(user.clone());
        assert!(resolved_author.is_resolved());
        assert_eq!(resolved_author.object().unwrap().id, user.id);

        Ok(())
    }

    #[test]
    fn test_storing_and_loading_relational_data() -> Result<()> {
        let (mut db, _temp_dir) = create_blog_database()?;

        // Initialize trees for blog schema
        let blog_discriminants = BlogSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&blog_discriminants)?;

        let user_tree = db.get_main_tree::<User>("User")?;
        let post_tree = db.get_main_tree::<Post>("Post")?;
        let profile_tree = db.get_main_tree::<Profile>("Profile")?;

        // Create user with relational links
        let mut user = create_sample_user(1);
        let profile = create_sample_profile(1, user.id);
        let post1 = create_sample_post(1, user.id);
        let post2 = create_sample_post(2, user.id);

        // Set up relational links in user
        user.profile = Some(RelationalLink::from_key(profile.key()));
        user.posts = vec![
            RelationalLink::from_key(post1.key()),
            RelationalLink::from_key(post2.key()),
        ];

        // Store all entities
        user_tree.insert(&user.key(), &user)?;
        profile_tree.insert(&profile.key(), &profile)?;
        post_tree.insert(&post1.key(), &post1)?;
        post_tree.insert(&post2.key(), &post2)?;

        // Load and verify relational data
        let loaded_user = user_tree.get(&user.key())?.unwrap();

        // Verify profile link
        assert!(loaded_user.profile.is_some());
        let profile_link = loaded_user.profile.as_ref().unwrap();
        assert!(profile_link.is_unresolved());
        assert_eq!(profile_link.key(), Some(&profile.key()));

        // Verify posts links
        assert_eq!(loaded_user.posts.len(), 2);
        for post_link in &loaded_user.posts {
            assert!(post_link.is_unresolved());
            assert!(post_link.key().is_some());
        }

        Ok(())
    }

    #[test]
    fn test_resolving_relational_links() -> Result<()> {
        let (mut db, _temp_dir) = create_blog_database()?;

        let blog_discriminants = BlogSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&blog_discriminants)?;

        let user_tree = db.get_main_tree::<User>("User")?;
        let post_tree = db.get_main_tree::<Post>("Post")?;
        let comment_tree = db.get_main_tree::<Comment>("Comment")?;

        // Create and store test data
        let user = create_sample_user(1);
        let mut post = create_sample_post(1, user.id);
        let comment = create_sample_comment(1, post.id, user.id);

        // Add comment link to post
        post.comments = vec![RelationalLink::from_key(comment.key())];

        user_tree.insert(&user.key(), &user)?;
        post_tree.insert(&post.key(), &post)?;
        comment_tree.insert(&comment.key(), &comment)?;

        // Load post and resolve its author relation
        let loaded_post = post_tree.get(&post.key())?.unwrap();
        assert!(loaded_post.author.is_unresolved());

        // Resolve the author link
        let author_key = loaded_post.author.key().unwrap();
        let author = user_tree.get(author_key)?.unwrap();
        let resolved_post_author = loaded_post.author.clone().resolve(author.clone());

        assert!(resolved_post_author.is_resolved());
        assert_eq!(resolved_post_author.object().unwrap().id, user.id);

        // Load comment and resolve its relations
        let loaded_comment = comment_tree.get(&comment.key())?.unwrap();

        // Resolve comment's post relation
        let comment_post_key = loaded_comment.post.key().unwrap();
        let comment_post = post_tree.get(comment_post_key)?.unwrap();
        let resolved_comment_post = loaded_comment.post.clone().resolve(comment_post);

        assert!(resolved_comment_post.is_resolved());
        assert_eq!(resolved_comment_post.object().unwrap().id, post.id);

        Ok(())
    }

    #[test]
    fn test_empty_relations() -> Result<()> {
        // Test that relation discriminants are properly generated

        // Test User relations
        let user_relations: Vec<&str> = User::relations();
        assert_eq!(user_relations.len(), 2);
        assert!(user_relations.contains(&"posts"));
        assert!(user_relations.contains(&"profile"));

        // Test Post relations
        let post_relations: Vec<&str> = Post::relations();
        assert_eq!(post_relations.len(), 2);
        assert!(post_relations.contains(&"author"));
        assert!(post_relations.contains(&"comments"));

        // Test Comment relations
        let comment_relations: Vec<&str> = Comment::relations();
        assert_eq!(comment_relations.len(), 2);
        assert!(comment_relations.contains(&"post"));
        assert!(comment_relations.contains(&"author"));

        // Test Profile relations
        let profile_relations: Vec<&str> = Profile::relations();
        assert_eq!(profile_relations.len(), 1);
        assert!(profile_relations.contains(&"user"));

        // Test ecommerce relations
        let customer_relations: Vec<&str> = Customer::relations();
        assert_eq!(customer_relations.len(), 2);
        assert!(customer_relations.contains(&"orders"));
        assert!(customer_relations.contains(&"addresses"));

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
        let (mut db, _temp_dir) = create_ecommerce_database()?;

        let ecommerce_discriminants = EcommerceSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&ecommerce_discriminants)?;

        let customer_tree = db.get_main_tree::<Customer>("Customer")?;
        let order_tree = db.get_main_tree::<Order>("Order")?;
        let order_item_tree = db.get_main_tree::<OrderItem>("OrderItem")?;
        let product_tree = db.get_main_tree::<Product>("Product")?;
        let address_tree = db.get_main_tree::<Address>("Address")?;

        // Create test data
        let mut customer = Customer {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: chrono::Utc::now(),
            orders: vec![],
            addresses: vec![],
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

        let mut order = Order {
            id: 1,
            customer_id: customer.id,
            status: "pending".to_string(),
            total_amount: 1059.97,
            created_at: chrono::Utc::now(),
            customer: RelationalLink::from_key(customer.key()),
            items: vec![],
        };

        let order_item1 = OrderItem {
            id: 1,
            order_id: order.id,
            product_id: product1.id,
            quantity: 1,
            price: product1.price,
            order: RelationalLink::from_key(order.key()),
            product: RelationalLink::from_key(product1.key()),
        };

        let order_item2 = OrderItem {
            id: 2,
            order_id: order.id,
            product_id: product2.id,
            quantity: 2,
            price: product2.price,
            order: RelationalLink::from_key(order.key()),
            product: RelationalLink::from_key(product2.key()),
        };

        let address = Address {
            id: 1,
            customer_id: customer.id,
            street: "123 Main St".to_string(),
            city: "Anytown".to_string(),
            state: "CA".to_string(),
            zip_code: "12345".to_string(),
            country: "USA".to_string(),
            customer: RelationalLink::from_key(customer.key()),
        };

        // Set up relations
        order.items = vec![
            RelationalLink::from_key(order_item1.key()),
            RelationalLink::from_key(order_item2.key()),
        ];

        customer.orders = vec![RelationalLink::from_key(order.key())];
        customer.addresses = vec![RelationalLink::from_key(address.key())];

        // Store all entities
        customer_tree.insert(&customer.key(), &customer)?;
        order_tree.insert(&order.key(), &order)?;
        order_item_tree.insert(&order_item1.key(), &order_item1)?;
        order_item_tree.insert(&order_item2.key(), &order_item2)?;
        product_tree.insert(&product1.key(), &product1)?;
        product_tree.insert(&product2.key(), &product2)?;
        address_tree.insert(&address.key(), &address)?;

        // Load and verify the complex relationship
        let loaded_customer = customer_tree.get(&customer.key())?.unwrap();

        // Customer should have one order and one address
        assert_eq!(loaded_customer.orders.len(), 1);
        assert_eq!(loaded_customer.addresses.len(), 1);

        // Resolve customer's order
        let order_link = &loaded_customer.orders[0];
        assert!(order_link.is_unresolved());
        let order_key = order_link.key().unwrap();
        let loaded_order = order_tree.get(order_key)?.unwrap();

        // Order should have two items
        assert_eq!(loaded_order.items.len(), 2);

        // Resolve order items and their products
        for item_link in &loaded_order.items {
            let item_key = item_link.key().unwrap();
            let loaded_item = order_item_tree.get(item_key)?.unwrap();

            // Each item should have a product relation
            assert!(loaded_item.product.is_unresolved());
            let product_key = loaded_item.product.key().unwrap();
            let loaded_product = product_tree.get(product_key)?;
            assert!(loaded_product.is_some());
        }

        Ok(())
    }

    #[test]
    fn test_bidirectional_relations() -> Result<()> {
        let (mut db, _temp_dir) = create_blog_database()?;

        let blog_discriminants = BlogSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&blog_discriminants)?;

        let user_tree = db.get_main_tree::<User>("User")?;
        let post_tree = db.get_main_tree::<Post>("Post")?;

        // Create bidirectional relationship: User <-> Post
        let mut user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // Set up bidirectional links
        user.posts = vec![RelationalLink::from_key(post.key())];
        // post.author is already set in create_sample_post

        // Store both entities
        user_tree.insert(&user.key(), &user)?;
        post_tree.insert(&post.key(), &post)?;

        // Load user and verify post relation
        let loaded_user = user_tree.get(&user.key())?.unwrap();
        assert_eq!(loaded_user.posts.len(), 1);

        let user_post_key = loaded_user.posts[0].key().unwrap();
        let user_post = post_tree.get(user_post_key)?.unwrap();
        assert_eq!(user_post.id, post.id);

        // Load post and verify author relation
        let loaded_post = post_tree.get(&post.key())?.unwrap();
        let post_author_key = loaded_post.author.key().unwrap();
        let post_author = user_tree.get(post_author_key)?.unwrap();
        assert_eq!(post_author.id, user.id);

        Ok(())
    }

    #[test]
    fn test_relational_tree_operations() -> Result<()> {
        let (mut db, _temp_dir) = create_blog_database()?;

        let blog_discriminants = BlogSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&blog_discriminants)?;

        // Test that relational trees are created for models with relations
        let user_relations_tree = db.get_relational_tree::<User>("User", "posts")?;
        let post_relations_tree = db.get_relational_tree::<Post>("Post", "comments")?;

        // These should be empty initially
        assert_eq!(user_relations_tree.len()?, 0);
        assert_eq!(post_relations_tree.len()?, 0);

        // Test storing relational mappings
        let user_key = UserKey::Primary(UserPrimaryKey { id: 1 });
        let post_key = PostKey::Primary(PostPrimaryKey { id: 1 });
        let comment_key = CommentKey::Primary(CommentPrimaryKey { id: 1 });

        // Store user -> posts relation
        let posts_relation = UserRelations::PostsRelation(vec![post_key.clone()]);
        user_relations_tree.insert(&user_key, &posts_relation)?;

        // Store post -> comments relation
        let comments_relation = PostRelations::CommentsRelation(vec![comment_key.clone()]);
        post_relations_tree.insert(&post_key, &comments_relation)?;

        // Verify storage
        assert_eq!(user_relations_tree.len()?, 1);
        assert_eq!(post_relations_tree.len()?, 1);

        Ok(())
    }

    #[test]
    fn test_secondary_key_operations_with_relations() -> Result<()> {
        let (mut db, _temp_dir) = create_blog_database()?;

        let blog_discriminants = BlogSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&blog_discriminants)?;

        let user_tree = db.get_main_tree::<User>("User")?;
        let post_tree = db.get_main_tree::<Post>("Post")?;
        let email_tree = db.get_secondary_tree::<User>("User", "email")?;
        let author_tree = db.get_secondary_tree::<Post>("Post", "author_id")?;

        // Create user and post with relations
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // Store main entities
        user_tree.insert(&user.key(), &user)?;
        post_tree.insert(&post.key(), &post)?;

        // Store secondary key mappings
        let email_key = UserSecondaryKeys::EmailKey(user.email.clone());
        let author_key = PostSecondaryKeys::AuthorIdKey(post.author_id);

        email_tree.insert(&email_key, &user.id)?;
        author_tree.insert(&author_key, &post.id)?;

        // Test lookup by secondary key
        let found_user_id = email_tree.get(&email_key)?.unwrap();
        assert_eq!(found_user_id, user.id);

        let found_post_id = author_tree.get(&author_key)?.unwrap();
        assert_eq!(found_post_id, post.id);

        // Load full entity and verify relations are intact
        let primary_key: UserKey = found_user_id.into();
        let loaded_user = user_tree.get(&primary_key)?.unwrap();
        // User should maintain its empty relations
        assert!(loaded_user.posts.is_empty());
        assert!(loaded_user.profile.is_none());

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

        // post.comments should be Vec<RelationalLink<CommentKey, Comment>>
        assert!(post.comments.is_empty());

        // comment.post should be RelationalLink<PostKey, Post>
        assert!(comment.post.is_unresolved());

        // comment.author should be RelationalLink<UserKey, User>
        assert!(comment.author.is_unresolved());

        // Test that we can resolve these links
        let resolved_author = post.author.resolve(user.clone());
        assert!(resolved_author.is_resolved());
        assert_eq!(resolved_author.object().unwrap().id, user.id);

        let resolved_post = comment.post.resolve(post.clone());
        assert!(resolved_post.is_resolved());
        assert_eq!(resolved_post.object().unwrap().id, post.id);

        Ok(())
    }
}
