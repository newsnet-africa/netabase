#![cfg(feature = "native")]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::sled_store::SledStore;

// Test schema
#[netabase_definition_module(BenchDefinition, BenchKeys)]
mod bench_schema {
    use netabase_deps::{bincode, serde};
    use netabase_macros::NetabaseModel;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    pub struct Article {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

use bench_schema::*;

fn bench_raw_sled_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    for size in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("raw_sled", size), size, |b, &size| {
            b.iter(|| {
                let config = sled::Config::new().temporary(true);
                let db = config.open().unwrap();
                let tree = db.open_tree("articles").unwrap();

                for i in 0u64..size {
                    let key = i.to_be_bytes();
                    let value = bincode::encode_to_vec(
                        &Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content {}", i),
                            author_id: i % 10,
                        },
                        bincode::config::standard(),
                    )
                    .unwrap();
                    tree.insert(key, value).unwrap();
                }
                black_box(tree.len());
            });
        });

        group.bench_with_input(BenchmarkId::new("wrapper", size), size, |b, &size| {
            b.iter(|| {
                let store = SledStore::<BenchDefinition>::temp().unwrap();
                let article_tree = store.open_tree::<Article>();

                for i in 0u64..size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content {}", i),
                        author_id: i % 10,
                    };
                    article_tree.put(article).unwrap();
                }
                black_box(article_tree.len());
            });
        });
    }

    group.finish();
}

fn bench_raw_sled_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    for size in [100, 1000, 5000].iter() {
        // Setup raw sled
        let config = sled::Config::new().temporary(true);
        let db = config.open().unwrap();
        let tree = db.open_tree("articles").unwrap();

        for i in 0u64..*size {
            let key = i.to_be_bytes();
            let value = bincode::encode_to_vec(
                &Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content {}", i),
                    author_id: i % 10,
                },
                bincode::config::standard(),
            )
            .unwrap();
            tree.insert(key, value).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("raw_sled", size), size, |b, &size| {
            b.iter(|| {
                for i in 0u64..size {
                    let key = i.to_be_bytes();
                    let value = tree.get(key).unwrap();
                    black_box(value);
                }
            });
        });

        // Setup wrapper
        let store = SledStore::<BenchDefinition>::temp().unwrap();
        let article_tree = store.open_tree::<Article>();

        for i in 0..*size {
            let article = Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                author_id: i % 10,
            };
            article_tree.put(article).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("wrapper", size), size, |b, &size| {
            b.iter(|| {
                for i in 0u64..size {
                    let article = article_tree.get(ArticlePrimaryKey(i)).unwrap();
                    black_box(article);
                }
            });
        });
    }

    group.finish();
}

fn bench_raw_sled_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");

    for size in [100, 1000, 5000].iter() {
        // Setup raw sled
        let config = sled::Config::new().temporary(true);
        let db = config.open().unwrap();
        let tree = db.open_tree("articles").unwrap();

        for i in 0u64..*size {
            let key = i.to_be_bytes();
            let value = bincode::encode_to_vec(
                &Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content {}", i),
                    author_id: i % 10,
                },
                bincode::config::standard(),
            )
            .unwrap();
            tree.insert(key, value).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("raw_sled", size), size, |b, _size| {
            b.iter(|| {
                let count = tree.iter().count();
                black_box(count);
            });
        });

        // Setup wrapper
        let store = SledStore::<BenchDefinition>::temp().unwrap();
        let article_tree = store.open_tree::<Article>();

        for i in 0..*size {
            let article = Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                author_id: i % 10,
            };
            article_tree.put(article).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("wrapper", size), size, |b, _size| {
            b.iter(|| {
                let count = article_tree.iter().count();
                black_box(count);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_raw_sled_insert, bench_raw_sled_get, bench_raw_sled_iteration);
criterion_main!(benches);
