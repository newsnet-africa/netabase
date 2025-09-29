//! Real schema integration example
//!
//! This example shows how to integrate the cycling iterator with the actual
//! generated SocialMediaSchema from the netabase macro system.

// Import the generated types from expanded_output.rs
// In a real project, these would come from your generated module
use std::collections::HashMap;
use std::time::Instant;

// Note: In practice, you would import from your generated module like:
// use crate::social_data::{SocialMediaSchema, SocialMediaSchemaDBIter, v1};

/// This example demonstrates integration with the actual generated schema
/// from the netabase macro system. It shows the complete pattern for
/// implementing cycling iterators with real types.

/// Implementation of cycling iterator for SocialMediaSchema
///
/// To use this in your project, add this implementation in your main module:
///
/// ```rust
/// use netabase::cycling_iterator_adapter::{impl_cycling_iterator, CacheConfig, CacheStats};
/// use your_generated_module::{SocialMediaSchema, SocialMediaSchemaDBIter, v1};
///
/// impl_cycling_iterator! {
///     schema = SocialMediaSchema,
///     scanner = SocialMediaSchemaDBIter<'db, 'stack_db>,
///     types = [
///         (0, PrimitiveTest, v1::PrimitiveTest, scan_type_0),
///         (1, TestUnit, v1::TestUnit, scan_type_1),
///         (2, TestTuple, v1::TestTuple, scan_type_2),
///         (3, User, v1::User, scan_type_3),
///         (4, Post, v1::Post, scan_type_4),
///         (5, Comment, v1::Comment, scan_type_5),
///         (6, Media, v1::Media, scan_type_6),
///         (7, Reaction, v1::Reaction, scan_type_7),
///         (8, Notification, v1::Notification, scan_type_8),
///         (9, UserStats, v1::UserStats, scan_type_9),
///         (10, HashTag, v1::HashTag, scan_type_10),
///     ]
/// }
/// ```

/// Example 1: Basic streaming through all catalog types
fn example_streaming_all_types() {
    println!("=== Example 1: Streaming All Types ===");

    // This is how you would use it with the real generated types:
    /*
    use native_db::{Database, Models};

    // Set up database
    let mut models = Models::new();
    models.define::<v1::User>()?;
    models.define::<v1::Post>()?;
    models.define::<v1::Comment>()?;
    models.define::<v1::Media>()?;
    models.define::<v1::Reaction>()?;
    models.define::<v1::Notification>()?;
    models.define::<v1::UserStats>()?;
    models.define::<v1::HashTag>()?;
    models.define::<v1::PrimitiveTest>()?;
    models.define::<v1::TestUnit>()?;
    models.define::<v1::TestTuple>()?;

    let db = Database::builder().create(&models, "path/to/db")?;

    // Create the database scanner
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;

    // Create cycling iterator
    let mut cycling = db_iter.cycling_iter();

    let mut total_processed = 0;
    let mut type_counts = HashMap::new();

    // Stream through all types with O(1) memory usage
    while let Some(result) = cycling.next() {
        match result {
            Ok(item) => {
                total_processed += 1;
                let type_name = cycling.current_type_name();
                *type_counts.entry(type_name).or_insert(0) += 1;

                // Process based on specific type
                match item {
                    SocialMediaSchema::User(user) => {
                        println!("👤 User: {} ({})",
                                user.display_name.unwrap_or_default(),
                                user.username);

                        // Access user-specific fields
                        if user.is_verified {
                            println!("  ✅ Verified user");
                        }

                        if user.posts_count > 100 {
                            println!("  📝 Power user with {} posts", user.posts_count);
                        }
                    }

                    SocialMediaSchema::Post(post) => {
                        println!("📝 Post by user {}: \"{}...\"",
                                post.user_id,
                                post.content.chars().take(50).collect::<String>());

                        // Access post-specific fields
                        println!("  💖 {} likes, 💬 {} comments",
                                post.likes_count, post.comments_count);

                        if !post.hashtags.is_empty() {
                            println!("  🏷️  Tags: {}", post.hashtags.join(", "));
                        }
                    }

                    SocialMediaSchema::Comment(comment) => {
                        println!("💬 Comment on post {}: \"{}...\"",
                                comment.post_id,
                                comment.content.chars().take(30).collect::<String>());
                    }

                    SocialMediaSchema::Media(media) => {
                        println!("🖼️  Media: {} ({})", media.filename, media.media_type);
                    }

                    SocialMediaSchema::Reaction(reaction) => {
                        println!("👍 Reaction: {} on target {}",
                                reaction.reaction_type, reaction.target_id);
                    }

                    SocialMediaSchema::Notification(notification) => {
                        println!("🔔 Notification for user {}: {}",
                                notification.user_id, notification.title);
                    }

                    SocialMediaSchema::UserStats(stats) => {
                        println!("📊 Stats for user {}: {} posts created",
                                stats.user_id, stats.posts_created);
                    }

                    SocialMediaSchema::HashTag(hashtag) => {
                        println!("🏷️  #{}: {} uses{}",
                                hashtag.tag,
                                hashtag.usage_count,
                                if hashtag.is_trending { " (trending)" } else { "" });
                    }

                    SocialMediaSchema::PrimitiveTest(primitive) => {
                        println!("🧪 PrimitiveTest: {}", primitive.text);
                    }

                    SocialMediaSchema::TestUnit(unit) => {
                        println!("🔧 TestUnit: {}", unit.id);
                    }

                    SocialMediaSchema::TestTuple(tuple) => {
                        println!("📦 TestTuple: {:?}", tuple.field_0);
                    }
                }

                // Item goes out of scope here, freeing memory immediately
            }
            Err(e) => {
                eprintln!("❌ Database error: {:?}", e);
                // Continue processing or abort based on error severity
                continue;
            }
        }
    }

    println!("\n📊 Processing Summary:");
    println!("  Total items processed: {}", total_processed);
    println!("  Memory usage: O(1) - constant per item");

    for (type_name, count) in type_counts {
        println!("  {}: {} items", type_name, count);
    }
    */

    println!("Streaming iteration provides memory-efficient access to all catalog types.");
    println!("Memory usage remains constant regardless of dataset size.\n");
}

/// Example 2: Cached iteration for interactive applications
fn example_cached_for_interactive_use() {
    println!("=== Example 2: Cached Iteration for Interactive Use ===");

    /*
    // Configure cache for interactive application
    let interactive_config = CacheConfig {
        max_items_per_type: 50,   // 50 items per type
        max_total_items: 300,     // 300 total items in cache
    };

    let db_iter = SocialMediaSchemaDBIter::new(&database)?;
    let mut cached_cycling = db_iter.cached_cycling_iter_with_config(interactive_config);

    // Pre-populate cache with recent data
    println!("🔄 Pre-populating cache for interactive queries...");
    let initial_batch = cached_cycling.collect_batch(200);
    println!("✅ Cached {} items for fast access", initial_batch.len());

    // Now you can perform fast lookups using references
    println!("\n🔍 Performing interactive searches:");

    // Search for a specific user
    if let Some(user_ref) = cached_cycling.find_in_cache(|item| {
        matches!(item, SocialMediaSchema::User(user)
                if user.username == "target_username")
    }) {
        if let SocialMediaSchema::User(user) = user_ref {
            println!("👤 Found user: {} ({})",
                    user.display_name.unwrap_or_default(),
                    user.email);
            println!("  📊 {} followers, {} following",
                    user.followers_count, user.following_count);
        }
    }

    // Search for posts with specific hashtags
    let rust_posts: Vec<_> = cached_cycling.cache
        .iter()
        .filter_map(|item| {
            if let SocialMediaSchema::Post(post) = item {
                if post.hashtags.contains(&"rust".to_string()) {
                    Some(post)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    println!("🦀 Found {} Rust-related posts in cache", rust_posts.len());

    // Search for verified users
    let verified_users: Vec<_> = cached_cycling.cache
        .iter()
        .filter_map(|item| {
            if let SocialMediaSchema::User(user) = item {
                if user.is_verified {
                    Some(user)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    println!("✅ Found {} verified users in cache", verified_users.len());

    // Monitor cache performance
    let stats = cached_cycling.cache_stats();
    println!("\n📈 Cache Performance:");
    println!("  Current size: {} / {}", stats.current_size, interactive_config.max_total_items);
    println!("  Cache hits: {}", stats.hits);
    println!("  Cache misses: {}", stats.misses);
    println!("  Evictions: {}", stats.evictions);

    if stats.hits + stats.misses > 0 {
        let hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0;
        println!("  Hit ratio: {:.1}%", hit_ratio);

        if hit_ratio < 50.0 {
            println!("  💡 Consider increasing cache size for better performance");
        } else if hit_ratio > 90.0 {
            println!("  💡 Cache is very effective!");
        }
    }
    */

    println!("Cached iteration enables fast reference-based access to recent data.");
    println!("Memory usage is bounded by cache configuration.\n");
}

/// Example 3: Type-specific analytics workflow
fn example_type_specific_analytics() {
    println!("=== Example 3: Type-specific Analytics ===");

    /*
    let db_iter = SocialMediaSchemaDBIter::new(&database)?;
    let mut cycling = db_iter.cycling_iter();

    // Analyze Users
    println!("📊 Analyzing user data...");
    cycling.skip_to_type(3)?; // Jump to User type

    let mut user_analytics = UserAnalytics::new();

    while let Some(result) = cycling.next() {
        match result {
            Ok(SocialMediaSchema::User(user)) => {
                user_analytics.process_user(user);
            }
            Ok(_) => {
                // Moved to next type, finish user analysis
                println!("✅ Finished analyzing users, moved to: {}",
                        cycling.current_type_name());
                break;
            }
            Err(e) => {
                eprintln!("❌ Error processing user: {:?}", e);
                continue;
            }
        }
    }

    user_analytics.print_report();

    // Analyze Posts
    println!("\n📊 Analyzing post data...");
    // No need to reset - we're already at the next type (Posts)

    let mut post_analytics = PostAnalytics::new();

    while let Some(result) = cycling.next() {
        match result {
            Ok(SocialMediaSchema::Post(post)) => {
                post_analytics.process_post(post);
            }
            Ok(_) => {
                // Moved to next type, finish post analysis
                println!("✅ Finished analyzing posts, moved to: {}",
                        cycling.current_type_name());
                break;
            }
            Err(e) => {
                eprintln!("❌ Error processing post: {:?}", e);
                continue;
            }
        }
    }

    post_analytics.print_report();

    // Analyze engagement patterns across types
    println!("\n📊 Cross-type engagement analysis...");
    cycling.reset(); // Start over for comprehensive analysis

    let mut engagement_analyzer = EngagementAnalyzer::new();

    while let Some(result) = cycling.next() {
        if let Ok(item) = result {
            engagement_analyzer.process_item(item);
        }
    }

    engagement_analyzer.print_cross_type_insights();
    */

    println!("Type-specific analytics allows focused analysis of individual data types");
    println!("while maintaining memory efficiency through streaming access.\n");
}

/// Example 4: Production ETL pipeline
fn example_production_etl_pipeline() {
    println!("=== Example 4: Production ETL Pipeline ===");

    /*
    let source_db_iter = SocialMediaSchemaDBIter::new(&source_database)?;
    let mut cycling = source_db_iter.cycling_iter();

    // ETL pipeline with error handling and monitoring
    let mut etl_monitor = ETLMonitor::new();
    let chunk_size = 1000;

    println!("🚀 Starting ETL pipeline...");

    loop {
        let chunk_start = Instant::now();
        let mut chunk = Vec::with_capacity(chunk_size);

        // Collect a chunk for batch processing
        for _ in 0..chunk_size {
            match cycling.next() {
                Some(Ok(item)) => chunk.push(item),
                Some(Err(e)) => {
                    etl_monitor.record_error(e);
                    continue; // Skip problematic items
                }
                None => break, // End of data
            }
        }

        if chunk.is_empty() {
            break;
        }

        // Transform the chunk
        let transformed_chunk = transform_chunk(chunk)?;

        // Load into destination
        load_chunk_to_destination(transformed_chunk)?;

        etl_monitor.record_chunk_processed(chunk.len(), chunk_start.elapsed());

        // Periodic progress reporting
        if etl_monitor.total_processed() % 10000 == 0 {
            etl_monitor.print_progress();
        }
    }

    etl_monitor.print_final_report();
    */

    println!("ETL pipelines benefit from streaming iteration's predictable memory usage");
    println!("and ability to handle large datasets without memory pressure.\n");
}

/// Example 5: Real-time feed generation
fn example_realtime_feed_generation() {
    println!("=== Example 5: Real-time Feed Generation ===");

    /*
    let db_iter = SocialMediaSchemaDBIter::new(&database)?;

    // Use cached iterator for feed generation to enable fast lookups
    let feed_config = CacheConfig {
        max_items_per_type: 200,  // Keep recent items for feed generation
        max_total_items: 1000,    // Total feed cache
    };

    let mut feed_generator = db_iter.cached_cycling_iter_with_config(feed_config);

    // Build feed cache with recent content
    println!("🔄 Building feed cache...");
    feed_generator.collect_batch(500);

    // Generate personalized feed for a user
    let user_id = 123;
    let mut personalized_feed = Vec::new();

    // Find posts from users the target user follows
    let following_posts: Vec<_> = feed_generator.cache
        .iter()
        .filter_map(|item| {
            if let SocialMediaSchema::Post(post) = item {
                if is_user_following(user_id, post.user_id) {
                    Some(post)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    personalized_feed.extend(following_posts);

    // Find relevant hashtag content
    let user_interests = get_user_interests(user_id);
    let relevant_posts: Vec<_> = feed_generator.cache
        .iter()
        .filter_map(|item| {
            if let SocialMediaSchema::Post(post) = item {
                if post.hashtags.iter().any(|tag| user_interests.contains(tag)) {
                    Some(post)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    personalized_feed.extend(relevant_posts);

    // Include recent notifications
    let notifications: Vec<_> = feed_generator.cache
        .iter()
        .filter_map(|item| {
            if let SocialMediaSchema::Notification(notification) = item {
                if notification.user_id == user_id && !notification.is_read {
                    Some(notification)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    println!("🎯 Generated personalized feed:");
    println!("  📝 {} relevant posts", personalized_feed.len());
    println!("  🔔 {} unread notifications", notifications.len());

    let stats = feed_generator.cache_stats();
    println!("  📊 Cache utilization: {} items ({} hits, {} misses)",
            stats.current_size, stats.hits, stats.misses);
    */

    println!("Real-time feed generation benefits from cached iteration's fast lookup capabilities");
    println!("while maintaining bounded memory usage for production scalability.\n");
}

/// Example 6: Data validation and cleanup
fn example_data_validation_and_cleanup() {
    println!("=== Example 6: Data Validation and Cleanup ===");

    /*
    let db_iter = SocialMediaSchemaDBIter::new(&database)?;
    let mut cycling = db_iter.cycling_iter();

    let mut validation_report = ValidationReport::new();

    println!("🔍 Running data validation across all types...");

    while let Some(result) = cycling.next() {
        match result {
            Ok(item) => {
                match item {
                    SocialMediaSchema::User(user) => {
                        // Validate user data
                        if user.email.is_empty() || !user.email.contains('@') {
                            validation_report.record_invalid_user(user.id, "Invalid email");
                        }

                        if user.username.is_empty() {
                            validation_report.record_invalid_user(user.id, "Empty username");
                        }

                        if user.followers_count > 1_000_000 {
                            validation_report.record_suspicious_user(user.id, "Suspiciously high follower count");
                        }
                    }

                    SocialMediaSchema::Post(post) => {
                        // Validate post data
                        if post.content.is_empty() {
                            validation_report.record_invalid_post(post.id, "Empty content");
                        }

                        if post.content.len() > 10_000 {
                            validation_report.record_suspicious_post(post.id, "Content too long");
                        }

                        // Check for broken media links
                        for media_url in &post.media_urls {
                            if media_url.is_empty() || !media_url.starts_with("http") {
                                validation_report.record_invalid_post(post.id, "Invalid media URL");
                            }
                        }
                    }

                    SocialMediaSchema::Comment(comment) => {
                        // Validate comment data
                        if comment.content.is_empty() {
                            validation_report.record_invalid_comment(comment.id, "Empty content");
                        }

                        // Validate parent relationships
                        if let Some(parent_id) = comment.parent_comment_id {
                            if !comment_exists(parent_id) {
                                validation_report.record_orphaned_comment(comment.id, parent_id);
                            }
                        }
                    }

                    SocialMediaSchema::Media(media) => {
                        // Validate media data
                        if media.url.is_empty() {
                            validation_report.record_invalid_media(media.id, "Empty URL");
                        }

                        if media.size_bytes == 0 {
                            validation_report.record_suspicious_media(media.id, "Zero byte file");
                        }
                    }

                    _ => {
                        // Validate other types as needed
                        validation_report.record_processed_item();
                    }
                }
            }
            Err(e) => {
                validation_report.record_database_error(e);
            }
        }
    }

    validation_report.print_summary();

    // Generate cleanup scripts based on validation results
    if validation_report.has_issues() {
        validation_report.generate_cleanup_scripts();
    }
    */

    println!("Data validation workflows benefit from streaming access to scan entire datasets");
    println!("efficiently without loading everything into memory simultaneously.\n");
}

/// Example helper structs for analytics (would be implemented based on your needs)
#[allow(dead_code)]
struct UserAnalytics {
    total_users: usize,
    verified_count: usize,
    total_followers: u64,
    avg_posts_per_user: f64,
}

#[allow(dead_code)]
impl UserAnalytics {
    fn new() -> Self {
        Self {
            total_users: 0,
            verified_count: 0,
            total_followers: 0,
            avg_posts_per_user: 0.0,
        }
    }

    fn print_report(&self) {
        println!("👥 User Analytics Report:");
        println!("  Total users: {}", self.total_users);
        println!(
            "  Verified users: {} ({:.1}%)",
            self.verified_count,
            (self.verified_count as f64 / self.total_users as f64) * 100.0
        );
        println!(
            "  Average followers: {:.0}",
            self.total_followers as f64 / self.total_users as f64
        );
        println!("  Average posts per user: {:.1}", self.avg_posts_per_user);
    }
}

#[allow(dead_code)]
struct PostAnalytics {
    total_posts: usize,
    total_likes: u64,
    total_comments: u64,
    hashtag_usage: HashMap<String, u32>,
}

#[allow(dead_code)]
impl PostAnalytics {
    fn new() -> Self {
        Self {
            total_posts: 0,
            total_likes: 0,
            total_comments: 0,
            hashtag_usage: HashMap::new(),
        }
    }

    fn print_report(&self) {
        println!("📝 Post Analytics Report:");
        println!("  Total posts: {}", self.total_posts);
        println!(
            "  Average likes per post: {:.1}",
            self.total_likes as f64 / self.total_posts as f64
        );
        println!(
            "  Average comments per post: {:.1}",
            self.total_comments as f64 / self.total_posts as f64
        );

        let top_hashtags: Vec<_> = self.hashtag_usage.iter().collect::<Vec<_>>();

        println!("  Top hashtags: {:?}", top_hashtags);
    }
}

#[allow(dead_code)]
struct EngagementAnalyzer {
    cross_type_metrics: HashMap<String, u64>,
}

#[allow(dead_code)]
impl EngagementAnalyzer {
    fn new() -> Self {
        Self {
            cross_type_metrics: HashMap::new(),
        }
    }

    fn print_cross_type_insights(&self) {
        println!("🔗 Cross-type Engagement Insights:");
        for (metric, value) in &self.cross_type_metrics {
            println!("  {}: {}", metric, value);
        }
    }
}

#[allow(dead_code)]
struct ValidationReport {
    total_processed: usize,
    errors: Vec<String>,
}

#[allow(dead_code)]
impl ValidationReport {
    fn new() -> Self {
        Self {
            total_processed: 0,
            errors: Vec::new(),
        }
    }

    fn record_invalid_user(&mut self, user_id: u64, reason: &str) {
        self.errors
            .push(format!("Invalid user {}: {}", user_id, reason));
    }

    fn record_invalid_post(&mut self, post_id: u64, reason: &str) {
        self.errors
            .push(format!("Invalid post {}: {}", post_id, reason));
    }

    fn record_invalid_comment(&mut self, comment_id: u64, reason: &str) {
        self.errors
            .push(format!("Invalid comment {}: {}", comment_id, reason));
    }

    fn record_invalid_media(&mut self, media_id: u64, reason: &str) {
        self.errors
            .push(format!("Invalid media {}: {}", media_id, reason));
    }

    fn record_suspicious_user(&mut self, user_id: u64, reason: &str) {
        self.errors
            .push(format!("Suspicious user {}: {}", user_id, reason));
    }

    fn record_suspicious_post(&mut self, post_id: u64, reason: &str) {
        self.errors
            .push(format!("Suspicious post {}: {}", post_id, reason));
    }

    fn record_suspicious_media(&mut self, media_id: u64, reason: &str) {
        self.errors
            .push(format!("Suspicious media {}: {}", media_id, reason));
    }

    fn record_orphaned_comment(&mut self, comment_id: u64, parent_id: u64) {
        self.errors.push(format!(
            "Orphaned comment {} references non-existent parent {}",
            comment_id, parent_id
        ));
    }

    fn record_processed_item(&mut self) {
        self.total_processed += 1;
    }

    fn record_database_error(&mut self, error: native_db::db_type::Error) {
        self.errors.push(format!("Database error: {:?}", error));
    }

    fn has_issues(&self) -> bool {
        !self.errors.is_empty()
    }

    fn print_summary(&self) {
        println!("🔍 Validation Summary:");
        println!("  Items processed: {}", self.total_processed);
        println!("  Issues found: {}", self.errors.len());

        if !self.errors.is_empty() {
            println!("  Sample issues:");
            for (i, error) in self.errors.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, error);
            }

            if self.errors.len() > 5 {
                println!("    ... and {} more issues", self.errors.len() - 5);
            }
        }
    }

    fn generate_cleanup_scripts(&self) {
        println!("🧹 Generating cleanup scripts based on validation results...");
        // Implementation would generate SQL or other cleanup scripts
    }
}

#[allow(dead_code)]
struct ETLMonitor {
    start_time: Instant,
    total_processed: usize,
    total_errors: usize,
    chunk_times: Vec<std::time::Duration>,
}

#[allow(dead_code)]
impl ETLMonitor {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_processed: 0,
            total_errors: 0,
            chunk_times: Vec::new(),
        }
    }

    fn record_chunk_processed(&mut self, chunk_size: usize, duration: std::time::Duration) {
        self.total_processed += chunk_size;
        self.chunk_times.push(duration);
    }

    fn record_error(&mut self, _error: native_db::db_type::Error) {
        self.total_errors += 1;
    }

    fn total_processed(&self) -> usize {
        self.total_processed
    }

    fn print_progress(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = self.total_processed as f64 / elapsed.as_secs_f64();

        println!(
            "⏳ ETL Progress: {} items ({:.1} items/sec, {} errors)",
            self.total_processed, rate, self.total_errors
        );
    }

    fn print_final_report(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = self.total_processed as f64 / elapsed.as_secs_f64();
        let avg_chunk_time = if !self.chunk_times.is_empty() {
            self.chunk_times.iter().sum::<std::time::Duration>() / self.chunk_times.len() as u32
        } else {
            std::time::Duration::from_secs(0)
        };

        println!("✅ ETL Pipeline Complete:");
        println!("  Total processed: {}", self.total_processed);
        println!("  Total time: {:?}", elapsed);
        println!("  Average rate: {:.1} items/second", rate);
        println!("  Average chunk time: {:?}", avg_chunk_time);
        println!("  Total errors: {}", self.total_errors);
        println!(
            "  Success rate: {:.1}%",
            (self.total_processed as f64 / (self.total_processed + self.total_errors) as f64)
                * 100.0
        );
    }
}

// Mock helper functions (implement based on your application logic)
#[allow(dead_code)]
fn is_user_following(_user_id: u64, _target_user_id: u64) -> bool {
    // Implementation would check following relationships
    true
}

#[allow(dead_code)]
fn get_user_interests(_user_id: u64) -> Vec<String> {
    // Implementation would fetch user interests
    vec!["rust".to_string(), "programming".to_string()]
}

#[allow(dead_code)]
fn comment_exists(_comment_id: u64) -> bool {
    // Implementation would check if comment exists
    true
}

#[allow(dead_code)]
fn transform_chunk(_chunk: Vec<()>) -> Result<Vec<()>, Box<dyn std::error::Error>> {
    // Implementation would transform data for destination format
    Ok(vec![])
}

#[allow(dead_code)]
fn load_chunk_to_destination(_chunk: Vec<()>) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation would load data into destination system
    Ok(())
}

/// Main demonstration
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Real Schema Integration Example");
    println!("==================================\n");

    println!("This example shows how to integrate the cycling iterator with");
    println!("the actual generated SocialMediaSchema from netabase macros.\n");

    example_streaming_all_types();
    example_cached_for_interactive_use();
    example_type_specific_analytics();
    example_production_etl_pipeline();
    example_realtime_feed_generation();
    example_data_validation_and_cleanup();

    println!("=== Integration Summary ===");
    println!("✅ The cycling iterator provides:");
    println!("   • Memory-efficient streaming through all catalog types");
    println!("   • Optional bounded caching for reference access");
    println!("   • Type-specific processing capabilities");
    println!("   • Robust error handling for production use");
    println!("   • Performance monitoring and optimization");

    println!("\n🔧 To integrate with your schema:");
    println!("   1. Use the impl_cycling_iterator! macro with your generated types");
    println!("   2. Configure cache sizes based on your memory constraints");
    println!("   3. Choose streaming for ETL/analytics, cached for interactive use");
    println!("   4. Monitor cache statistics to optimize performance");
    println!("   5. Handle errors gracefully for robust production operation");

    println!("\n📚 Key Benefits:");
    println!("   • Works seamlessly with existing PrimaryScanIterator infrastructure");
    println!("   • Provides both owned and reference access patterns");
    println!("   • Scales from small interactive queries to large batch operations");
    println!("   • Maintains predictable memory usage characteristics");

    Ok(())
}

#[cfg(test)]
mod real_integration_tests {
    use super::*;

    #[test]
    fn test_example_functions_execute() {
        // Verify all example functions can execute without panicking
        example_streaming_all_types();
        example_cached_for_interactive_use();
        example_type_specific_analytics();
        example_production_etl_pipeline();
        example_realtime_feed_generation();
        example_data_validation_and_cleanup();
    }

    #[test]
    fn test_analytics_structs() {
        let user_analytics = UserAnalytics::new();
        assert_eq!(user_analytics.total_users, 0);

        let post_analytics = PostAnalytics::new();
        assert_eq!(post_analytics.total_posts, 0);

        let engagement_analyzer = EngagementAnalyzer::new();
        assert!(engagement_analyzer.cross_type_metrics.is_empty());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(!report.has_issues());

        report.record_invalid_user(1, "test");
        assert!(report.has_issues());

        report.record_processed_item();
        assert_eq!(report.total_processed, 1);
    }

    #[test]
    fn test_etl_monitor() {
        let mut monitor = ETLMonitor::new();
        assert_eq!(monitor.total_processed(), 0);

        monitor.record_chunk_processed(100, std::time::Duration::from_millis(50));
        assert_eq!(monitor.total_processed(), 100);

        // Should not panic
        monitor.print_progress();
        monitor.print_final_report();
    }

    #[test]
    fn test_helper_functions() {
        assert!(is_user_following(1, 2));
        assert!(!get_user_interests(1).is_empty());
        assert!(comment_exists(1));

        let transform_result = transform_chunk(vec![]);
        assert!(transform_result.is_ok());

        let load_result = load_chunk_to_destination(vec![]);
        assert!(load_result.is_ok());
    }
}
