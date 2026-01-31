use criterion::{black_box, criterion_group, criterion_main, Criterion};
use netabase::query::{DatabaseQuery, QueryExecutor, ValidateQuery};
use netabase::node::capabilities::{
    Capability, CapabilityPermission, CapabilityRange, PathRange
};
use netabase::node::metadata::{PublicNodeData, NodePublicKey};
use netabase::node::primitives::{Signature, SubspaceId};
use netabase_store::prelude::*;
use netabase_store::doc_examples::{ExampleDef, User, UserID, UserKeys, UserSecondaryKeys, UserEmail};
use netabase_store::databases::redb::RedbStore;
use libp2p::PeerId;
use std::time::{SystemTime, UNIX_EPOCH};

fn dummy_signature() -> Signature {
    Signature([0u8; 64])
}

fn dummy_node_data() -> PublicNodeData {
    PublicNodeData {
        node_id: PeerId::random(),
        public_key: NodePublicKey(SubspaceId::new([0u8; 32]).0),
    }
}

fn create_capability(
    range: CapabilityRange<ExampleDef, User>
) -> Capability<ExampleDef, User> {
    let node = dummy_node_data();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    Capability {
        subscription: (),
        owner: node.clone(),
        granted_by: node.clone(),
        issued_to: node,
        resource: CapabilityPermission::Read(range),
        expiry: now + 3600,
        signature: dummy_signature(),
        delegation: None,
    }
}

fn setup_db() -> RedbStore<ExampleDef> {
    let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
    let txn = store.begin_write().unwrap();
    
    // Populate 100 users
    for i in 0..100 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            name: if i % 2 == 0 { "Alice".to_string() } else { "Bob".to_string() },
            email: format!("user{}@example.com", i),
        };
        txn.create(&user).unwrap();
    }
    txn.commit().unwrap();
    store
}

fn benchmark_queries(c: &mut Criterion) {
    let store = setup_db();
    let txn = store.begin_read().unwrap();

    let cap_full = create_capability(CapabilityRange::FullTable);
    
    let cap_prefix = create_capability(CapabilityRange::PrimaryRange(
        PathRange::PathPrefix(netabase::store::primitives::EntryPath("user_".into()))
    ));

    let cap_secondary = create_capability(CapabilityRange::SecondaryRange(
        PathRange::Range {
            start: UserSecondaryKeys::Email(UserEmail("user50@example.com".to_string())),
            end: UserSecondaryKeys::Email(UserEmail("user50@example.com".to_string())),
        }
    ));

    let query_get = DatabaseQuery::<ExampleDef, User>::Get { 
        key: UserID("user_50".to_string()) 
    };

    let query_range = DatabaseQuery::<ExampleDef, User>::Range {
        start: Some(UserID("user_0".to_string())),
        end: Some(UserID("user_9".to_string())),
        limit: None,
    };

    let query_secondary = DatabaseQuery::<ExampleDef, User>::GetBySecondary {
        key: UserSecondaryKeys::Email(UserEmail("user50@example.com".to_string()))
    };

    let mut group = c.benchmark_group("Validation");
    
    group.bench_function("validate_get_full_cap", |b| {
        b.iter(|| {
            black_box(&query_get).validate(black_box(&cap_full))
        })
    });

    group.bench_function("validate_get_prefix_cap", |b| {
        b.iter(|| {
            black_box(&query_get).validate(black_box(&cap_prefix))
        })
    });

    group.bench_function("validate_secondary_cap", |b| {
        b.iter(|| {
            black_box(&query_secondary).validate(black_box(&cap_secondary))
        })
    });
    
    group.finish();

    let mut group = c.benchmark_group("Execution");

    group.bench_function("execute_get", |b| {
        b.iter(|| {
            txn.execute(black_box(query_get.clone())).unwrap()
        })
    });

    group.bench_function("execute_range_10_items", |b| {
        b.iter(|| {
            txn.execute(black_box(query_range.clone())).unwrap()
        })
    });

    group.bench_function("execute_secondary_exact_50_items", |b| {
        b.iter(|| {
            txn.execute(black_box(query_secondary.clone())).unwrap()
        })
    });

    group.finish();

    let mut group = c.benchmark_group("End-to-End");

    group.bench_function("e2e_get_full_flow", |b| {
        b.iter(|| {
            let q = black_box(query_get.clone());
            let c = black_box(&cap_full);
            q.validate(c).unwrap();
            txn.execute(q).unwrap()
        })
    });

    group.bench_function("e2e_secondary_full_flow", |b| {
        b.iter(|| {
            let q = black_box(query_secondary.clone());
            let c = black_box(&cap_secondary);
            q.validate(c).unwrap();
            txn.execute(q).unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_queries);
criterion_main!(benches);
