use netabase::query::{
    DatabaseQuery, DatabaseQueryResult, QueryExecutor, ValidateQuery, ValidationError
};
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

// Helper to create a dummy signature
fn dummy_signature() -> Signature {
    Signature([0u8; 64])
}

// Helper to create a dummy peer
fn dummy_peer() -> PeerId {
    PeerId::random()
}

// Helper to create dummy node data
fn dummy_node_data(owner: SubspaceId, peer: PeerId) -> PublicNodeData {
    PublicNodeData {
        node_id: peer,
        public_key: NodePublicKey(owner.0),
    }
}

// Helper to create a dummy owner
fn dummy_owner(byte: u8) -> SubspaceId {
    SubspaceId::new([byte; 32])
}

fn create_capability(
    owner: PublicNodeData, 
    grantee: PublicNodeData,
    range: CapabilityRange<ExampleDef, User>
) -> Capability<ExampleDef, User> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    Capability {
        subscription: (),
        owner: owner.clone(),
        granted_by: owner.clone(),
        issued_to: grantee,
        resource: CapabilityPermission::Read(range),
        expiry: now + 3600,
        signature: dummy_signature(),
        delegation: None,
    }
}

#[test]
fn test_query_integration_flow() {
    // 1. Setup Database (In-Memory)
    let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
    
    // 2. Populate Data
    {
        let txn = store.begin_write().unwrap();
        let user1 = User {
            id: UserID("alice".into()),
            name: "Alice".into(),
            email: "alice@example.com".into(),
        };
        let user2 = User {
            id: UserID("bob".into()),
            name: "Bob".into(),
            email: "bob@example.com".into(),
        };
        txn.create(&user1).unwrap();
        txn.create(&user2).unwrap();
        txn.commit().unwrap();
    }

    // 3. Setup Nodes
    let owner_id = dummy_owner(0xAA);
    let owner_peer = dummy_peer();
    let owner_data = dummy_node_data(owner_id, owner_peer);
    
    let client_id = dummy_owner(0xBB);
    let client_peer = dummy_peer();
    let client_data = dummy_node_data(client_id, client_peer);

    let read_txn = store.begin_read().unwrap();
    
    // =========================================================================
    // Test A: Valid GET Query
    // =========================================================================
    
    // Capability allowing access to "alice"
    let cap_alice = create_capability(
        owner_data.clone(), 
        client_data.clone(), 
        CapabilityRange::PrimaryRange(PathRange::Range {
            start: UserID("alice".into()),
            end: UserID("alice".into())
        })
    );

    let query_get_alice = DatabaseQuery::Get { key: UserID("alice".into()) };

    // Validation
    assert!(query_get_alice.validate(&cap_alice).is_ok());

    // Execution via RedbTransaction (read_txn)
    let result = read_txn.execute(query_get_alice).unwrap();
    if let DatabaseQueryResult::Record(Some(user)) = result {
        assert_eq!(user.name, "Alice");
    } else {
        panic!("Expected Record(Some(Alice))");
    }

    // =========================================================================
    // Test B: Out of Scope GET Query
    // =========================================================================
    
    // Try to access "bob" with "alice" capability
    let query_get_bob = DatabaseQuery::Get { key: UserID("bob".into()) };
    
    // Validation should fail
    let val_err = query_get_bob.validate(&cap_alice);
    assert!(matches!(val_err, Err(ValidationError::OutOfScope { .. })));

    // =========================================================================
    // Test C: Valid RANGE Query with FullTable Capability
    // =========================================================================
    
    let cap_full = create_capability(
        owner_data.clone(),
        client_data.clone(),
        CapabilityRange::FullTable
    );

    let query_range = DatabaseQuery::Range { 
        start: Some(UserID("alice".into())), 
        end: Some(UserID("bob".into())),
        limit: None
    };

    assert!(query_range.validate(&cap_full).is_ok());

    let result_range = read_txn.execute(query_range).unwrap();
    if let DatabaseQueryResult::Range(users) = result_range {
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|u| u.name == "Alice"));
        assert!(users.iter().any(|u| u.name == "Bob"));
    } else {
        panic!("Expected Range result");
    }

    // =========================================================================
    // Test D: EXISTS Query
    // =========================================================================
    
    let query_exists = DatabaseQuery::Exists { key: UserID("alice".into()) };
    assert!(query_exists.validate(&cap_full).is_ok());
    
    let result_exists = read_txn.execute(query_exists).unwrap();
    assert_eq!(result_exists, DatabaseQueryResult::Exists(true));

    // =========================================================================
    // Test E: Valid Secondary Key Query
    // =========================================================================
    
    // Capability allowing access to users with email "alice@example.com" via Secondary Range
    let cap_secondary = create_capability(
        owner_data.clone(),
        client_data.clone(),
        CapabilityRange::SecondaryRange(PathRange::Range {
            start: UserSecondaryKeys::Email(UserEmail("alice@example.com".to_string())),
            end: UserSecondaryKeys::Email(UserEmail("alice@example.com".to_string())),
        })
    );

    let query_secondary_valid = DatabaseQuery::GetBySecondary { 
        key: UserSecondaryKeys::Email(UserEmail("alice@example.com".to_string())) 
    };

    assert!(query_secondary_valid.validate(&cap_secondary).is_ok());

    let result_secondary = read_txn.execute(query_secondary_valid).unwrap();
    if let DatabaseQueryResult::Range(users) = result_secondary {
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "Alice");
    } else {
        panic!("Expected Range result for secondary query");
    }

    // =========================================================================
    // Test F: Out of Scope Secondary Key Query
    // =========================================================================
    
    let query_secondary_invalid = DatabaseQuery::GetBySecondary { 
        key: UserSecondaryKeys::Email(UserEmail("bob@example.com".to_string())) 
    };

    let val_err_sec = query_secondary_invalid.validate(&cap_secondary);
    assert!(matches!(val_err_sec, Err(ValidationError::OutOfScope { .. })));
}
