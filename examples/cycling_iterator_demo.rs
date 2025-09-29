//! Comprehensive example demonstrating the cycling iterator functionality
//!
//! This example shows how to use the memory-efficient cycling iterator
//! to stream through catalog data without loading everything into memory.

use native_db::{Database, Models};
use netabase::cycling_iterator::{
    CacheConfig, CachedCyclingIterator, CyclingIterator, CyclingIteratorExt, SocialMediaSchema,
    SocialMediaSchemaDBIter, utils, v1,
};
use std::collections::HashMap;

// Mock data generators for demonstration
fn create_sample_users() -> Vec<v1::User> {
    vec![
        v1::User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            display_name: Some("Alice Smith".to_string()),
            bio: Some("Software engineer".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: None,
            birth_timestamp: None,
            last_active: Some(chrono::Utc::now().timestamp()),
            followers_count: 150,
            following_count: 200,
            posts_count: 50,
            age: Some(28),
            is_verified: true,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests: vec!["rust".to_string(), "tech".to_string()],
            languages: vec!["en".to_string()],
            settings: HashMap::new(),
        },
        v1::User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            display_name: Some("Bob Johnson".to_string()),
            bio: Some("Product manager".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: None,
            birth_timestamp: None,
            last_active: Some(chrono::Utc::now().timestamp()),
            followers_count: 300,
            following_count: 180,
            posts_count: 75,
            age: Some(32),
            is_verified: false,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests: vec!["business".to_string(), "travel".to_string()],
            languages: vec!["en".to_string(), "es".to_string()],
            settings: HashMap::new(),
        },
    ]
}

fn create_sample_posts() -> Vec<v1::Post> {
    vec![
        v1::Post {
            id: 1,
            user_id: 1,
            created_at: chrono::Utc::now().timestamp(),
            content: "Just shipped a new Rust feature!".to_string(),
            updated_at: None,
            media_urls: vec![],
            hashtags: vec!["rust".to_string(), "programming".to_string()],
            mentions: vec![],
            likes_count: 42,
            comments_count: 5,
            shares_count: 8,
            views_count: 200,
            is_public: true,
            allow_comments: true,
            allow_shares: true,
            latitude: None,
            longitude: None,
            location_name: None,
        },
        v1::Post {
            id: 2,
            user_id: 2,
            created_at: chrono::Utc::now().timestamp(),
            content: "Great product launch today!".to_string(),
            updated_at: None,
            media_urls: vec!["https://example.com/image.jpg".to_string()],
            hashtags: vec!["product".to_string(), "launch".to_string()],
            mentions: vec!["alice".to_string()],
            likes_count: 87,
            comments_count: 12,
            shares_count: 15,
            views_count: 450,
            is_public: true,
            allow_comments: true,
            allow_shares: true,
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            location_name: Some("San Francisco".to_string()),
        },
    ]
}

fn create_sample_comments() -> Vec<v1::Comment> {
    vec![
        v1::Comment {
            id: 1,
            post_id: 1,
            user_id: 2,
            created_at: chrono::Utc::now().timestamp(),
            content: "Awesome work on the Rust feature!".to_string(),
            parent_comment_id: None,
            likes_count: 12,
            replies_count: 2,
            is_edited: false,
            edited_at: None,
        },
        v1::Comment {
            id: 2,
            post_id: 2,
            user_id: 1,
            created_at: chrono::Utc::now().timestamp(),
            content: "Congratulations on the launch!".to_string(),
            parent_comment_id: None,
            likes_count: 8,
            replies_count: 0,
            is_edited: false,
            edited_at: None,
        },
    ]
}

/// Example 1: Basic cycling through all types
fn example_basic_cycling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Basic Cycling Iterator ===");

    // Note: In a real application, you would set up your database like this:
    // let mut models = Models::new();
    // models.define::<v1::User>()?;
    // models.define::<v1::Post>()?;
    // models.define::<v1::Comment>()?;
    // // ... define other types
    //
    // let db = Database::builder()
    //     .create(&models, "path/to/db")?;
    //
    // let db_iter = SocialMediaSchemaDBIter::new(&db)?;
    // let mut cycling = db_iter.cycling_iter();

    // Simulated iteration (showing the pattern)
    println!("Cycling through all catalog types:");

    // This is what the actual usage would look like:
    /*
    let mut total_count = 0;
    let mut type_counts = HashMap::new();

    while let Some(result) = cycling.next() {
        match result {
            Ok(item) => {
                total_count += 1;
                let type_name = cycling.current_type_name();
                *type_counts.entry(type_name).or_insert(0) += 1;

                // Process item based on its type
                match item {
                    SocialMediaSchema::User(user) => {
                        println!("User: {} ({})", user.display_name.unwrap_or_default(), user.username);
                    }
                    SocialMediaSchema::Post(post) => {
                        println!("Post: {} (by user {})",
                                 post.content.chars().take(50).collect::<String>(),
                                 post.user_id);
                    }
                    SocialMediaSchema::Comment(comment) => {
                        println!("Comment: {} (on post {})",
                                 comment.content.chars().take(30).collect::<String>(),
                                 comment.post_id);
                    }
                    _ => {
                        println!("Other type: {}", type_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Database error: {:?}", e);
                break;
            }
        }
    }

    println!("Total items processed: {}", total_count);
    for (type_name, count) in type_counts {
        println!("  {}: {} items", type_name, count);
    }
    */

    println!("Basic cycling iterator provides streaming access with minimal memory usage.");
    println!("Each item is processed individually without storing collections in memory.\n");

    Ok(())
}

/// Example 2: Using cached cycling iterator for reference access
fn example_cached_cycling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 2: Cached Cycling Iterator ===");

    // Custom cache configuration
    let cache_config = CacheConfig {
        max_items_per_type: 50,
        max_total_items: 200,
    };

    println!("Cache configuration:");
    println!("  Max items per type: {}", cache_config.max_items_per_type);
    println!("  Max total items: {}", cache_config.max_total_items);

    // This is what the actual usage would look like:
    /*
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;
    let mut cached_cycling = db_iter.cached_cycling_iter_with_config(cache_config);

    // Populate cache with a batch of items
    println!("Populating cache...");
    let batch = cached_cycling.collect_batch(100);
    println!("Collected {} items into cache", batch.len());

    // Now you can search the cache and get references
    println!("\nSearching cache for specific items:");

    // Find a user by username
    if let Some(user_ref) = cached_cycling.find_in_cache(|item| {
        matches!(item, SocialMediaSchema::User(user) if user.username == "alice")
    }) {
        if let SocialMediaSchema::User(user) = user_ref {
            println!("Found cached user: {} ({})", user.display_name.unwrap_or_default(), user.username);
            println!("  Followers: {}, Following: {}", user.followers_count, user.following_count);
        }
    }

    // Find posts with many likes
    let mut popular_posts = Vec::new();
    for item in &cached_cycling.cache {
        if let SocialMediaSchema::Post(post) = item {
            if post.likes_count > 50 {
                popular_posts.push(post);
            }
        }
    }
    println!("Found {} popular posts in cache", popular_posts.len());

    // Check cache performance
    let stats = cached_cycling.cache_stats();
    println!("\nCache statistics:");
    println!("  Current size: {}", stats.current_size);
    println!("  Cache hits: {}", stats.hits);
    println!("  Cache misses: {}", stats.misses);
    println!("  Evictions: {}", stats.evictions);

    // Demonstrate cache efficiency
    if stats.hits + stats.misses > 0 {
        let hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;
        println!("  Hit ratio: {:.2}%", hit_ratio * 100.0);
    }
    */

    println!("Cached cycling iterator allows reference access to recently seen items.");
    println!("Memory usage is bounded by the cache configuration.\n");

    Ok(())
}

/// Example 3: Type-specific iteration
fn example_type_specific() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 3: Type-Specific Iteration ===");

    // This is what the actual usage would look like:
    /*
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;
    let mut cycling = db_iter.cycling_iter();

    // Process only Users (type index 3)
    println!("Processing only Users...");
    cycling.skip_to_type(3)?;

    let mut user_count = 0;
    while let Some(result) = cycling.next() {
        match result {
            Ok(SocialMediaSchema::User(user)) => {
                user_count += 1;
                println!("User {}: {} ({})",
                         user_count,
                         user.display_name.unwrap_or_default(),
                         user.username);

                if user.is_verified {
                    println!("  ✓ Verified user");
                }

                if user.posts_count > 100 {
                    println!("  📝 Power user ({} posts)", user.posts_count);
                }
            }
            Ok(_) => {
                // We've moved to the next type, stop processing users
                println!("Finished processing users, moved to: {}", cycling.current_type_name());
                break;
            }
            Err(e) => {
                eprintln!("Database error: {:?}", e);
                break;
            }
        }
    }

    println!("Processed {} users total", user_count);

    // Reset and process only Posts (type index 4)
    println!("\nProcessing only Posts...");
    cycling.reset();
    cycling.skip_to_type(4)?;

    let mut post_count = 0;
    let mut total_likes = 0;

    while let Some(result) = cycling.next() {
        match result {
            Ok(SocialMediaSchema::Post(post)) => {
                post_count += 1;
                total_likes += post.likes_count;

                println!("Post {}: \"{}...\" ({} likes)",
                         post_count,
                         post.content.chars().take(30).collect::<String>(),
                         post.likes_count);

                if !post.hashtags.is_empty() {
                    println!("  Tags: {}", post.hashtags.join(", "));
                }
            }
            Ok(_) => {
                println!("Finished processing posts, moved to: {}", cycling.current_type_name());
                break;
            }
            Err(e) => {
                eprintln!("Database error: {:?}", e);
                break;
            }
        }
    }

    if post_count > 0 {
        println!("Processed {} posts total", post_count);
        println!("Average likes per post: {:.1}", total_likes as f64 / post_count as f64);
    }
    */

    println!("Type-specific iteration allows focused processing of single catalog types.");
    println!(
        "You can skip to any type and process items until the iterator moves to the next type.\n"
    );

    Ok(())
}

/// Example 4: Advanced cache usage patterns
fn example_advanced_cache_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 4: Advanced Cache Usage ===");

    // This demonstrates more sophisticated cache usage patterns:
    /*
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;

    // Create a small, focused cache for interactive queries
    let interactive_config = CacheConfig {
        max_items_per_type: 20,
        max_total_items: 100,
    };

    let mut interactive_cache = db_iter.cached_cycling_iter_with_config(interactive_config);

    // Build up cache with recent items
    println!("Building interactive cache...");
    let initial_batch = interactive_cache.collect_batch(50);
    println!("Cached {} items for interactive queries", initial_batch.len());

    // Simulate user searches in cache
    println!("\nSimulating user search queries:");

    // Search for verified users
    let mut verified_users = Vec::new();
    for item in &interactive_cache.cache {
        if let SocialMediaSchema::User(user) = item {
            if user.is_verified {
                verified_users.push(user);
            }
        }
    }
    println!("Found {} verified users in cache", verified_users.len());

    // Search for popular posts
    let mut popular_posts = Vec::new();
    for item in &interactive_cache.cache {
        if let SocialMediaSchema::Post(post) = item {
            if post.likes_count > 100 {
                popular_posts.push(post);
            }
        }
    }
    println!("Found {} popular posts in cache", popular_posts.len());

    // Demonstrate cache hit/miss patterns
    println!("\nTesting cache search patterns:");

    // This would be a cache hit
    if let Some(_user_ref) = interactive_cache.find_in_cache(|item| {
        matches!(item, SocialMediaSchema::User(user) if user.username == "alice")
    }) {
        println!("✓ Cache hit for user 'alice'");
    }

    // This would likely be a cache miss
    if interactive_cache.find_in_cache(|item| {
        matches!(item, SocialMediaSchema::User(user) if user.username == "nonexistent_user")
    }).is_none() {
        println!("✗ Cache miss for user 'nonexistent_user'");
    }

    let stats = interactive_cache.cache_stats();
    println!("\nFinal cache statistics:");
    println!("  Total searches: {}", stats.hits + stats.misses);
    println!("  Cache hits: {} ({:.1}%)", stats.hits,
             if stats.hits + stats.misses > 0 {
                 stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0
             } else {
                 0.0
             });
    println!("  Cache misses: {}", stats.misses);
    println!("  Evictions: {}", stats.evictions);
    println!("  Current size: {} / {}", stats.current_size, interactive_config.max_total_items);
    */

    println!("Advanced cache usage enables efficient reference-based queries on recent data.");
    println!("Cache statistics help optimize cache size for your access patterns.\n");

    Ok(())
}

/// Example 5: Utility functions demonstration
fn example_utility_functions() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 5: Utility Functions ===");

    // This demonstrates the utility functions:
    /*
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;

    // Count all items across all types
    println!("Counting all items in database...");
    let total_count = utils::count_all_items(&db_iter)?;
    println!("Total items in database: {}", total_count);

    // Count items for specific types
    println!("\nCounting items by type:");
    let type_names = [
        "PrimitiveTest", "TestUnit", "TestTuple", "User", "Post",
        "Comment", "Media", "Reaction", "Notification", "UserStats", "HashTag"
    ];

    for (index, type_name) in type_names.iter().enumerate() {
        let count = utils::count_items_for_type(&db_iter, index)?;
        if count > 0 {
            println!("  {}: {} items", type_name, count);
        }
    }

    // Collect a sample from each type
    println!("\nCollecting sample data (5 items per type):");
    let sample = utils::collect_sample(&db_iter, 5)?;

    let mut type_distribution = HashMap::new();
    for item in &sample {
        let type_name = match item {
            SocialMediaSchema::PrimitiveTest(_) => "PrimitiveTest",
            SocialMediaSchema::TestUnit(_) => "TestUnit",
            SocialMediaSchema::TestTuple(_) => "TestTuple",
            SocialMediaSchema::User(_) => "User",
            SocialMediaSchema::Post(_) => "Post",
            SocialMediaSchema::Comment(_) => "Comment",
            SocialMediaSchema::Media(_) => "Media",
            SocialMediaSchema::Reaction(_) => "Reaction",
            SocialMediaSchema::Notification(_) => "Notification",
            SocialMediaSchema::UserStats(_) => "UserStats",
            SocialMediaSchema::HashTag(_) => "HashTag",
        };
        *type_distribution.entry(type_name).or_insert(0) += 1;
    }

    println!("Sample distribution:");
    for (type_name, count) in type_distribution {
        println!("  {}: {} items", type_name, count);
    }

    println!("Total sample size: {} items", sample.len());
    */

    println!("Utility functions provide convenient ways to analyze and sample catalog data.");
    println!("They use the cycling iterator internally for memory-efficient operations.\n");

    Ok(())
}

/// Example 6: Memory usage patterns and best practices
fn example_memory_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 6: Memory Usage Patterns ===");

    println!("Memory-efficient patterns:");
    println!("1. Streaming processing (CyclingIterator):");
    println!("   - Processes one item at a time");
    println!("   - Memory usage: O(1) - constant");
    println!("   - Best for: Large datasets, ETL operations");

    println!("\n2. Bounded cache (CachedCyclingIterator):");
    println!("   - Maintains recent items for reference access");
    println!("   - Memory usage: O(cache_size) - bounded");
    println!("   - Best for: Interactive queries, lookups");

    println!("\n3. Type-specific processing:");
    println!("   - Process one type completely before moving to next");
    println!("   - Memory usage: O(1) per item");
    println!("   - Best for: Type-specific analytics, migrations");

    println!("\nAnti-patterns to avoid:");
    println!("  ✗ Collecting all items into Vec - O(n) memory");
    println!("  ✗ Multiple concurrent iterators - O(k*n) memory");
    println!("  ✗ Unbound caches - unbounded memory growth");

    println!("\nRecommended patterns:");
    println!("  ✓ Use CyclingIterator for streaming operations");
    println!("  ✓ Use bounded cache only when references are needed");
    println!("  ✓ Configure cache size based on available memory");
    println!("  ✓ Monitor cache statistics to optimize size");

    // Example memory-efficient processing pattern:
    /*
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;
    let mut cycling = db_iter.cycling_iter();

    // Process items in chunks to balance memory and I/O
    let chunk_size = 1000;
    let mut processed = 0;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);

        // Collect a chunk
        for _ in 0..chunk_size {
            match cycling.next() {
                Some(Ok(item)) => chunk.push(item),
                Some(Err(e)) => return Err(e.into()),
                None => break,
            }
        }

        if chunk.is_empty() {
            break;
        }

        // Process the chunk
        process_chunk(chunk);
        processed += chunk.len();

        println!("Processed {} items so far...", processed);
    }
    */

    println!("\nMemory-efficient chunked processing allows handling large datasets");
    println!("while maintaining predictable memory usage.\n");

    Ok(())
}

/// Example helper function to simulate chunk processing
#[allow(dead_code)]
fn process_chunk(chunk: Vec<SocialMediaSchema>) {
    // Simulate processing a chunk of data
    for item in chunk {
        match item {
            SocialMediaSchema::User(user) => {
                // Process user data
                let _ = user.username;
            }
            SocialMediaSchema::Post(post) => {
                // Process post data
                let _ = post.content;
            }
            _ => {
                // Process other types
            }
        }
    }
}

/// Example 7: Error handling patterns
fn example_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 7: Error Handling Patterns ===");

    println!("Robust error handling with cycling iterators:");

    // This demonstrates proper error handling:
    /*
    let db_iter = SocialMediaSchemaDBIter::new(&db)?;
    let mut cycling = db_iter.cycling_iter();

    let mut successful_items = 0;
    let mut error_count = 0;

    while let Some(result) = cycling.next() {
        match result {
            Ok(item) => {
                successful_items += 1;

                // Process item safely
                match process_catalog_item(item) {
                    Ok(()) => {
                        // Item processed successfully
                    }
                    Err(processing_error) => {
                        eprintln!("Processing error: {}", processing_error);
                        // Continue with next item rather than failing completely
                    }
                }
            }
            Err(db_error) => {
                error_count += 1;
                eprintln!("Database error: {:?}", db_error);

                // Decide whether to continue or abort based on error type
                match db_error {
                    native_db::db_type::Error::Io(_) => {
                        // I/O errors might be transient, continue
                        continue;
                    }
                    _ => {
                        // Other errors might be fatal, abort
                        return Err(db_error.into());
                    }
                }
            }
        }
    }

    println!("Processing complete:");
    println!("  Successful items: {}", successful_items);
    println!("  Errors encountered: {}", error_count);
    */

    println!("Proper error handling ensures robustness when processing large datasets.");
    println!("The iterator continues gracefully even when individual items have issues.\n");

    Ok(())
}

/// Mock function to simulate item processing
#[allow(dead_code)]
fn process_catalog_item(_item: SocialMediaSchema) -> Result<(), String> {
    // Simulate processing that might fail
    Ok(())
}

/// Main demonstration function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Netabase Cycling Iterator Demonstration\n");
    println!("This example shows how to efficiently iterate through catalog data");
    println!("using memory-efficient streaming and optional caching.\n");

    // Run all examples
    example_basic_cycling()?;
    example_cached_cycling()?;
    example_type_specific()?;
    example_advanced_cache_usage()?;
    example_utility_functions()?;
    example_memory_patterns()?;
    example_error_handling()?;

    println!("=== Summary ===");
    println!("The cycling iterator system provides:");
    println!("  ✓ Memory-efficient streaming through large datasets");
    println!("  ✓ Optional bounded caching for reference access");
    println!("  ✓ Type-specific iteration capabilities");
    println!("  ✓ Robust error handling");
    println!("  ✓ Configurable cache management");
    println!("  ✓ Utility functions for common operations");

    println!("\nKey benefits:");
    println!("  • Constant memory usage for streaming operations");
    println!("  • Bounded memory usage when caching is needed");
    println!("  • Leverages existing PrimaryScanIterator efficiency");
    println!("  • Provides both owned and reference access patterns");

    println!("\nRecommended usage:");
    println!("  • Use CyclingIterator for ETL, analytics, and bulk operations");
    println!("  • Use CachedCyclingIterator for interactive applications");
    println!("  • Configure cache size based on available memory and access patterns");
    println!("  • Monitor cache statistics to optimize performance");

    Ok(())
}

#[cfg(test)]
mod demo_tests {
    use super::*;

    #[test]
    fn test_cache_config_creation() {
        let default_config = CacheConfig::default();
        assert_eq!(default_config.max_items_per_type, 100);
        assert_eq!(default_config.max_total_items, 500);

        let custom_config = CacheConfig {
            max_items_per_type: 25,
            max_total_items: 100,
        };
        assert_eq!(custom_config.max_items_per_type, 25);
        assert_eq!(custom_config.max_total_items, 100);
    }

    #[test]
    fn test_sample_data_generation() {
        let users = create_sample_users();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "alice");
        assert_eq!(users[1].username, "bob");

        let posts = create_sample_posts();
        assert_eq!(posts.len(), 2);
        assert!(posts[0].content.contains("Rust"));
        assert!(posts[1].content.contains("product"));

        let comments = create_sample_comments();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].post_id, 1);
        assert_eq!(comments[1].post_id, 2);
    }

    #[test]
    fn test_all_examples_run_without_panic() {
        // These should run without panicking even without a real database
        assert!(example_basic_cycling().is_ok());
        assert!(example_cached_cycling().is_ok());
        assert!(example_type_specific().is_ok());
        assert!(example_advanced_cache_usage().is_ok());
        assert!(example_utility_functions().is_ok());
        assert!(example_memory_patterns().is_ok());
        assert!(example_error_handling().is_ok());
    }
}
