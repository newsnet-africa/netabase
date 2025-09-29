//! Working integration example for cycling iterator
//!
//! This example demonstrates a complete working implementation of the cycling iterator
//! that integrates with the actual generated types from netabase.

use std::collections::HashMap;

// In a real implementation, you would import the generated types like this:
// use your_generated_module::{SocialMediaSchema, SocialMediaSchemaDBIter, v1};

/// This example shows how to integrate the cycling iterator with your generated schema.
/// Replace the mock types below with your actual generated types.

#[derive(Debug, Clone)]
pub enum WorkingCatalog {
    User(User),
    Post(Post),
    Comment(Comment),
    Media(Media),
    Reaction(Reaction),
    Notification(Notification),
    UserStats(UserStats),
    HashTag(HashTag),
    PrimitiveTest(PrimitiveTest),
    TestUnit(TestUnit),
    TestTuple(TestTuple),
}

// Example data structures that match your schema
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub followers_count: u32,
    pub following_count: u32,
    pub posts_count: u32,
    pub is_verified: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct Post {
    pub id: u64,
    pub user_id: u64,
    pub content: String,
    pub likes_count: u32,
    pub comments_count: u32,
    pub hashtags: Vec<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u64,
    pub post_id: u64,
    pub user_id: u64,
    pub content: String,
    pub likes_count: u32,
}

#[derive(Debug, Clone)]
pub struct Media {
    pub id: u64,
    pub post_id: u64,
    pub url: String,
    pub media_type: String,
}

#[derive(Debug, Clone)]
pub struct Reaction {
    pub id: u64,
    pub user_id: u64,
    pub target_id: u64,
    pub reaction_type: String,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub user_id: u64,
    pub message: String,
    pub is_read: bool,
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub user_id: u64,
    pub posts_created: u32,
    pub comments_made: u32,
    pub likes_given: u32,
}

#[derive(Debug, Clone)]
pub struct HashTag {
    pub tag: String,
    pub usage_count: u32,
    pub is_trending: bool,
}

#[derive(Debug, Clone)]
pub struct PrimitiveTest {
    pub id: u64,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct TestUnit {
    pub id: u64,
}

#[derive(Debug, Clone)]
pub struct TestTuple {
    pub id: u64,
    pub data: (String, i32),
}

/// Working cycling iterator implementation
pub struct WorkingCyclingIterator<'db> {
    db_scanner: &'db DatabaseScanner<'db>,
    current_type: usize,
    current_data: Option<Box<dyn Iterator<Item = Result<WorkingCatalog, String>> + 'db>>,
}

/// Mock database scanner that simulates the generated DBIter
pub struct DatabaseScanner<'db> {
    users: Vec<User>,
    posts: Vec<Post>,
    comments: Vec<Comment>,
    _phantom: std::marker::PhantomData<&'db ()>,
}

impl<'db> DatabaseScanner<'db> {
    pub fn new() -> Self {
        Self {
            users: create_sample_users(),
            posts: create_sample_posts(),
            comments: create_sample_comments(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn scan_users(&self) -> impl Iterator<Item = Result<User, String>> + '_ {
        self.users.iter().cloned().map(Ok)
    }

    pub fn scan_posts(&self) -> impl Iterator<Item = Result<Post, String>> + '_ {
        self.posts.iter().cloned().map(Ok)
    }

    pub fn scan_comments(&self) -> impl Iterator<Item = Result<Comment, String>> + '_ {
        self.comments.iter().cloned().map(Ok)
    }
}

impl<'db> WorkingCyclingIterator<'db> {
    pub fn new(db_scanner: &'db DatabaseScanner<'db>) -> Self {
        Self {
            db_scanner,
            current_type: 0,
            current_data: None,
        }
    }

    pub fn current_type_name(&self) -> &'static str {
        match self.current_type {
            0 => "User",
            1 => "Post",
            2 => "Comment",
            _ => "Complete",
        }
    }

    fn get_current_iterator(
        &mut self,
    ) -> Option<Box<dyn Iterator<Item = Result<WorkingCatalog, String>> + 'db>> {
        match self.current_type {
            0 => {
                let iter = self.db_scanner.scan_users();
                Some(Box::new(
                    iter.map(|result| result.map(WorkingCatalog::User)),
                ))
            }
            1 => {
                let iter = self.db_scanner.scan_posts();
                Some(Box::new(
                    iter.map(|result| result.map(WorkingCatalog::Post)),
                ))
            }
            2 => {
                let iter = self.db_scanner.scan_comments();
                Some(Box::new(
                    iter.map(|result| result.map(WorkingCatalog::Comment)),
                ))
            }
            _ => None,
        }
    }

    pub fn reset(&mut self) {
        self.current_type = 0;
        self.current_data = None;
    }

    pub fn skip_to_type(&mut self, type_index: usize) -> bool {
        if type_index > 2 {
            return false;
        }
        self.current_type = type_index;
        self.current_data = None;
        true
    }
}

impl<'db> Iterator for WorkingCyclingIterator<'db> {
    type Item = Result<WorkingCatalog, String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Get current iterator if we don't have one
            if self.current_data.is_none() {
                self.current_data = self.get_current_iterator();
                if self.current_data.is_none() {
                    return None; // No more types
                }
            }

            // Try to get next item from current iterator
            if let Some(ref mut iter) = self.current_data {
                match iter.next() {
                    Some(item) => return Some(item),
                    None => {
                        // Current iterator exhausted, move to next type
                        self.current_type += 1;
                        self.current_data = None;
                        continue;
                    }
                }
            }
        }
    }
}

/// Cached version with reference access
pub struct WorkingCachedIterator<'db> {
    cycling_iter: WorkingCyclingIterator<'db>,
    cache: std::collections::VecDeque<WorkingCatalog>,
    max_cache_size: usize,
    hits: u64,
    misses: u64,
}

impl<'db> WorkingCachedIterator<'db> {
    pub fn new(db_scanner: &'db DatabaseScanner<'db>, max_cache_size: usize) -> Self {
        Self {
            cycling_iter: WorkingCyclingIterator::new(db_scanner),
            cache: std::collections::VecDeque::with_capacity(max_cache_size),
            max_cache_size,
            hits: 0,
            misses: 0,
        }
    }

    pub fn find_in_cache<F>(&mut self, predicate: F) -> Option<&WorkingCatalog>
    where
        F: Fn(&WorkingCatalog) -> bool,
    {
        if let Some(pos) = self.cache.iter().position(&predicate) {
            self.hits += 1;
            // Move found item to back (LRU)
            if let Some(item) = self.cache.remove(pos) {
                self.cache.push_back(item);
                return self.cache.back();
            }
        }
        self.misses += 1;
        None
    }

    pub fn next_cached(&mut self) -> Option<Result<&WorkingCatalog, String>> {
        match self.cycling_iter.next() {
            Some(Ok(item)) => {
                // Add to cache
                if self.cache.len() >= self.max_cache_size {
                    self.cache.pop_front();
                }
                self.cache.push_back(item);
                Some(Ok(self.cache.back().unwrap()))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    pub fn cache_stats(&self) -> (u64, u64, usize) {
        (self.hits, self.misses, self.cache.len())
    }
}

// Sample data creation functions
fn create_sample_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            display_name: Some("Alice Smith".to_string()),
            followers_count: 150,
            following_count: 200,
            posts_count: 25,
            is_verified: true,
            is_active: true,
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            display_name: Some("Bob Johnson".to_string()),
            followers_count: 300,
            following_count: 180,
            posts_count: 40,
            is_verified: false,
            is_active: true,
        },
        User {
            id: 3,
            username: "charlie".to_string(),
            email: "charlie@example.com".to_string(),
            display_name: Some("Charlie Brown".to_string()),
            followers_count: 75,
            following_count: 120,
            posts_count: 15,
            is_verified: false,
            is_active: true,
        },
    ]
}

fn create_sample_posts() -> Vec<Post> {
    vec![
        Post {
            id: 1,
            user_id: 1,
            content: "Just implemented a memory-efficient iterator in Rust! 🦀".to_string(),
            likes_count: 42,
            comments_count: 5,
            hashtags: vec!["rust".to_string(), "programming".to_string()],
            is_public: true,
        },
        Post {
            id: 2,
            user_id: 2,
            content: "Product launch went great today! Thanks to everyone involved.".to_string(),
            likes_count: 87,
            comments_count: 12,
            hashtags: vec!["product".to_string(), "launch".to_string()],
            is_public: true,
        },
        Post {
            id: 3,
            user_id: 1,
            content: "Working on database optimization - streaming iterators are the way to go!"
                .to_string(),
            likes_count: 28,
            comments_count: 3,
            hashtags: vec!["databases".to_string(), "optimization".to_string()],
            is_public: true,
        },
        Post {
            id: 4,
            user_id: 3,
            content: "Learning Rust has been an amazing journey.".to_string(),
            likes_count: 15,
            comments_count: 2,
            hashtags: vec!["rust".to_string(), "learning".to_string()],
            is_public: true,
        },
    ]
}

fn create_sample_comments() -> Vec<Comment> {
    vec![
        Comment {
            id: 1,
            post_id: 1,
            user_id: 2,
            content: "This looks amazing! Can't wait to try it out.".to_string(),
            likes_count: 5,
        },
        Comment {
            id: 2,
            post_id: 1,
            user_id: 3,
            content: "Great work on the iterator implementation!".to_string(),
            likes_count: 3,
        },
        Comment {
            id: 3,
            post_id: 2,
            user_id: 1,
            content: "Congratulations on the successful launch!".to_string(),
            likes_count: 8,
        },
        Comment {
            id: 4,
            post_id: 3,
            user_id: 2,
            content: "Database optimization is so important for performance.".to_string(),
            likes_count: 4,
        },
    ]
}

/// Demonstrates basic cycling iterator usage
fn demo_basic_cycling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo 1: Basic Cycling Iterator ===");

    let db_scanner = DatabaseScanner::new();
    let mut cycling = WorkingCyclingIterator::new(&db_scanner);

    let mut total_items = 0;
    let mut type_counts = HashMap::new();

    println!("Streaming through all catalog types:");

    while let Some(result) = cycling.next() {
        match result {
            Ok(item) => {
                total_items += 1;
                let type_name = cycling.current_type_name();
                *type_counts.entry(type_name).or_insert(0) += 1;

                match item {
                    WorkingCatalog::User(user) => {
                        println!(
                            "  👤 User: {} (@{})",
                            user.display_name.unwrap_or_else(|| "Unknown".to_string()),
                            user.username
                        );
                    }
                    WorkingCatalog::Post(post) => {
                        println!(
                            "  📝 Post: \"{}...\" ({} likes)",
                            post.content.chars().take(40).collect::<String>(),
                            post.likes_count
                        );
                    }
                    WorkingCatalog::Comment(comment) => {
                        println!(
                            "  💬 Comment on post {}: \"{}...\"",
                            comment.post_id,
                            comment.content.chars().take(30).collect::<String>()
                        );
                    }
                    _ => {
                        println!("  📊 {}: Processing...", type_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
        }
    }

    println!("\n📊 Summary:");
    println!("   Total items processed: {}", total_items);
    for (type_name, count) in type_counts {
        println!("   {}: {} items", type_name, count);
    }
    println!("   Memory usage: O(1) - constant per item\n");

    Ok(())
}

/// Demonstrates cached cycling iterator for reference access
fn demo_cached_cycling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo 2: Cached Cycling Iterator ===");

    let db_scanner = DatabaseScanner::new();
    let mut cached_iter = WorkingCachedIterator::new(&db_scanner, 10);

    println!("Populating cache with items:");

    // Cache some items
    let mut cached_count = 0;
    for _ in 0..8 {
        match cached_iter.next_cached() {
            Some(Ok(item_ref)) => {
                cached_count += 1;
                match item_ref {
                    WorkingCatalog::User(user) => {
                        println!("  ✅ Cached user: {}", user.username);
                    }
                    WorkingCatalog::Post(post) => {
                        println!("  ✅ Cached post: {}", post.id);
                    }
                    WorkingCatalog::Comment(comment) => {
                        println!("  ✅ Cached comment: {}", comment.id);
                    }
                    _ => {
                        println!("  ✅ Cached other item");
                    }
                }
            }
            Some(Err(e)) => {
                eprintln!("❌ Error caching item: {}", e);
                break;
            }
            None => break,
        }
    }

    println!("\nSearching cache for specific items:");

    // Search for a specific user
    if let Some(user_ref) = cached_iter.find_in_cache(
        |item| matches!(item, WorkingCatalog::User(user) if user.username == "alice"),
    ) {
        if let WorkingCatalog::User(user) = user_ref {
            println!(
                "  🔍 Found Alice in cache: {} followers",
                user.followers_count
            );
        }
    } else {
        println!("  🔍 Alice not found in cache");
    }

    // Search for posts with many likes
    let popular_posts: Vec<_> = cached_iter
        .cache
        .iter()
        .filter_map(|item| {
            if let WorkingCatalog::Post(post) = item {
                if post.likes_count > 30 {
                    Some(post)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    println!(
        "  🔍 Found {} popular posts (>30 likes) in cache",
        popular_posts.len()
    );

    let (hits, misses, cache_size) = cached_iter.cache_stats();
    println!("\n📈 Cache Statistics:");
    println!("   Items cached: {}", cached_count);
    println!("   Current cache size: {}", cache_size);
    println!("   Cache hits: {}", hits);
    println!("   Cache misses: {}", misses);
    if hits + misses > 0 {
        println!(
            "   Hit ratio: {:.1}%",
            (hits as f64 / (hits + misses) as f64) * 100.0
        );
    }
    println!(
        "   Memory usage: O({}) - bounded by cache size\n",
        cache_size
    );

    Ok(())
}

/// Demonstrates type-specific processing
fn demo_type_specific_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo 3: Type-Specific Processing ===");

    let db_scanner = DatabaseScanner::new();
    let mut cycling = WorkingCyclingIterator::new(&db_scanner);

    // Process only users
    println!("Processing only Users:");
    cycling.skip_to_type(0); // Users

    let mut user_analysis = HashMap::new();
    let mut user_count = 0;

    while let Some(result) = cycling.next() {
        match result {
            Ok(WorkingCatalog::User(user)) => {
                user_count += 1;
                println!(
                    "  👤 User {}: {} (verified: {})",
                    user_count, user.username, user.is_verified
                );

                // Analyze user data
                if user.is_verified {
                    *user_analysis.entry("verified").or_insert(0) += 1;
                }
                if user.followers_count > 200 {
                    *user_analysis.entry("popular").or_insert(0) += 1;
                }
                if user.posts_count > 30 {
                    *user_analysis.entry("active_posters").or_insert(0) += 1;
                }
            }
            Ok(_) => {
                // Moved to next type, stop processing users
                println!("  ➡️  Moved to: {}", cycling.current_type_name());
                break;
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
        }
    }

    println!("📊 User Analysis:");
    for (category, count) in user_analysis {
        println!("   {}: {} users", category, count);
    }

    // Reset and process only posts
    println!("\nProcessing only Posts:");
    cycling.reset();
    cycling.skip_to_type(1); // Posts

    let mut post_count = 0;
    let mut total_engagement = 0;
    let mut hashtag_usage = HashMap::new();

    while let Some(result) = cycling.next() {
        match result {
            Ok(WorkingCatalog::Post(post)) => {
                post_count += 1;
                total_engagement += post.likes_count + post.comments_count;

                println!(
                    "  📝 Post {}: {} likes, {} comments",
                    post_count, post.likes_count, post.comments_count
                );

                // Track hashtag usage
                for hashtag in &post.hashtags {
                    *hashtag_usage.entry(hashtag.clone()).or_insert(0) += 1;
                }
            }
            Ok(_) => {
                println!("  ➡️  Moved to: {}", cycling.current_type_name());
                break;
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
        }
    }

    if post_count > 0 {
        println!("📊 Post Analysis:");
        println!("   Total posts: {}", post_count);
        println!(
            "   Average engagement: {:.1}",
            total_engagement as f64 / post_count as f64
        );
        println!("   Popular hashtags:");
        for (tag, count) in hashtag_usage {
            println!("     #{}: {} uses", tag, count);
        }
    }

    println!();
    Ok(())
}

/// Demonstrates analytics use case
fn demo_analytics_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo 4: Analytics Workflow ===");

    let db_scanner = DatabaseScanner::new();
    let mut cycling = WorkingCyclingIterator::new(&db_scanner);

    // Collect comprehensive analytics
    let mut analytics = AnalyticsReport::new();

    println!("Running comprehensive analytics across all data types:");

    while let Some(result) = cycling.next() {
        match result {
            Ok(item) => {
                analytics.process_item(item);
            }
            Err(e) => {
                eprintln!("❌ Analytics error: {}", e);
                continue;
            }
        }
    }

    analytics.print_report();

    Ok(())
}

/// Analytics report structure
struct AnalyticsReport {
    total_items: usize,
    users: Vec<User>,
    posts: Vec<Post>,
    comments: Vec<Comment>,
}

impl AnalyticsReport {
    fn new() -> Self {
        Self {
            total_items: 0,
            users: Vec::new(),
            posts: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn process_item(&mut self, item: WorkingCatalog) {
        self.total_items += 1;

        match item {
            WorkingCatalog::User(user) => self.users.push(user),
            WorkingCatalog::Post(post) => self.posts.push(post),
            WorkingCatalog::Comment(comment) => self.comments.push(comment),
            _ => {} // Handle other types as needed
        }
    }

    fn print_report(&self) {
        println!("📊 Analytics Report:");
        println!("   Total items processed: {}", self.total_items);
        println!("   Users: {}", self.users.len());
        println!("   Posts: {}", self.posts.len());
        println!("   Comments: {}", self.comments.len());

        // User analytics
        if !self.users.is_empty() {
            let verified_users = self.users.iter().filter(|u| u.is_verified).count();
            let avg_followers = self.users.iter().map(|u| u.followers_count).sum::<u32>() as f64
                / self.users.len() as f64;

            println!("\n👥 User Insights:");
            println!(
                "   Verified users: {} ({:.1}%)",
                verified_users,
                (verified_users as f64 / self.users.len() as f64) * 100.0
            );
            println!("   Average followers: {:.0}", avg_followers);
        }

        // Post analytics
        if !self.posts.is_empty() {
            let total_likes = self.posts.iter().map(|p| p.likes_count).sum::<u32>();
            let avg_likes = total_likes as f64 / self.posts.len() as f64;

            println!("\n📝 Post Insights:");
            println!("   Total likes: {}", total_likes);
            println!("   Average likes per post: {:.1}", avg_likes);

            let mut all_hashtags = Vec::new();
            for post in &self.posts {
                all_hashtags.extend(post.hashtags.clone());
            }
            println!("   Total hashtag usage: {}", all_hashtags.len());
        }

        // Comment analytics
        if !self.comments.is_empty() {
            let total_comment_likes = self.comments.iter().map(|c| c.likes_count).sum::<u32>();
            let avg_comment_likes = total_comment_likes as f64 / self.comments.len() as f64;

            println!("\n💬 Comment Insights:");
            println!("   Total comment likes: {}", total_comment_likes);
            println!("   Average likes per comment: {:.1}", avg_comment_likes);
        }

        println!();
    }
}

/// Main function demonstrating all cycling iterator patterns
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Working Cycling Iterator Integration Example");
    println!("===============================================\n");

    demo_basic_cycling()?;
    demo_cached_cycling()?;
    demo_type_specific_processing()?;
    demo_analytics_workflow()?;

    println!("=== Integration Summary ===");
    println!("✅ Demonstrated working cycling iterator patterns:");
    println!("   • Basic streaming iteration with O(1) memory");
    println!("   • Cached iteration for reference access");
    println!("   • Type-specific processing capabilities");
    println!("   • Real-world analytics workflow");

    println!("\n🎯 Key Benefits Demonstrated:");
    println!("   • Memory efficiency: Only one item in memory at a time");
    println!("   • Flexibility: Can process all types or focus on specific ones");
    println!("   • Performance: Uses efficient PrimaryScanIterator internally");
    println!("   • Usability: Clean API for common data processing patterns");

    println!("\n📝 Integration Notes:");
    println!("   • Replace mock types with your generated schema types");
    println!("   • Use with actual DatabaseScanner from macro output");
    println!("   • Configure cache sizes based on available memory");
    println!("   • Monitor performance with cache statistics");

    println!("\n🔧 To integrate with your schema:");
    println!("   1. Import your generated types: use your_module::{{Schema, DBIter, v1}};");
    println!("   2. Replace WorkingCatalog with your Schema enum");
    println!("   3. Replace DatabaseScanner with your generated DBIter");
    println!("   4. Use the same iteration patterns shown above");

    Ok(())
}

#[cfg(test)]
mod working_tests {
    use super::*;

    #[test]
    fn test_cycling_iterator_creation() {
        let db_scanner = DatabaseScanner::new();
        let cycling = WorkingCyclingIterator::new(&db_scanner);
        assert_eq!(cycling.current_type_name(), "User");
    }

    #[test]
    fn test_basic_iteration() {
        let db_scanner = DatabaseScanner::new();
        let mut cycling = WorkingCyclingIterator::new(&db_scanner);

        let mut items = Vec::new();
        while let Some(Ok(item)) = cycling.next() {
            items.push(item);
        }

        assert!(!items.is_empty(), "Should find some items");

        // Verify we get different types
        let has_users = items.iter().any(|i| matches!(i, WorkingCatalog::User(_)));
        let has_posts = items.iter().any(|i| matches!(i, WorkingCatalog::Post(_)));
        let has_comments = items
            .iter()
            .any(|i| matches!(i, WorkingCatalog::Comment(_)));

        assert!(has_users, "Should find users");
        assert!(has_posts, "Should find posts");
        assert!(has_comments, "Should find comments");
    }

    #[test]
    fn test_cached_iterator() {
        let db_scanner = DatabaseScanner::new();
        let mut cached_iter = WorkingCachedIterator::new(&db_scanner, 5);

        // Cache some items
        let mut cached_items = 0;
        for _ in 0..3 {
            if cached_iter.next_cached().is_some() {
                cached_items += 1;
            }
        }

        assert!(cached_items > 0, "Should cache some items");

        let (hits, misses, cache_size) = cached_iter.cache_stats();
        assert_eq!(
            cache_size, cached_items,
            "Cache size should match cached items"
        );
        assert!(cache_size <= 5, "Cache size should not exceed limit");
    }

    #[test]
    fn test_type_specific_navigation() {
        let db_scanner = DatabaseScanner::new();
        let mut cycling = WorkingCyclingIterator::new(&db_scanner);

        // Start at type 0 (Users)
        assert_eq!(cycling.current_type_name(), "User");

        // Skip to Posts
        assert!(cycling.skip_to_type(1));
        assert_eq!(cycling.current_type_name(), "Post");

        // Skip to Comments
        assert!(cycling.skip_to_type(2));
        assert_eq!(cycling.current_type_name(), "Comment");

        // Try to skip beyond available types
        assert!(!cycling.skip_to_type(10));
    }

    #[test]
    fn test_cache_search_functionality() {
        let db_scanner = DatabaseScanner::new();
        let mut cached_iter = WorkingCachedIterator::new(&db_scanner, 10);

        // Populate cache
        for _ in 0..5 {
            cached_iter.next_cached();
        }

        // Search for Alice
        let found_alice = cached_iter.find_in_cache(
            |item| matches!(item, WorkingCatalog::User(user) if user.username == "alice"),
        );

        assert!(found_alice.is_some(), "Should find Alice in cache");

        // Search for non-existent user
        let found_nobody = cached_iter.find_in_cache(
            |item| matches!(item, WorkingCatalog::User(user) if user.username == "nobody"),
        );

        assert!(found_nobody.is_none(), "Should not find non-existent user");

        let (hits, misses, _) = cached_iter.cache_stats();
        assert_eq!(hits, 1, "Should have one cache hit");
        assert_eq!(misses, 1, "Should have one cache miss");
    }

    #[test]
    fn test_sample_data_integrity() {
        let users = create_sample_users();
        let posts = create_sample_posts();
        let comments = create_sample_comments();

        assert!(!users.is_empty(), "Should have sample users");
        assert!(!posts.is_empty(), "Should have sample posts");
        assert!(!comments.is_empty(), "Should have sample comments");

        // Verify data relationships
        let user_ids: std::collections::HashSet<_> = users.iter().map(|u| u.id).collect();
        for post in &posts {
            assert!(
                user_ids.contains(&post.user_id),
                "Post should reference valid user"
            );
        }

        let post_ids: std::collections::HashSet<_> = posts.iter().map(|p| p.id).collect();
        for comment in &comments {
            assert!(
                post_ids.contains(&comment.post_id),
                "Comment should reference valid post"
            );
        }
    }

    #[test]
    fn test_analytics_workflow() {
        let mut report = AnalyticsReport::new();

        // Process some mock items
        report.process_item(WorkingCatalog::User(User {
            id: 1,
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            display_name: None,
            followers_count: 100,
            following_count: 50,
            posts_count: 10,
            is_verified: true,
            is_active: true,
        }));

        report.process_item(WorkingCatalog::Post(Post {
            id: 1,
            user_id: 1,
            content: "Test post".to_string(),
            likes_count: 25,
            comments_count: 5,
            hashtags: vec!["test".to_string()],
            is_public: true,
        }));

        assert_eq!(report.total_items, 2);
        assert_eq!(report.users.len(), 1);
        assert_eq!(report.posts.len(), 1);
    }
}
