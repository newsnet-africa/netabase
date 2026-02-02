use netabase::node::capabilities::{Capability, CapabilityPermission, CapabilityRange, PathRange};
use netabase::node::key::NetabasePath;
use netabase::node::primitives::{NamespaceId, Operation, Signature, SubspaceId};
use netabase::node::metadata::{PublicNodeData, NodePublicKey};
use netabase::store::primitives::EntryPath;
use netabase_store::prelude::*;
use netabase_store::doc_examples::{ExampleDef, User, UserID};
use netabase_store::traits::registry::definition::redb_definition::RedbDefinition;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use libp2p::PeerId;
use serde_big_array::BigArray;

// Helper to create a dummy signature
fn dummy_signature() -> Signature {
    Signature([0u8; 64])
}

// Helper to create a dummy peer
fn dummy_peer() -> PeerId {
    PeerId::random()
}

// Helper to create a dummy namespace
fn dummy_namespace() -> NamespaceId {
    NamespaceId([1u8; 32])
}

// Helper to create a dummy owner
fn dummy_owner(byte: u8) -> SubspaceId {
    SubspaceId::new([byte; 32])
}

// Helper to create dummy node data
fn dummy_node_data(owner: SubspaceId, peer: PeerId) -> PublicNodeData {
    PublicNodeData {
        node_id: peer,
        public_key: NodePublicKey(owner.0),
    }
}

#[test]
fn test_capability_creation_and_path_generation() {
    let owner = dummy_owner(0xAA);
    let peer = dummy_peer();
    let user_id = UserID("alice".to_string());
    
    // 1. Path (Record) Scope -> PrimaryRange(Range)
    let path_range = CapabilityRange::<ExampleDef, User>::PrimaryRange(
        PathRange::Range {
            start: user_id.clone(),
            end: user_id.clone(),
        }
    );
    
    let path_bytes = path_range.to_path();
    // to_path implementation for Range only returns the Key bytes (start key) currently
    let key_bytes = netabase_store::postcard::to_allocvec(&user_id).unwrap();
    assert_eq!(path_bytes, key_bytes);

    // 2. FullTable Scope
    let table_range = CapabilityRange::<ExampleDef, User>::FullTable;
    let table_bytes = table_range.to_path();
    assert!(table_bytes.is_empty());
    
    // 3. Prefix Scope -> PrimaryRange(PathPrefix)
    let prefix = vec![0x01, 0x02];
    let prefix_range = CapabilityRange::<ExampleDef, User>::PrimaryRange(
        PathRange::PathPrefix(EntryPath(prefix.clone()))
    );
    let pre_bytes = prefix_range.to_path();
    assert_eq!(pre_bytes, prefix);
}

#[test]
fn test_capability_verification() {
    let owner_id = dummy_owner(0xBB);
    let peer_id = dummy_peer();
    let owner_data = dummy_node_data(owner_id, peer_id);
    
    let grantee_peer = dummy_peer();
    // Grantee node data needs an owner ID too. Assume they are their own owner or irrelevant?
    // In capability, issued_to is PublicNodeData.
    let grantee_data = dummy_node_data(dummy_owner(0xCC), grantee_peer);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Valid Read Capability on Subspace (Root)
    let cap = Capability::<ExampleDef, User> {
        subscription: (),
        owner: owner_data.clone(),
        granted_by: owner_data.clone(), // Root signed by owner
        issued_to: grantee_data,
        resource: CapabilityPermission::Read(CapabilityRange::FullTable),
        expiry: now + 1000,
        signature: dummy_signature(),
        delegation: None,
    };

    // Verify chain against the owner's public key
    assert!(cap.verify_chain(&owner_data.public_key));
}

#[test]
fn test_delegation_chain() {
    let root_owner_id = dummy_owner(0xCC);
    let root_peer = dummy_peer();
    let root_data = dummy_node_data(root_owner_id, root_peer);

    let node_a_peer = dummy_peer();
    let node_a_data = dummy_node_data(dummy_owner(0xA1), node_a_peer);

    let node_b_peer = dummy_peer();
    let node_b_data = dummy_node_data(dummy_owner(0xB1), node_b_peer);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // 1. Root Capability: Owner -> NodeA (Read FullTable)
    let root_cap = Capability::<ExampleDef, User> {
        subscription: (),
        owner: root_data.clone(),
        granted_by: root_data.clone(),
        issued_to: node_a_data.clone(),
        resource: CapabilityPermission::Read(CapabilityRange::FullTable),
        expiry: now + 2000,
        signature: dummy_signature(),
        delegation: None,
    };

    // 2. Delegated Capability: NodeA -> NodeB (Read specific path)
    let user_id = UserID("bob".to_string());
    let delegated_cap = Capability::<ExampleDef, User> {
        subscription: (),
        owner: root_data.clone(),
        granted_by: node_a_data.clone(), // Signed by NodeA
        issued_to: node_b_data,
        resource: CapabilityPermission::Read(
            CapabilityRange::PrimaryRange(PathRange::Range {
                start: user_id.clone(),
                end: user_id,
            })
        ),
        expiry: now + 1000,
        signature: dummy_signature(), 
        delegation: Some(Box::new(root_cap)),
    };

    // Verify the delegated capability chain against the root owner
    assert!(delegated_cap.verify_chain(&root_data.public_key));
}
