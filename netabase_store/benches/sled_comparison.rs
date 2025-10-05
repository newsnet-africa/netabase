use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::NetabaseDatabase,
    traits::{NetabaseModel as NetabaseModelTrait, NetabaseSchema, NetabaseSchemaQuery},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use tempfile::TempDir;

// Test schema for benchmarks
#[netabase_schema_module(BenchSchema, BenchSchemaKeys)]
mod bench_schema {
    use super::*;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        netabase_store::bincode::Encode,
        netabase_store::bincode::Decode,
        Serialize,
        Deserialize,
    )]
    #[key_name(BenchUserKey)]
    pub struct BenchUser {
        #[key]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub age: u32,
        pub data: Vec<u8>,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        netabase_store::bincode::Encode,
        netabase_store::bincode::Decode,
        Serialize,
        Deserialize,
    )]
    #[key_name(BenchPostKey)]
    pub struct BenchPost {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author_id: u64,
        pub timestamp: u64,
    }
}

use bench_schema::*;

fn create_bench_user(id: u64) -> BenchUser {
    let mut rng = rand::thread_rng();
    BenchUser {
        id,
        name: format!("User{}", id),
        email: format!("user{}@example.com", id),
        age: rng.gen_range(18..80),
        data: (0..100).map(|_| rng.r#gen::<u8>()).collect(),
    }
}

fn create_bench_post(id: u64, author_id: u64) -> BenchPost {
    BenchPost {
        id,
        title: format!("Post {} by User {}", id, author_id),
        content: format!(
            "This is the content of post {} written by user {}",
            id, author_id
        ),
        author_id,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

fn bench_netabase_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Netabase insert benchmark
        group.bench_with_input(BenchmarkId::new("netabase", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let mut db =
                        NetabaseDatabase::<BenchSchema>::new_with_path(temp_dir.path()).unwrap();
                    let discriminants = BenchSchema::all_schema_discriminants();
                    db.initialize_trees_from_discriminants(&discriminants)
                        .unwrap();
                    let users: Vec<_> = (0..size).map(|i| create_bench_user(i as u64)).collect();
                    (db, users, temp_dir)
                },
                |(mut db, users, _temp_dir)| {
                    for user in users {
                        let schema = BenchSchema::BenchUser(user);
                        black_box(db.put_schema(&schema).unwrap());
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        // Raw sled insert benchmark
        group.bench_with_input(BenchmarkId::new("raw_sled", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let db = sled::open(temp_dir.path()).unwrap();
                    let tree = db.open_tree("bench_users").unwrap();
                    let users: Vec<_> = (0..size).map(|i| create_bench_user(i as u64)).collect();
                    (tree, users, temp_dir)
                },
                |(tree, users, _temp_dir)| {
                    for user in users {
                        let key = netabase_store::bincode::encode_to_vec(
                            &user.id,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        let value = netabase_store::bincode::encode_to_vec(
                            &user,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        black_box(tree.insert(key, value).unwrap());
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_netabase_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Netabase get benchmark
        group.bench_with_input(BenchmarkId::new("netabase", size), size, |b, &size| {
            // Setup data once
            let temp_dir = TempDir::new().unwrap();
            let mut db = NetabaseDatabase::<BenchSchema>::new_with_path(temp_dir.path()).unwrap();
            let discriminants = BenchSchema::all_schema_discriminants();
            db.initialize_trees_from_discriminants(&discriminants)
                .unwrap();

            // Insert test data
            for i in 0..size {
                let user = create_bench_user(i as u64);
                let schema = BenchSchema::BenchUser(user);
                db.put_schema(&schema).unwrap();
            }

            let keys: Vec<_> = (0..size)
                .map(|i| {
                    let user_key = BenchUserKey::Primary(BenchUserPrimaryKey(i as u64));
                    BenchSchemaKeys::from(user_key)
                })
                .collect();

            b.iter(|| {
                for key in &keys {
                    black_box(db.get_schema(key).unwrap());
                }
            });
        });

        // Raw sled get benchmark
        group.bench_with_input(BenchmarkId::new("raw_sled", size), size, |b, &size| {
            // Setup data once
            let temp_dir = TempDir::new().unwrap();
            let db = sled::open(temp_dir.path()).unwrap();
            let tree = db.open_tree("bench_users").unwrap();

            // Insert test data
            for i in 0..size {
                let user = create_bench_user(i as u64);
                let key = netabase_store::bincode::encode_to_vec(
                    &user.id,
                    netabase_store::bincode::config::standard(),
                )
                .unwrap();
                let value = netabase_store::bincode::encode_to_vec(
                    &user,
                    netabase_store::bincode::config::standard(),
                )
                .unwrap();
                tree.insert(key, value).unwrap();
            }

            let keys: Vec<_> = (0..size)
                .map(|i| {
                    netabase_store::bincode::encode_to_vec(
                        &(i as u64),
                        netabase_store::bincode::config::standard(),
                    )
                    .unwrap()
                })
                .collect();

            b.iter(|| {
                for key in &keys {
                    if let Some(value) = tree.get(key).unwrap() {
                        let _user: BenchUser = netabase_store::bincode::decode_from_slice(
                            &value,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap()
                        .0;
                        black_box(_user);
                    }
                }
            });
        });
    }
    group.finish();
}

fn bench_netabase_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    for size in [1000, 5000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Netabase mixed workload benchmark
        group.bench_with_input(BenchmarkId::new("netabase", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let mut db =
                        NetabaseDatabase::<BenchSchema>::new_with_path(temp_dir.path()).unwrap();
                    let discriminants = BenchSchema::all_schema_discriminants();
                    db.initialize_trees_from_discriminants(&discriminants)
                        .unwrap();

                    // Pre-populate with some data
                    for i in 0..size / 2 {
                        let user = create_bench_user(i as u64);
                        let schema = BenchSchema::BenchUser(user);
                        db.put_schema(&schema).unwrap();
                    }

                    (db, temp_dir)
                },
                |(mut db, _temp_dir)| {
                    let mut rng = rand::thread_rng();

                    // Mixed operations: 50% reads, 30% writes, 20% posts
                    for i in 0..size {
                        let op = rng.gen_range(0..10);
                        match op {
                            0..=4 => {
                                // Read operation (50%)
                                let user_id = rng.gen_range(0..size / 2) as u64;
                                let user_key = BenchUserKey::Primary(BenchUserPrimaryKey(user_id));
                                let schema_key = BenchSchemaKeys::from(user_key);
                                black_box(db.get_schema(&schema_key).unwrap());
                            }
                            5..=7 => {
                                // Write user operation (30%)
                                let user = create_bench_user(i as u64);
                                let schema = BenchSchema::BenchUser(user);
                                black_box(db.put_schema(&schema).unwrap());
                            }
                            8..=9 => {
                                // Write post operation (20%)
                                let post =
                                    create_bench_post(i as u64, rng.gen_range(0..size / 2) as u64);
                                let schema = BenchSchema::BenchPost(post);
                                black_box(db.put_schema(&schema).unwrap());
                            }
                            _ => unreachable!(),
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        // Raw sled mixed workload benchmark
        group.bench_with_input(BenchmarkId::new("raw_sled", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let db = sled::open(temp_dir.path()).unwrap();
                    let user_tree = db.open_tree("users").unwrap();
                    let post_tree = db.open_tree("posts").unwrap();

                    // Pre-populate with some data
                    for i in 0..size / 2 {
                        let user = create_bench_user(i as u64);
                        let key = netabase_store::bincode::encode_to_vec(
                            &user.id,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        let value = netabase_store::bincode::encode_to_vec(
                            &user,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        user_tree.insert(key, value).unwrap();
                    }

                    (user_tree, post_tree, temp_dir)
                },
                |(user_tree, post_tree, _temp_dir)| {
                    let mut rng = rand::thread_rng();

                    // Mixed operations: 50% reads, 30% writes, 20% posts
                    for i in 0..size {
                        let op = rng.gen_range(0..10);
                        match op {
                            0..=4 => {
                                // Read operation (50%)
                                let user_id = rng.gen_range(0..size / 2) as u64;
                                let key = netabase_store::bincode::encode_to_vec(
                                    &user_id,
                                    netabase_store::bincode::config::standard(),
                                )
                                .unwrap();
                                if let Some(value) = user_tree.get(&key).unwrap() {
                                    let _user: BenchUser =
                                        netabase_store::bincode::decode_from_slice(
                                            &value,
                                            netabase_store::bincode::config::standard(),
                                        )
                                        .unwrap()
                                        .0;
                                    black_box(_user);
                                }
                            }
                            5..=7 => {
                                // Write user operation (30%)
                                let user = create_bench_user(i as u64);
                                let key = netabase_store::bincode::encode_to_vec(
                                    &user.id,
                                    netabase_store::bincode::config::standard(),
                                )
                                .unwrap();
                                let value = netabase_store::bincode::encode_to_vec(
                                    &user,
                                    netabase_store::bincode::config::standard(),
                                )
                                .unwrap();
                                black_box(user_tree.insert(key, value).unwrap());
                            }
                            8..=9 => {
                                // Write post operation (20%)
                                let post =
                                    create_bench_post(i as u64, rng.gen_range(0..size / 2) as u64);
                                let key = netabase_store::bincode::encode_to_vec(
                                    &post.id,
                                    netabase_store::bincode::config::standard(),
                                )
                                .unwrap();
                                let value = netabase_store::bincode::encode_to_vec(
                                    &post,
                                    netabase_store::bincode::config::standard(),
                                )
                                .unwrap();
                                black_box(post_tree.insert(key, value).unwrap());
                            }
                            _ => unreachable!(),
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    for size in [100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Netabase batch insert
        group.bench_with_input(
            BenchmarkId::new("netabase_batch", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let mut db =
                            NetabaseDatabase::<BenchSchema>::new_with_path(temp_dir.path())
                                .unwrap();
                        let discriminants = BenchSchema::all_schema_discriminants();
                        db.initialize_trees_from_discriminants(&discriminants)
                            .unwrap();
                        let users: Vec<_> =
                            (0..size).map(|i| create_bench_user(i as u64)).collect();
                        (db, users, temp_dir)
                    },
                    |(mut db, users, _temp_dir)| {
                        // Simulate batch by doing multiple operations in sequence
                        for user in users {
                            let schema = BenchSchema::BenchUser(user);
                            black_box(db.put_schema(&schema).unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Raw sled batch insert
        group.bench_with_input(BenchmarkId::new("sled_batch", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let db = sled::open(temp_dir.path()).unwrap();
                    let tree = db.open_tree("users").unwrap();
                    let users: Vec<_> = (0..size).map(|i| create_bench_user(i as u64)).collect();
                    (tree, users, temp_dir)
                },
                |(tree, users, _temp_dir)| {
                    let mut batch = sled::Batch::default();
                    for user in users {
                        let key = netabase_store::bincode::encode_to_vec(
                            &user.id,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        let value = netabase_store::bincode::encode_to_vec(
                            &user,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        batch.insert(key, value);
                    }
                    black_box(tree.apply_batch(batch).unwrap());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_data_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_sizes");

    for data_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*data_size as u64));

        // Netabase with varying data sizes
        group.bench_with_input(
            BenchmarkId::new("netabase", data_size),
            data_size,
            |b, &data_size| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let mut db =
                            NetabaseDatabase::<BenchSchema>::new_with_path(temp_dir.path())
                                .unwrap();
                        let discriminants = BenchSchema::all_schema_discriminants();
                        db.initialize_trees_from_discriminants(&discriminants)
                            .unwrap();

                        let mut user = create_bench_user(1);
                        user.data = vec![0u8; data_size];

                        (db, user, temp_dir)
                    },
                    |(mut db, user, _temp_dir)| {
                        let schema = BenchSchema::BenchUser(user);
                        black_box(db.put_schema(&schema).unwrap());

                        let user_key = BenchUserKey::Primary(BenchUserPrimaryKey(1));
                        let schema_key = BenchSchemaKeys::from(user_key);
                        black_box(db.get_schema(&schema_key).unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Raw sled with varying data sizes
        group.bench_with_input(
            BenchmarkId::new("raw_sled", data_size),
            data_size,
            |b, &data_size| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let db = sled::open(temp_dir.path()).unwrap();
                        let tree = db.open_tree("users").unwrap();

                        let mut user = create_bench_user(1);
                        user.data = vec![0u8; data_size];

                        (tree, user, temp_dir)
                    },
                    |(tree, user, _temp_dir)| {
                        let key = netabase_store::bincode::encode_to_vec(
                            &user.id,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        let value = netabase_store::bincode::encode_to_vec(
                            &user,
                            netabase_store::bincode::config::standard(),
                        )
                        .unwrap();
                        black_box(tree.insert(&key, value).unwrap());

                        if let Some(retrieved_value) = tree.get(&key).unwrap() {
                            let _user: BenchUser = netabase_store::bincode::decode_from_slice(
                                &retrieved_value,
                                netabase_store::bincode::config::standard(),
                            )
                            .unwrap()
                            .0;
                            black_box(_user);
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_netabase_insert,
    bench_netabase_get,
    bench_netabase_mixed_workload,
    bench_batch_operations,
    bench_data_sizes
);
criterion_main!(benches);
