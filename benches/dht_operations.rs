//! Benchmarks for DHT operations
//!
//! These benchmarks measure the performance of key operations:
//! - Record storage (put_record)
//! - Record retrieval (get_record)
//! - Provider operations
//! - Bootstrap operations
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use netabase::Netabase;
use netabase_store::*;
use tokio::runtime::Runtime;
use tempfile::TempDir;

// Test schema for benchmarks
#[netabase_definition_module(BenchDefinition, BenchKeys)]
mod bench_schema {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BenchDefinition)]
    pub struct BenchRecord {
        #[primary_key]
        pub id: u64,
        pub data: Vec<u8>,
    }
}

use bench_schema::*;

/// Benchmark: Creating a new Netabase instance
fn bench_netabase_creation(c: &mut Criterion) {
    c.bench_function("netabase_creation", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let _ = Netabase::<BenchDefinition>::new_with_path(temp_dir.path());
        });
    });
}

/// Benchmark: Starting and stopping the swarm
fn bench_swarm_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("swarm_lifecycle", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let mut netabase = Netabase::<BenchDefinition>::new_with_path(temp_dir.path()).unwrap();
                let _ = netabase.start_swarm().await;
                let _ = netabase.stop_swarm().await;
            });
        });
    });
}

/// Benchmark: Storing records locally
fn bench_local_record_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("local_record_storage");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let temp_dir = TempDir::new().unwrap();
            let mut netabase = rt.block_on(async {
                let mut nb = Netabase::<BenchDefinition>::new_with_path(temp_dir.path()).unwrap();
                let _ = nb.start_swarm().await;
                nb
            });

            b.iter(|| {
                rt.block_on(async {
                    let record = BenchRecord {
                        id: black_box(rand::random::<u64>()),
                        data: vec![0u8; size],
                    };
                    let _ = netabase.put_record_locally(BenchDefinition::BenchRecord(record)).await;
                });
            });

            rt.block_on(async {
                let _ = netabase.stop_swarm().await;
            });
        });
    }

    group.finish();
}

/// Benchmark: Querying local records
fn bench_query_local_records(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("query_local_records");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let temp_dir = TempDir::new().unwrap();
            let mut netabase = rt.block_on(async {
                let mut nb = Netabase::<BenchDefinition>::new_with_path(temp_dir.path()).unwrap();
                let _ = nb.start_swarm().await;

                // Pre-populate with records
                for i in 0..count {
                    let record = BenchRecord {
                        id: i,
                        data: vec![0u8; 100],
                    };
                    let _ = nb.put_record_locally(BenchDefinition::BenchRecord(record)).await;
                }

                nb
            });

            b.iter(|| {
                rt.block_on(async {
                    let _ = netabase.query_local_records(None).await;
                });
            });

            rt.block_on(async {
                let _ = netabase.stop_swarm().await;
            });
        });
    }

    group.finish();
}

/// Benchmark: Publishing records to DHT
fn bench_dht_put_record(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("dht_put_record");

    for size in [100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let temp_dir = TempDir::new().unwrap();
            let mut netabase = rt.block_on(async {
                let mut nb = Netabase::<BenchDefinition>::new_with_path(temp_dir.path()).unwrap();
                let _ = nb.start_swarm().await;
                nb
            });

            b.iter(|| {
                rt.block_on(async {
                    let record = BenchRecord {
                        id: black_box(rand::random::<u64>()),
                        data: vec![0u8; size],
                    };
                    // Note: This will timeout in benchmark as there are no peers,
                    // but we're measuring the local overhead
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_millis(100),
                        netabase.put_record(record)
                    ).await;
                });
            });

            rt.block_on(async {
                let _ = netabase.stop_swarm().await;
            });
        });
    }

    group.finish();
}

/// Benchmark: Subscribing and unsubscribing from events
fn bench_event_subscription(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("event_subscription", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mut netabase = rt.block_on(async {
            let mut nb = Netabase::<BenchDefinition>::new_with_path(temp_dir.path()).unwrap();
            let _ = nb.start_swarm().await;
            nb
        });

        b.iter(|| {
            let _receiver = netabase.subscribe_to_broadcasts();
            drop(_receiver);
        });

        rt.block_on(async {
            let _ = netabase.stop_swarm().await;
        });
    });
}

criterion_group!(
    benches,
    bench_netabase_creation,
    bench_swarm_lifecycle,
    bench_local_record_storage,
    bench_query_local_records,
    bench_dht_put_record,
    bench_event_subscription,
);

criterion_main!(benches);
