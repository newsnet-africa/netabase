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
#[netabase_schema_module(QuickBenchSchema, QuickBenchSchemaKeys)]
mod quick_bench_schema {
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
    #[key_name(QuickBenchUserKey)]
    pub struct QuickBenchUser {
        #[key]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub data: Vec<u8>,
    }
}

use quick_bench_schema::*;

fn create_quick_bench_user(id: u64) -> QuickBenchUser {
    let mut rng = rand::thread_rng();
    QuickBenchUser {
        id,
        name: format!("User{}", id),
        email: format!("user{}@example.com", id),
        data: (0..50).map(|_| rng.r#gen::<u8>()).collect(),
    }
}

fn quick_bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_insert");
    group.sample_size(10);

    for size in [10, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Netabase insert benchmark
        group.bench_with_input(BenchmarkId::new("netabase", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let mut db =
                        NetabaseDatabase::<QuickBenchSchema>::new_with_path(temp_dir.path())
                            .unwrap();
                    let discriminants = QuickBenchSchema::all_schema_discriminants();
                    db.initialize_trees_from_discriminants(&discriminants)
                        .unwrap();
                    let users: Vec<_> = (0..size)
                        .map(|i| create_quick_bench_user(i as u64))
                        .collect();
                    (db, users, temp_dir)
                },
                |(mut db, users, _temp_dir)| {
                    for user in users {
                        let schema = QuickBenchSchema::QuickBenchUser(user);
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
                    let tree = db.open_tree("quick_bench_users").unwrap();
                    let users: Vec<_> = (0..size)
                        .map(|i| create_quick_bench_user(i as u64))
                        .collect();
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

fn quick_bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_get");
    group.sample_size(10);

    for size in [10, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Netabase get benchmark
        group.bench_with_input(BenchmarkId::new("netabase", size), size, |b, &size| {
            // Setup data once
            let temp_dir = TempDir::new().unwrap();
            let mut db =
                NetabaseDatabase::<QuickBenchSchema>::new_with_path(temp_dir.path()).unwrap();
            let discriminants = QuickBenchSchema::all_schema_discriminants();
            db.initialize_trees_from_discriminants(&discriminants)
                .unwrap();

            // Insert test data
            for i in 0..size {
                let user = create_quick_bench_user(i as u64);
                let schema = QuickBenchSchema::QuickBenchUser(user);
                db.put_schema(&schema).unwrap();
            }

            let keys: Vec<_> = (0..size)
                .map(|i| {
                    let user_key = QuickBenchUserKey::Primary(QuickBenchUserPrimaryKey(i as u64));
                    QuickBenchSchemaKeys::from(user_key)
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
            let tree = db.open_tree("quick_bench_users").unwrap();

            // Insert test data
            for i in 0..size {
                let user = create_quick_bench_user(i as u64);
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
                        let _user: QuickBenchUser = netabase_store::bincode::decode_from_slice(
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

criterion_group!(quick_benches, quick_bench_insert, quick_bench_get);
criterion_main!(quick_benches);
