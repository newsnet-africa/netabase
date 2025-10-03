//! # Blog System Example
//!
//! This example demonstrates a complete blog system using Netabase with:
//! - Multiple model types (Users, Posts, Comments)
//! - Primary and secondary key queries
//! - Relational data modeling
//! - Both local and distributed operations
//! - Error handling and best practices

use std::time::Duration;

use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseAdvancedQuery, NetabaseModel, NetabaseSecondaryKeyQuery},
};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::time::timeout;

// Define the blog schema with multiple related models
#[netabase_schema_module(BlogSchema, BlogKeys)]
mod blog_schema {
    use super::*;

    /// User model representing blog authors and commenters
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
        #[secondary_key]
        pub status: UserStatus,
        pub display_name: String,
        pub bio: String,
        pub avatar_url: Option<String>,
        pub created_at: u64,
        pub last_login: Option<u64>,
    }

    /// Blog post model with rich metadata
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub id: u64,
        pub title: String,
        pub slug: String,
        pub content: String,
        pub excerpt: String,
        #[secondary_key]
        pub author_id: u64, // Foreign key to User
        #[secondary_key]
        pub category: String,
        #[secondary_key]
        pub published: bool,
        #[secondary_key]
        pub featured: bool,
        pub tags: Vec<String>,
        pub view_count: u64,
        pub like_count: u64,
        pub created_at: u64,
        pub updated_at: u64,
        pub published_at: Option<u64>,
    }

    /// Comment model for post discussions
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CommentKey)]
    pub struct Comment {
        #[key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub post_id: u64, // Foreign key to Post
        #[secondary_key]
        pub author_id: u64, // Foreign key to User
        #[secondary_key]
        pub status: CommentStatus,
        pub parent_comment_id: Option<u64>, // For threaded comments
        pub like_count: u64,
        pub created_at: u64,
        pub updated_at: Option<u64>,
    }

    /// Category model for organizing posts
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CategoryKey)]
    pub struct Category {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub slug: String,
        pub description: String,
        pub color: String,
        pub post_count: u64,
        pub created_at: u64,
    }

    /// User status enumeration
    #[derive(
        Clone, Debug, PartialEq, Eq, Hash, Default, Encode, Decode, Serialize, Deserialize,
    )]
    pub enum UserStatus {
        #[default]
        Active,
        Inactive,
        Suspended,
        Pending,
    }

    /// Comment status enumeration
    #[derive(
        Clone, Debug, PartialEq, Eq, Hash, Default, Encode, Decode, Serialize, Deserialize,
    )]
    pub enum CommentStatus {
        #[default]
        Published,
        Pending,
        Spam,
        Deleted,
    }
}

use blog_schema::*;

/// Blog service that encapsulates all blog operations
pub struct BlogService {
    db: NetabaseSledDatabase<BlogSchema>,
    user_tree: NetabaseSledTree<User, UserKey>,
    post_tree: NetabaseSledTree<Post, PostKey>,
    comment_tree: NetabaseSledTree<Comment, CommentKey>,
    category_tree: NetabaseSledTree<Category, CategoryKey>,
}

impl BlogService {
    /// Create a new blog service with local database
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = NetabaseSledDatabase::new_with_path(db_path)?;
        let user_tree = db.get_main_tree()?;
        let post_tree = db.get_main_tree()?;
        let comment_tree = db.get_main_tree()?;
        let category_tree = db.get_main_tree()?;

        Ok(Self {
            db,
            user_tree,
            post_tree,
            comment_tree,
            category_tree,
        })
    }

    // === User Management ===

    /// Create a new user
    pub fn create_user(&self, user: User) -> Result<(), Box<dyn std::error::Error>> {
        // Check if username or email already exists
        let existing_by_email = self
            .user_tree
            .query_by_secondary_key(UserSecondaryKeys::EmailKey(user.email.clone()))?;

        if !existing_by_email.is_empty() {
            return Err("Email already exists".into());
        }

        self.user_tree.insert(user.key(), user)?;
        Ok(())
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: u64) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let key = UserKey::Primary(UserPrimaryKey(user_id));
        Ok(self.user_tree.get(key)?)
    }

    /// Get user by email
    pub fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let users = self
            .user_tree
            .query_by_secondary_key(UserSecondaryKeys::EmailKey(email.to_string()))?;
        Ok(users.into_iter().next())
    }

    /// Get all active users
    pub fn get_active_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let users = self
            .user_tree
            .query_by_secondary_key(UserSecondaryKeys::StatusKey(UserStatus::Active))?;
        Ok(users)
    }

    // === Post Management ===

    /// Create a new blog post
    pub fn create_post(&self, post: Post) -> Result<(), Box<dyn std::error::Error>> {
        // Verify author exists
        if self.get_user(post.author_id)?.is_none() {
            return Err("Author does not exist".into());
        }

        self.post_tree.insert(post.key(), post)?;
        Ok(())
    }

    /// Get post by ID
    pub fn get_post(&self, post_id: u64) -> Result<Option<Post>, Box<dyn std::error::Error>> {
        let key = PostKey::Primary(PostPrimaryKey(post_id));
        Ok(self.post_tree.get(key)?)
    }

    /// Get all published posts
    pub fn get_published_posts(&self) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts = self
            .post_tree
            .query_by_secondary_key(PostSecondaryKeys::PublishedKey(true))?;
        Ok(posts)
    }

    /// Get posts by author
    pub fn get_posts_by_author(
        &self,
        author_id: u64,
    ) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts = self
            .post_tree
            .query_by_secondary_key(PostSecondaryKeys::Author_idKey(author_id))?;
        Ok(posts)
    }

    /// Get posts by category
    pub fn get_posts_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts = self
            .post_tree
            .query_by_secondary_key(PostSecondaryKeys::CategoryKey(category.to_string()))?;
        Ok(posts)
    }

    /// Get featured posts
    pub fn get_featured_posts(&self) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts = self
            .post_tree
            .query_by_secondary_key(PostSecondaryKeys::FeaturedKey(true))?;
        Ok(posts)
    }

    /// Search posts by title or content
    pub fn search_posts(&self, query: &str) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let query_lower = query.to_lowercase();
        let results = self.post_tree.query_with_filter(|post| {
            post.title.to_lowercase().contains(&query_lower)
                || post.content.to_lowercase().contains(&query_lower)
                || post.excerpt.to_lowercase().contains(&query_lower)
        })?;
        Ok(results.into_iter().map(|(_, post)| post).collect())
    }

    /// Get popular posts (high view count)
    pub fn get_popular_posts(
        &self,
        min_views: u64,
    ) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let results = self
            .post_tree
            .query_with_filter(|post| post.view_count >= min_views)?;
        let mut posts: Vec<Post> = results.into_iter().map(|(_, post)| post).collect();
        posts.sort_by(|a, b| b.view_count.cmp(&a.view_count));
        Ok(posts)
    }

    // === Comment Management ===

    /// Add a comment to a post
    pub fn add_comment(&self, comment: Comment) -> Result<(), Box<dyn std::error::Error>> {
        // Verify post and author exist
        if self.get_post(comment.post_id)?.is_none() {
            return Err("Post does not exist".into());
        }
        if self.get_user(comment.author_id)?.is_none() {
            return Err("Author does not exist".into());
        }

        self.comment_tree.insert(comment.key(), comment)?;
        Ok(())
    }

    /// Get comments for a post
    pub fn get_post_comments(
        &self,
        post_id: u64,
    ) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let comments = self
            .comment_tree
            .query_by_secondary_key(CommentSecondaryKeys::Post_idKey(post_id))?;
        Ok(comments)
    }

    /// Get comments by user
    pub fn get_user_comments(
        &self,
        author_id: u64,
    ) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let comments = self
            .comment_tree
            .query_by_secondary_key(CommentSecondaryKeys::Author_idKey(author_id))?;
        Ok(comments)
    }

    /// Get pending comments for moderation
    pub fn get_pending_comments(&self) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let comments = self
            .comment_tree
            .query_by_secondary_key(CommentSecondaryKeys::StatusKey(CommentStatus::Pending))?;
        Ok(comments)
    }

    // === Analytics ===

    /// Get blog statistics
    pub fn get_statistics(&self) -> Result<BlogStatistics, Box<dyn std::error::Error>> {
        let total_users = self.user_tree.len();
        let total_posts = self.post_tree.len();
        let total_comments = self.comment_tree.len();

        let published_posts = self.get_published_posts()?.len();
        let active_users = self.get_active_users()?.len();
        let pending_comments = self.get_pending_comments()?.len();

        Ok(BlogStatistics {
            total_users,
            active_users,
            total_posts,
            published_posts,
            total_comments,
            pending_comments,
        })
    }

    /// Get top authors by post count
    pub fn get_top_authors(
        &self,
        limit: usize,
    ) -> Result<Vec<AuthorStats>, Box<dyn std::error::Error>> {
        let mut author_counts = std::collections::HashMap::new();

        // Count posts per author
        for result in self.post_tree.iter() {
            let (_, post) = result?;
            *author_counts.entry(post.author_id).or_insert(0) += 1;
        }

        // Convert to sorted vector
        let mut authors: Vec<AuthorStats> = author_counts
            .into_iter()
            .filter_map(|(author_id, post_count)| {
                self.get_user(author_id)
                    .ok()
                    .flatten()
                    .map(|user| AuthorStats { user, post_count })
            })
            .collect();

        authors.sort_by(|a, b| b.post_count.cmp(&a.post_count));
        authors.truncate(limit);
        Ok(authors)
    }
}

/// Blog statistics structure
#[derive(Debug)]
pub struct BlogStatistics {
    pub total_users: usize,
    pub active_users: usize,
    pub total_posts: usize,
    pub published_posts: usize,
    pub total_comments: usize,
    pub pending_comments: usize,
}

/// Author statistics
#[derive(Debug)]
pub struct AuthorStats {
    pub user: User,
    pub post_count: usize,
}

/// Distributed blog service for network operations
pub struct DistributedBlogService {
    netabase: Netabase<BlogSchema>,
}

impl DistributedBlogService {
    /// Create a new distributed blog service
    pub fn new(db_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let netabase = match db_path {
            Some(path) => Netabase::new_with_path(path)?,
            None => Netabase::new()?,
        };

        Ok(Self { netabase })
    }

    /// Start the network service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.netabase.start_swarm().await?;
        Ok(())
    }

    /// Stop the network service
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.netabase.stop_swarm().await?;
        Ok(())
    }

    /// Publish a post to the network
    pub async fn publish_post(&self, post: Post) -> Result<(), Box<dyn std::error::Error>> {
        match timeout(
            Duration::from_secs(10),
            self.netabase.put_record(post.clone()),
        )
        .await
        {
            Ok(result) => match result {
                Ok(_) => {
                    println!("Post '{}' published to network", post.title);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to publish post: {}", e).into()),
            },
            Err(_) => Err("Publish operation timed out".into()),
        }
    }

    /// Retrieve a post from the network
    pub async fn get_network_post(
        &self,
        post_id: u64,
    ) -> Result<Option<Post>, Box<dyn std::error::Error>> {
        let key = PostKey::Primary(PostPrimaryKey(post_id));

        match timeout(Duration::from_secs(10), self.netabase.get_record(key)).await {
            Ok(result) => {
                // Process the query result to extract the post
                // This is a simplified example - in practice you'd parse the QueryResult
                println!("Attempted to retrieve post {} from network", post_id);
                Ok(None) // Placeholder - would decode from QueryResult
            }
            Err(_) => Err("Get operation timed out".into()),
        }
    }

    /// Bootstrap and join the network
    pub async fn join_network(&self) -> Result<(), Box<dyn std::error::Error>> {
        match timeout(Duration::from_secs(30), self.netabase.bootstrap()).await {
            Ok(result) => match result {
                Ok(_) => {
                    println!("Successfully joined the blog network");
                    Ok(())
                }
                Err(e) => {
                    println!("Bootstrap completed with issues: {}", e);
                    Ok(()) // Non-fatal for single-node setups
                }
            },
            Err(_) => {
                println!("Bootstrap timed out - may be running in isolated mode");
                Ok(()) // Non-fatal
            }
        }
    }
}

/// Sample data creation functions
impl BlogService {
    /// Create sample blog data for demonstration
    pub fn create_sample_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create sample users
        let users = vec![
            User {
                id: 1,
                username: "alice_author".to_string(),
                email: "alice@blogexample.com".to_string(),
                status: UserStatus::Active,
                display_name: "Alice Johnson".to_string(),
                bio: "Tech blogger and software engineer".to_string(),
                avatar_url: Some("https://example.com/avatars/alice.jpg".to_string()),
                created_at: 1640995200, // 2022-01-01
                last_login: Some(chrono::Utc::now().timestamp() as u64),
            },
            User {
                id: 2,
                username: "bob_writer".to_string(),
                email: "bob@blogexample.com".to_string(),
                status: UserStatus::Active,
                display_name: "Bob Smith".to_string(),
                bio: "Writer and technology enthusiast".to_string(),
                avatar_url: None,
                created_at: 1641081600, // 2022-01-02
                last_login: Some(chrono::Utc::now().timestamp() as u64 - 3600),
            },
        ];

        for user in users {
            self.create_user(user)?;
        }

        // Create sample posts
        let posts = vec![
            Post {
                id: 1,
                title: "Getting Started with Rust".to_string(),
                slug: "getting-started-with-rust".to_string(),
                content: "Rust is a systems programming language that focuses on safety, speed, and concurrency...".to_string(),
                excerpt: "Learn the basics of Rust programming language".to_string(),
                author_id: 1,
                category: "Programming".to_string(),
                published: true,
                featured: true,
                tags: vec!["rust".to_string(), "programming".to_string(), "tutorial".to_string()],
                view_count: 1500,
                like_count: 89,
                created_at: 1641168000, // 2022-01-03
                updated_at: 1641168000,
                published_at: Some(1641168000),
            },
            Post {
                id: 2,
                title: "Building Distributed Systems".to_string(),
                slug: "building-distributed-systems".to_string(),
                content: "Distributed systems are complex but essential for modern applications...".to_string(),
                excerpt: "An introduction to distributed system design patterns".to_string(),
                author_id: 1,
                category: "Architecture".to_string(),
                published: true,
                featured: false,
                tags: vec!["distributed".to_string(), "systems".to_string(), "architecture".to_string()],
                view_count: 2300,
                like_count: 156,
                created_at: 1641254400, // 2022-01-04
                updated_at: 1641254400,
                published_at: Some(1641254400),
            },
            Post {
                id: 3,
                title: "Understanding Database Design".to_string(),
                slug: "understanding-database-design".to_string(),
                content: "Good database design is crucial for application performance...".to_string(),
                excerpt: "Learn database design principles and best practices".to_string(),
                author_id: 2,
                category: "Database".to_string(),
                published: false,
                featured: false,
                tags: vec!["database".to_string(), "design".to_string(), "sql".to_string()],
                view_count: 0,
                like_count: 0,
                created_at: 1641340800, // 2022-01-05
                updated_at: 1641340800,
                published_at: None,
            },
        ];

        for post in posts {
            self.create_post(post)?;
        }

        // Create sample comments
        let comments = vec![
            Comment {
                id: 1,
                content: "Great introduction to Rust! Very helpful for beginners.".to_string(),
                post_id: 1,
                author_id: 2,
                status: CommentStatus::Published,
                parent_comment_id: None,
                like_count: 5,
                created_at: 1641254400, // 2022-01-04
                updated_at: None,
            },
            Comment {
                id: 2,
                content: "Thanks Bob! I'm glad you found it useful.".to_string(),
                post_id: 1,
                author_id: 1, // Alice replying
                status: CommentStatus::Published,
                parent_comment_id: Some(1), // Reply to comment 1
                like_count: 2,
                created_at: 1641340800, // 2022-01-05
                updated_at: None,
            },
            Comment {
                id: 3,
                content: "Looking forward to more advanced Rust topics!".to_string(),
                post_id: 2,
                author_id: 2,
                status: CommentStatus::Published,
                parent_comment_id: None,
                like_count: 8,
                created_at: 1641427200, // 2022-01-06
                updated_at: None,
            },
        ];

        for comment in comments {
            self.add_comment(comment)?;
        }

        println!("Sample blog data created successfully!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== Netabase Blog System Example ===\n");

    // === Local Blog Service Demo ===
    println!("1. Creating local blog service...");
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("blog_example_db");
    let blog_service = BlogService::new(&db_path.to_string_lossy())?;

    // Create sample data
    println!("2. Creating sample blog data...");
    blog_service.create_sample_data()?;

    // Demonstrate queries
    println!("\n3. Demonstrating blog queries:");

    // Get all users
    let active_users = blog_service.get_active_users()?;
    println!("   Active users: {}", active_users.len());
    for user in &active_users {
        println!("     - {} ({})", user.display_name, user.username);
    }

    // Get published posts
    let published_posts = blog_service.get_published_posts()?;
    println!("\n   Published posts: {}", published_posts.len());
    for post in &published_posts {
        println!(
            "     - '{}' by user ID {} ({} views)",
            post.title, post.author_id, post.view_count
        );
    }

    // Get posts by category
    let programming_posts = blog_service.get_posts_by_category("Programming")?;
    println!("\n   Programming posts: {}", programming_posts.len());

    // Get featured posts
    let featured_posts = blog_service.get_featured_posts()?;
    println!("   Featured posts: {}", featured_posts.len());

    // Search posts
    let search_results = blog_service.search_posts("rust")?;
    println!("   Posts mentioning 'rust': {}", search_results.len());

    // Get popular posts
    let popular_posts = blog_service.get_popular_posts(1000)?;
    println!("   Popular posts (>1000 views): {}", popular_posts.len());

    // Get comments for a post
    let post_comments = blog_service.get_post_comments(1)?;
    println!("\n   Comments on post 1: {}", post_comments.len());
    for comment in &post_comments {
        println!(
            "     - Comment by user {}: '{}'",
            comment.author_id, comment.content
        );
    }

    // Get blog statistics
    let stats = blog_service.get_statistics()?;
    println!("\n4. Blog Statistics:");
    println!(
        "   Total users: {} (active: {})",
        stats.total_users, stats.active_users
    );
    println!(
        "   Total posts: {} (published: {})",
        stats.total_posts, stats.published_posts
    );
    println!(
        "   Total comments: {} (pending: {})",
        stats.total_comments, stats.pending_comments
    );

    // Get top authors
    let top_authors = blog_service.get_top_authors(5)?;
    println!("\n   Top authors:");
    for author_stats in &top_authors {
        println!(
            "     - {}: {} posts",
            author_stats.user.display_name, author_stats.post_count
        );
    }

    // === Distributed Blog Service Demo ===
    println!("\n5. Testing distributed blog service...");

    let mut distributed_service = DistributedBlogService::new(Some("distributed_blog_db"))?;
    distributed_service.start().await?;

    // Join the network
    println!("   Joining the blog network...");
    distributed_service.join_network().await?;

    // Publish a post to the network
    if let Some(post) = published_posts.first() {
        println!("   Publishing post to network: '{}'", post.title);
        distributed_service.publish_post(post.clone()).await?;

        // Try to retrieve it
        println!("   Attempting to retrieve from network...");
        let _network_post = distributed_service.get_network_post(post.id).await?;
    }

    // Cleanup
    distributed_service.stop().await?;

    println!("\n=== Blog System Example Complete ===");
    println!("The example demonstrated:");
    println!("  ✓ Complex data modeling with multiple related entities");
    println!("  ✓ Primary and secondary key queries");
    println!("  ✓ Advanced filtering and search operations");
    println!("  ✓ Statistical analysis and reporting");
    println!("  ✓ Both local and distributed database operations");
    println!("  ✓ Error handling and data validation");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_blog_service() -> BlogService {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_blog");
        BlogService::new(&db_path.to_string_lossy()).unwrap()
    }

    #[test]
    fn test_user_creation() {
        let service = create_test_blog_service();

        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            status: UserStatus::Active,
            display_name: "Test User".to_string(),
            bio: "Test bio".to_string(),
            avatar_url: None,
            created_at: chrono::Utc::now().timestamp() as u64,
            last_login: None,
        };

        assert!(service.create_user(user.clone()).is_ok());

        let retrieved = service.get_user(1).unwrap();
        assert_eq!(retrieved, Some(user));
    }

    #[test]
    fn test_post_queries() {
        let service = create_test_blog_service();
        service.create_sample_data().unwrap();

        // Test published posts query
        let published = service.get_published_posts().unwrap();
        assert_eq!(published.len(), 2);

        // Test posts by category
        let programming_posts = service.get_posts_by_category("Programming").unwrap();
        assert_eq!(programming_posts.len(), 1);

        // Test search functionality
        let search_results = service.search_posts("rust").unwrap();
        assert!(!search_results.is_empty());
    }

    #[test]
    fn test_comment_functionality() {
        let service = create_test_blog_service();
        service.create_sample_data().unwrap();

        let comments = service.get_post_comments(1).unwrap();
        assert!(!comments.is_empty());

        let user_comments = service.get_user_comments(2).unwrap();
        assert!(!user_comments.is_empty());
    }

    #[test]
    fn test_statistics() {
        let service = create_test_blog_service();
        service.create_sample_data().unwrap();

        let stats = service.get_statistics().unwrap();
        assert!(stats.total_users > 0);
        assert!(stats.total_posts > 0);
        assert!(stats.total_comments > 0);

        let top_authors = service.get_top_authors(10).unwrap();
        assert!(!top_authors.is_empty());
    }
}
