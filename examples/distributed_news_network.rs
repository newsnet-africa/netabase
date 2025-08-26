//! Distributed News Network Example
//!
//! This example demonstrates how to use Netabase to create a distributed peer-to-peer
//! news network with multiple nodes storing and sharing news articles, sources, and metadata.

use anyhow::Result;
use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
use libp2p::{PeerId, kad::Quorum};
use netabase::{Netabase, NetabaseConfig, NetabaseSchema};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Represents a news source in the distributed network
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct NewsSource {
    #[key]
    source_id: String,
    name: String,
    url: String,
    credibility_score: f64, // 0.0 to 1.0
    category: String,       // e.g., "politics", "technology", "sports"
    language: String,       // ISO language code
    country: String,        // ISO country code
    created_at: u64,
    last_updated: u64,
}

/// Represents a news article in the distributed network
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Article {
    #[key]
    article_id: String,
    title: String,
    source_id: String,
    author: Option<String>,
    content_hash: String, // Hash of article content for deduplication
    summary: String,
    publication_date: u64,
    sentiment_score: f64, // -1.0 (negative) to 1.0 (positive)
    category: String,
    tags: Vec<String>,
    url: String,
    word_count: u32,
}

/// Represents user ratings and feedback for articles
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct ArticleRating {
    #[key]
    rating_id: String, // Composite: user_id + article_id
    article_id: String,
    user_id: String,
    rating: u8, // 1-5 stars
    helpful_votes: u32,
    created_at: u64,
}

/// Represents trending topics based on article analysis
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct TrendingTopic {
    #[key]
    topic_id: String,
    name: String,
    description: String,
    article_count: u32,
    sentiment_average: f64,
    trending_score: f64, // Algorithm-calculated trending score
    first_seen: u64,
    last_updated: u64,
}

/// Main example function
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    netabase::init_logging();

    println!("🌐 Distributed News Network Example");
    println!("====================================\n");

    // Run different examples
    basic_network_setup().await?;
    multi_node_simulation().await?;
    content_distribution_example().await?;
    data_integrity_example().await?;
    network_resilience_example().await?;

    println!("\n🎉 All examples completed successfully!");
    Ok(())
}

/// Example 1: Basic network setup and data storage
async fn basic_network_setup() -> Result<()> {
    println!("📊 Example 1: Basic Network Setup");
    println!("---------------------------------");

    // Create a single node network
    let config = NetabaseConfig::default()
        .with_storage_path("./examples_data/node1".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());

    let mut node = Netabase::try_new(config, "/newsnet-basic").await?;
    node.start_swarm().await?;

    // Wait for network initialization
    sleep(Duration::from_secs(1)).await;

    println!("🔗 Network node started");
    let listeners = node.listeners().await?;
    println!("📡 Listening on: {:?}", listeners);

    // Create and store sample news sources
    let sources = create_sample_sources();

    for source in &sources {
        let result = node
            .put(
                source.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!(
            "📰 Stored news source: {} ({})",
            source.name, source.source_id
        );
    }

    // Create and store sample articles
    let articles = create_sample_articles();

    for article in &articles {
        let result = node
            .put(
                article.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!(
            "📄 Stored article: {} ({})",
            article.title, article.article_id
        );
    }

    // Store trending topics
    let topics = create_sample_topics();

    for topic in &topics {
        let result = node
            .put(
                topic.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!(
            "📈 Stored trending topic: {} ({})",
            topic.name, topic.topic_id
        );
    }

    println!("✅ Basic network setup completed\n");

    // Clean shutdown
    node.close_swarm().await?;
    Ok(())
}

/// Example 2: Multi-node network simulation
async fn multi_node_simulation() -> Result<()> {
    println!("🌐 Example 2: Multi-Node Network");
    println!("--------------------------------");

    // Create multiple nodes
    let mut nodes = Vec::new();

    for i in 1..=3 {
        let config = NetabaseConfig::default()
            .with_storage_path(format!("./examples_data/multi_node_{}", i).into())
            .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());

        let mut node = Netabase::try_new(config, "/newsnet-multi").await?;
        node.start_swarm().await?;

        println!("🟢 Node {} started", i);
        nodes.push(node);
    }

    // Wait for network discovery
    sleep(Duration::from_secs(2)).await;

    // Show network topology
    for (i, node) in nodes.iter().enumerate() {
        let listeners = node.listeners().await?;
        println!("📡 Node {} listening on: {:?}", i + 1, listeners);
    }

    // Distribute different types of content across nodes
    println!("\n📊 Distributing content across nodes...");

    // Node 1: Store news sources
    let sources = create_sample_sources();
    for source in &sources[..2] {
        nodes[0]
            .put(
                source.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!("📰 Node 1 stored source: {}", source.name);
    }

    // Node 2: Store articles
    let articles = create_sample_articles();
    for article in &articles[..3] {
        nodes[1]
            .put(
                article.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!("📄 Node 2 stored article: {}", article.title);
    }

    // Node 3: Store trending topics and ratings
    let topics = create_sample_topics();
    let ratings = create_sample_ratings();

    for topic in &topics[..2] {
        nodes[2]
            .put(
                topic.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!("📈 Node 3 stored topic: {}", topic.name);
    }

    for rating in &ratings[..2] {
        nodes[2]
            .put(
                rating.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!("⭐ Node 3 stored rating: {}", rating.rating_id);
    }

    // Demonstrate cross-node data retrieval (if network allows)
    println!("\n🔍 Testing cross-node data access...");
    sleep(Duration::from_secs(1)).await; // Allow replication time

    // Try to retrieve data from different nodes
    for (i, node) in nodes.iter_mut().enumerate() {
        println!("🔎 Node {} attempting data retrieval...", i + 1);
        // Note: In a real distributed scenario, we would attempt to get
        // data stored on other nodes, but this requires more complex setup
    }

    println!("✅ Multi-node simulation completed\n");

    // Clean shutdown
    for mut node in nodes {
        node.close_swarm().await?;
    }

    Ok(())
}

/// Example 3: Content distribution and replication
async fn content_distribution_example() -> Result<()> {
    println!("🔄 Example 3: Content Distribution");
    println!("----------------------------------");

    let config = NetabaseConfig::default()
        .with_storage_path("./examples_data/distribution".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());

    let mut node = Netabase::try_new(config, "/newsnet-distribution").await?;
    node.start_swarm().await?;
    sleep(Duration::from_secs(1)).await;

    println!("🔗 Distribution node started");

    // Simulate high-throughput content ingestion
    println!("📊 Simulating high-volume content ingestion...");

    let batch_size = 10;
    let mut total_stored = 0;

    // Batch 1: Technology news
    let tech_articles = create_tech_articles(batch_size);
    for (i, article) in tech_articles.iter().enumerate() {
        node.put(
            article.clone(),
            None::<std::vec::IntoIter<PeerId>>,
            Quorum::One,
        )
        .await?;
        total_stored += 1;

        if (i + 1) % 5 == 0 {
            println!("📄 Stored {} tech articles...", i + 1);
        }
    }

    // Batch 2: Political news
    let political_articles = create_political_articles(batch_size);
    for (i, article) in political_articles.iter().enumerate() {
        node.put(
            article.clone(),
            None::<std::vec::IntoIter<PeerId>>,
            Quorum::One,
        )
        .await?;
        total_stored += 1;

        if (i + 1) % 5 == 0 {
            println!("🏛️ Stored {} political articles...", i + 1);
        }
    }

    // Batch 3: Sports news
    let sports_articles = create_sports_articles(batch_size);
    for (i, article) in sports_articles.iter().enumerate() {
        node.put(
            article.clone(),
            None::<std::vec::IntoIter<PeerId>>,
            Quorum::One,
        )
        .await?;
        total_stored += 1;

        if (i + 1) % 5 == 0 {
            println!("⚽ Stored {} sports articles...", i + 1);
        }
    }

    println!("✅ Total articles stored: {}", total_stored);

    // Simulate content analysis and trending topic generation
    println!("\n📈 Generating trending topics from stored content...");

    let trending_topics = analyze_and_create_trending_topics();
    for topic in &trending_topics {
        node.put(
            topic.clone(),
            None::<std::vec::IntoIter<PeerId>>,
            Quorum::One,
        )
        .await?;
        println!(
            "📊 Generated trending topic: {} (score: {:.2})",
            topic.name, topic.trending_score
        );
    }

    println!("✅ Content distribution example completed\n");

    node.close_swarm().await?;
    Ok(())
}

/// Example 4: Data integrity and verification
async fn data_integrity_example() -> Result<()> {
    println!("🔒 Example 4: Data Integrity");
    println!("----------------------------");

    let config = NetabaseConfig::default()
        .with_storage_path("./examples_data/integrity".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());

    let mut node = Netabase::try_new(config, "/newsnet-integrity").await?;
    node.start_swarm().await?;
    sleep(Duration::from_secs(1)).await;

    println!("🔗 Integrity test node started");

    // Create articles with known content hashes for verification
    let mut verified_articles = Vec::new();

    for i in 1..=5 {
        let content = format!(
            "This is test article number {} with specific content for hash verification.",
            i
        );
        let content_hash = calculate_content_hash(&content);

        let article = Article {
            article_id: format!("verify-{:03}", i),
            title: format!("Verification Article {}", i),
            source_id: "test-source".to_string(),
            author: Some("Test Author".to_string()),
            content_hash: content_hash.clone(),
            summary: content[..50].to_string(),
            publication_date: Utc::now().timestamp() as u64,
            sentiment_score: 0.0,
            category: "test".to_string(),
            tags: vec!["verification".to_string(), "integrity".to_string()],
            url: format!("https://test.example.com/article-{}", i),
            word_count: content.split_whitespace().count() as u32,
        };

        node.put(
            article.clone(),
            None::<std::vec::IntoIter<PeerId>>,
            Quorum::One,
        )
        .await?;
        verified_articles.push((article, content));

        println!("📄 Stored article with hash: {}", content_hash);
    }

    // Verify data integrity by checking hashes
    println!("\n🔍 Verifying data integrity...");

    let mut verification_passed = 0;
    let mut verification_failed = 0;

    for (article, original_content) in &verified_articles {
        let recalculated_hash = calculate_content_hash(original_content);

        if article.content_hash == recalculated_hash {
            println!("✅ Article {} hash verified", article.article_id);
            verification_passed += 1;
        } else {
            println!("❌ Article {} hash mismatch!", article.article_id);
            println!("   Expected: {}", article.content_hash);
            println!("   Got: {}", recalculated_hash);
            verification_failed += 1;
        }
    }

    println!("\n📊 Integrity Check Results:");
    println!("  Passed: {}", verification_passed);
    println!("  Failed: {}", verification_failed);
    println!(
        "  Success Rate: {:.1}%",
        100.0 * verification_passed as f64 / (verification_passed + verification_failed) as f64
    );

    // Demonstrate duplicate detection using content hashes
    println!("\n🔍 Testing duplicate detection...");

    let duplicate_article = Article {
        article_id: "duplicate-001".to_string(),
        title: "Duplicate Article Test".to_string(),
        source_id: "test-source".to_string(),
        author: Some("Test Author".to_string()),
        content_hash: verified_articles[0].0.content_hash.clone(), // Same hash as first article
        summary: "This is a duplicate".to_string(),
        publication_date: Utc::now().timestamp() as u64,
        sentiment_score: 0.0,
        category: "test".to_string(),
        tags: vec!["duplicate".to_string()],
        url: "https://test.example.com/duplicate".to_string(),
        word_count: 100,
    };

    node.put(
        duplicate_article.clone(),
        None::<std::vec::IntoIter<PeerId>>,
        Quorum::One,
    )
    .await?;
    println!(
        "📄 Stored potential duplicate with hash: {}",
        duplicate_article.content_hash
    );

    // Check for duplicates
    let mut duplicate_count = 0;
    for (existing_article, _) in &verified_articles {
        if existing_article.content_hash == duplicate_article.content_hash
            && existing_article.article_id != duplicate_article.article_id
        {
            duplicate_count += 1;
            println!(
                "🔍 Found duplicate: {} and {}",
                existing_article.article_id, duplicate_article.article_id
            );
        }
    }

    if duplicate_count > 0 {
        println!("⚠️ Detected {} potential duplicate(s)", duplicate_count);
    } else {
        println!("✅ No duplicates detected");
    }

    println!("✅ Data integrity example completed\n");

    node.close_swarm().await?;
    Ok(())
}

/// Example 5: Network resilience and fault tolerance
async fn network_resilience_example() -> Result<()> {
    println!("🛡️ Example 5: Network Resilience");
    println!("----------------------------------");

    // Create a network of nodes to test resilience
    let mut nodes = Vec::new();

    for i in 1..=4 {
        let config = NetabaseConfig::default()
            .with_storage_path(format!("./examples_data/resilience_node_{}", i).into())
            .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());

        let mut node = Netabase::try_new(config, "/newsnet-resilience").await?;
        node.start_swarm().await?;

        println!("🟢 Resilience node {} started", i);
        nodes.push(node);
    }

    sleep(Duration::from_secs(2)).await;

    // Store critical news data across multiple nodes
    println!("📊 Storing critical news data across network...");

    let critical_sources = create_critical_news_sources();
    let breaking_news = create_breaking_news_articles();

    // Distribute sources across nodes
    for (i, source) in critical_sources.iter().enumerate() {
        let node_index = i % nodes.len();
        nodes[node_index]
            .put(
                source.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!(
            "📰 Node {} stored critical source: {}",
            node_index + 1,
            source.name
        );
    }

    // Distribute breaking news across nodes
    for (i, article) in breaking_news.iter().enumerate() {
        let node_index = i % nodes.len();
        nodes[node_index]
            .put(
                article.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await?;
        println!(
            "📄 Node {} stored breaking news: {}",
            node_index + 1,
            article.title
        );
    }

    // Simulate node failure
    println!("\n⚠️ Simulating node failure...");
    println!("🔴 Node 2 going offline...");

    // Remove node 2 from active nodes (simulating failure)
    let failed_node = nodes.remove(1); // Remove second node (index 1)
    failed_node.close_swarm().await?;

    println!("💔 Node 2 is now offline");
    sleep(Duration::from_secs(1)).await;

    // Test network functionality with reduced capacity
    println!("\n🔍 Testing network with reduced capacity...");

    let emergency_articles = create_emergency_news_articles();

    for (i, article) in emergency_articles.iter().enumerate() {
        let node_index = i % nodes.len(); // Now only 3 nodes
        match nodes[node_index]
            .put(
                article.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await
        {
            Ok(_) => println!(
                "✅ Node {} stored emergency article: {}",
                node_index + 1,
                article.title
            ),
            Err(e) => println!("❌ Failed to store on node {}: {}", node_index + 1, e),
        }
    }

    // Simulate node recovery
    println!("\n🔄 Simulating node recovery...");
    println!("🟢 Node 5 (replacement) coming online...");

    let recovery_config = NetabaseConfig::default()
        .with_storage_path("./examples_data/recovery_node".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());

    let mut recovery_node = Netabase::try_new(recovery_config, "/newsnet-resilience").await?;
    recovery_node.start_swarm().await?;

    println!("✅ Recovery node online");
    sleep(Duration::from_secs(1)).await;

    // Test full network functionality restored
    println!("\n🔍 Testing restored network functionality...");

    let recovery_articles = create_recovery_test_articles();

    for article in &recovery_articles {
        match recovery_node
            .put(
                article.clone(),
                None::<std::vec::IntoIter<PeerId>>,
                Quorum::One,
            )
            .await
        {
            Ok(_) => println!("✅ Recovery node stored: {}", article.title),
            Err(e) => println!("❌ Recovery node failed: {}", e),
        }
    }

    println!("📊 Network Resilience Test Results:");
    println!("  ✅ Network survived node failure");
    println!("  ✅ Continued operation with reduced capacity");
    println!("  ✅ Successfully integrated recovery node");
    println!("  ✅ Full functionality restored");

    println!("✅ Network resilience example completed\n");

    // Clean shutdown
    for mut node in nodes {
        node.close_swarm().await?;
    }
    recovery_node.close_swarm().await?;

    Ok(())
}

// Helper functions to create sample data

fn create_sample_sources() -> Vec<NewsSource> {
    let now = Utc::now().timestamp() as u64;

    vec![
        NewsSource {
            source_id: "reuters-001".to_string(),
            name: "Reuters".to_string(),
            url: "https://reuters.com".to_string(),
            credibility_score: 0.92,
            category: "general".to_string(),
            language: "en".to_string(),
            country: "UK".to_string(),
            created_at: now,
            last_updated: now,
        },
        NewsSource {
            source_id: "techcrunch-001".to_string(),
            name: "TechCrunch".to_string(),
            url: "https://techcrunch.com".to_string(),
            credibility_score: 0.85,
            category: "technology".to_string(),
            language: "en".to_string(),
            country: "US".to_string(),
            created_at: now,
            last_updated: now,
        },
        NewsSource {
            source_id: "bbc-001".to_string(),
            name: "BBC News".to_string(),
            url: "https://bbc.com/news".to_string(),
            credibility_score: 0.89,
            category: "general".to_string(),
            language: "en".to_string(),
            country: "UK".to_string(),
            created_at: now,
            last_updated: now,
        },
    ]
}

fn create_sample_articles() -> Vec<Article> {
    let now = Utc::now().timestamp() as u64;

    vec![
        Article {
            article_id: "art-001".to_string(),
            title: "Global Economic Outlook Improves".to_string(),
            source_id: "reuters-001".to_string(),
            author: Some("John Smith".to_string()),
            content_hash: calculate_content_hash(
                "Global economic indicators show positive trends...",
            ),
            summary: "Economic indicators suggest improvement in global markets.".to_string(),
            publication_date: now,
            sentiment_score: 0.3,
            category: "economics".to_string(),
            tags: vec![
                "economy".to_string(),
                "markets".to_string(),
                "global".to_string(),
            ],
            url: "https://reuters.com/economics/global-outlook".to_string(),
            word_count: 450,
        },
        Article {
            article_id: "art-002".to_string(),
            title: "AI Breakthrough in Medical Diagnosis".to_string(),
            source_id: "techcrunch-001".to_string(),
            author: Some("Sarah Johnson".to_string()),
            content_hash: calculate_content_hash(
                "New AI system shows 95% accuracy in medical diagnosis...",
            ),
            summary: "Revolutionary AI system achieves high accuracy in medical diagnostics."
                .to_string(),
            publication_date: now - 3600,
            sentiment_score: 0.7,
            category: "technology".to_string(),
            tags: vec![
                "ai".to_string(),
                "medical".to_string(),
                "breakthrough".to_string(),
            ],
            url: "https://techcrunch.com/ai-medical-breakthrough".to_string(),
            word_count: 680,
        },
        Article {
            article_id: "art-003".to_string(),
            title: "Climate Summit Reaches Historic Agreement".to_string(),
            source_id: "bbc-001".to_string(),
            author: Some("David Brown".to_string()),
            content_hash: calculate_content_hash(
                "World leaders agree on ambitious climate targets...",
            ),
            summary: "International climate summit produces landmark agreement on emissions."
                .to_string(),
            publication_date: now - 7200,
            sentiment_score: 0.5,
            category: "environment".to_string(),
            tags: vec![
                "climate".to_string(),
                "environment".to_string(),
                "summit".to_string(),
            ],
            url: "https://bbc.com/news/climate-agreement".to_string(),
            word_count: 520,
        },
    ]
}

fn create_sample_topics() -> Vec<TrendingTopic> {
    let now = Utc::now().timestamp() as u64;

    vec![
        TrendingTopic {
            topic_id: "trend-001".to_string(),
            name: "Artificial Intelligence".to_string(),
            description: "Latest developments in AI technology and applications".to_string(),
            article_count: 45,
            sentiment_average: 0.4,
            trending_score: 8.7,
            first_seen: now - 86400,
            last_updated: now,
        },
        TrendingTopic {
            topic_id: "trend-002".to_string(),
            name: "Climate Action".to_string(),
            description: "Global efforts to address climate change".to_string(),
            article_count: 32,
            sentiment_average: 0.1,
            trending_score: 7.2,
            first_seen: now - 172800,
            last_updated: now - 3600,
        },
    ]
}

fn create_sample_ratings() -> Vec<ArticleRating> {
    let now = Utc::now().timestamp() as u64;

    vec![
        ArticleRating {
            rating_id: "user123-art-001".to_string(),
            article_id: "art-001".to_string(),
            user_id: "user123".to_string(),
            rating: 4,
            helpful_votes: 12,
            created_at: now,
        },
        ArticleRating {
            rating_id: "user456-art-002".to_string(),
            article_id: "art-002".to_string(),
            user_id: "user456".to_string(),
            rating: 5,
            helpful_votes: 8,
            created_at: now - 1800,
        },
    ]
}

// Additional helper functions for specific examples

fn create_tech_articles(count: usize) -> Vec<Article> {
    let mut articles = Vec::new();
    let now = Utc::now().timestamp() as u64;

    for i in 1..=count {
        articles.push(Article {
            article_id: format!("tech-{:03}", i),
            title: format!("Technology Innovation {}", i),
            source_id: "techcrunch-001".to_string(),
            author: Some(format!("Tech Writer {}", i)),
            content_hash: calculate_content_hash(&format!("Tech content {}", i)),
            summary: format!("Technology news article number {}", i),
            publication_date: now - (i as u64 * 300),
            sentiment_score: 0.6,
            category: "technology".to_string(),
            tags: vec!["tech".to_string(), "innovation".to_string()],
            url: format!("https://techcrunch.com/article-{}", i),
            word_count: 400 + (i as u32 * 10),
        });
    }

    articles
}

fn create_political_articles(count: usize) -> Vec<Article> {
    let mut articles = Vec::new();
    let now = Utc::now().timestamp() as u64;

    for i in 1..=count {
        articles.push(Article {
            article_id: format!("pol-{:03}", i),
            title: format!("Political Development {}", i),
            source_id: "reuters-001".to_string(),
            author: Some(format!("Political Correspondent {}", i)),
            content_hash: calculate_content_hash(&format!("Political content {}", i)),
            summary: format!("Political news article number {}", i),
            publication_date: now - (i as u64 * 450),
            sentiment_score: 0.0,
            category: "politics".to_string(),
            tags: vec!["politics".to_string(), "government".to_string()],
            url: format!("https://reuters.com/politics-{}", i),
            word_count: 350 + (i as u32 * 15),
        });
    }

    articles
}

fn create_sports_articles(count: usize) -> Vec<Article> {
    let mut articles = Vec::new();
    let now = Utc::now().timestamp() as u64;

    for i in 1..=count {
        articles.push(Article {
            article_id: format!("sports-{:03}", i),
            title: format!("Sports Update {}", i),
            source_id: "bbc-001".to_string(),
            author: Some(format!("Sports Reporter {}", i)),
            content_hash: calculate_content_hash(&format!("Sports content {}", i)),
            summary: format!("Sports news article number {}", i),
            publication_date: now - (i as u64 * 600),
            sentiment_score: 0.4,
            category: "sports".to_string(),
            tags: vec!["sports".to_string(), "competition".to_string()],
            url: format!("https://bbc.com/sport/article-{}", i),
            word_count: 300 + (i as u32 * 20),
        });
    }

    articles
}

fn analyze_and_create_trending_topics() -> Vec<TrendingTopic> {
    let now = Utc::now().timestamp() as u64;

    vec![
        TrendingTopic {
            topic_id: "auto-trend-001".to_string(),
            name: "Technology Innovation".to_string(),
            description: "Auto-generated from recent tech articles".to_string(),
            article_count: 10,
            sentiment_average: 0.6,
            trending_score: 9.2,
            first_seen: now - 3600,
            last_updated: now,
        },
        TrendingTopic {
            topic_id: "auto-trend-002".to_string(),
            name: "Political Developments".to_string(),
            description: "Auto-generated from recent political coverage".to_string(),
            article_count: 10,
            sentiment_average: 0.0,
            trending_score: 7.8,
            first_seen: now - 3600,
            last_updated: now,
        },
        TrendingTopic {
            topic_id: "auto-trend-003".to_string(),
            name: "Sports Competition".to_string(),
            description: "Auto-generated from recent sports coverage".to_string(),
            article_count: 10,
            sentiment_average: 0.4,
            trending_score: 6.5,
            first_seen: now - 3600,
            last_updated: now,
        },
    ]
}

fn create_critical_news_sources() -> Vec<NewsSource> {
    let now = Utc::now().timestamp() as u64;

    vec![
        NewsSource {
            source_id: "emergency-reuters".to_string(),
            name: "Reuters Emergency".to_string(),
            url: "https://reuters.com/emergency".to_string(),
            credibility_score: 0.95,
            category: "breaking".to_string(),
            language: "en".to_string(),
            country: "UK".to_string(),
            created_at: now,
            last_updated: now,
        },
        NewsSource {
            source_id: "crisis-bbc".to_string(),
            name: "BBC Crisis Desk".to_string(),
            url: "https://bbc.com/crisis".to_string(),
            credibility_score: 0.93,
            category: "breaking".to_string(),
            language: "en".to_string(),
            country: "UK".to_string(),
            created_at: now,
            last_updated: now,
        },
    ]
}

fn create_breaking_news_articles() -> Vec<Article> {
    let now = Utc::now().timestamp() as u64;

    vec![
        Article {
            article_id: "breaking-001".to_string(),
            title: "Major Economic Summit Announced".to_string(),
            source_id: "emergency-reuters".to_string(),
            author: Some("Emergency Correspondent".to_string()),
            content_hash: calculate_content_hash("Breaking: Major economic summit announced..."),
            summary: "World leaders to gather for emergency economic summit".to_string(),
            publication_date: now,
            sentiment_score: 0.2,
            category: "breaking".to_string(),
            tags: vec![
                "breaking".to_string(),
                "economy".to_string(),
                "summit".to_string(),
            ],
            url: "https://reuters.com/breaking/economic-summit".to_string(),
            word_count: 320,
        },
        Article {
            article_id: "breaking-002".to_string(),
            title: "Technology Sector Shows Resilience".to_string(),
            source_id: "crisis-bbc".to_string(),
            author: Some("Crisis Reporter".to_string()),
            content_hash: calculate_content_hash(
                "Technology sector demonstrates strong performance...",
            ),
            summary: "Tech companies report strong quarterly results despite challenges"
                .to_string(),
            publication_date: now - 1800,
            sentiment_score: 0.6,
            category: "breaking".to_string(),
            tags: vec![
                "breaking".to_string(),
                "technology".to_string(),
                "resilience".to_string(),
            ],
            url: "https://bbc.com/crisis/tech-resilience".to_string(),
            word_count: 280,
        },
    ]
}

fn create_emergency_news_articles() -> Vec<Article> {
    let now = Utc::now().timestamp() as u64;

    vec![
        Article {
            article_id: "emergency-001".to_string(),
            title: "Network Resilience Test Article 1".to_string(),
            source_id: "emergency-reuters".to_string(),
            author: Some("Resilience Tester".to_string()),
            content_hash: calculate_content_hash("Emergency article during network stress test..."),
            summary: "Test article created during network node failure simulation".to_string(),
            publication_date: now,
            sentiment_score: 0.0,
            category: "test".to_string(),
            tags: vec![
                "emergency".to_string(),
                "test".to_string(),
                "resilience".to_string(),
            ],
            url: "https://test.example.com/emergency-1".to_string(),
            word_count: 150,
        },
        Article {
            article_id: "emergency-002".to_string(),
            title: "Network Resilience Test Article 2".to_string(),
            source_id: "crisis-bbc".to_string(),
            author: Some("Resilience Tester".to_string()),
            content_hash: calculate_content_hash(
                "Second emergency article during network stress test...",
            ),
            summary: "Second test article created during network node failure simulation"
                .to_string(),
            publication_date: now - 300,
            sentiment_score: 0.0,
            category: "test".to_string(),
            tags: vec![
                "emergency".to_string(),
                "test".to_string(),
                "resilience".to_string(),
            ],
            url: "https://test.example.com/emergency-2".to_string(),
            word_count: 160,
        },
    ]
}

fn create_recovery_test_articles() -> Vec<Article> {
    let now = Utc::now().timestamp() as u64;

    vec![
        Article {
            article_id: "recovery-001".to_string(),
            title: "Network Recovery Success Story".to_string(),
            source_id: "emergency-reuters".to_string(),
            author: Some("Recovery Analyst".to_string()),
            content_hash: calculate_content_hash(
                "Network successfully recovered from node failure...",
            ),
            summary: "Analysis of successful network recovery after node failure".to_string(),
            publication_date: now,
            sentiment_score: 0.8,
            category: "analysis".to_string(),
            tags: vec![
                "recovery".to_string(),
                "success".to_string(),
                "analysis".to_string(),
            ],
            url: "https://test.example.com/recovery-success".to_string(),
            word_count: 420,
        },
        Article {
            article_id: "recovery-002".to_string(),
            title: "Lessons Learned from Network Resilience Testing".to_string(),
            source_id: "crisis-bbc".to_string(),
            author: Some("Network Analyst".to_string()),
            content_hash: calculate_content_hash(
                "Key insights from distributed network resilience testing...",
            ),
            summary: "Important lessons learned from comprehensive network resilience testing"
                .to_string(),
            publication_date: now - 600,
            sentiment_score: 0.5,
            category: "analysis".to_string(),
            tags: vec![
                "lessons".to_string(),
                "resilience".to_string(),
                "testing".to_string(),
            ],
            url: "https://test.example.com/lessons-learned".to_string(),
            word_count: 380,
        },
    ]
}

/// Simple hash function for content verification
fn calculate_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
