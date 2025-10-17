#![cfg(feature = "native")]

use netabase_macros::netabase_definition_module;
use netabase_store::databases::sled_store::{SledStore, SledStoreTree};
use netabase_store::traits::model::NetabaseModel;

// Test schema
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
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
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
    }

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
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub published: bool,
    }

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
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub post_id: u64,
        pub author: String,
        pub content: String,
    }
}

use blog::*;

#[test]
fn test_sled_store_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<BlogDefinition>::new(temp_dir.path());
    assert!(store.is_ok());
}

#[test]
fn test_temp_store_creation() {
    let store = SledStore::<BlogDefinition>::temp();
    assert!(store.is_ok());
}

#[test]
fn test_put_and_get_user() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Insert user
    assert!(user_tree.put(alice.clone()).is_ok());

    // Retrieve user
    let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert_eq!(Some(alice), retrieved);
}

#[test]
fn test_remove_user() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Insert and remove
    user_tree.put(alice.clone()).unwrap();
    let removed = user_tree.remove(UserPrimaryKey(1)).unwrap();
    assert_eq!(Some(alice), removed);

    // Verify it's gone
    let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert_eq!(None, retrieved);
}

#[test]
fn test_multiple_models() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Insert users
    let user_tree = store.open_tree::<User>();
    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let bob = User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    user_tree.put(alice.clone()).unwrap();
    user_tree.put(bob.clone()).unwrap();

    // Insert posts
    let post_tree = store.open_tree::<Post>();
    let post1 = Post {
        id: 1,
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author_id: 1,
        published: true,
    };
    let post2 = Post {
        id: 2,
        title: "Draft".to_string(),
        content: "Work in progress".to_string(),
        author_id: 2,
        published: false,
    };
    post_tree.put(post1.clone()).unwrap();
    post_tree.put(post2.clone()).unwrap();

    // Verify users
    assert_eq!(Some(alice), user_tree.get(UserPrimaryKey(1)).unwrap());
    assert_eq!(Some(bob), user_tree.get(UserPrimaryKey(2)).unwrap());

    // Verify posts
    assert_eq!(Some(post1), post_tree.get(PostPrimaryKey(1)).unwrap());
    assert_eq!(Some(post2), post_tree.get(PostPrimaryKey(2)).unwrap());
}

#[test]
fn test_iteration() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        User {
            id: 3,
            username: "carol".to_string(),
            email: "carol@example.com".to_string(),
        },
    ];

    for user in &users {
        user_tree.put(user.clone()).unwrap();
    }

    let mut retrieved = Vec::new();
    for result in user_tree.iter() {
        let (_key, user) = result.unwrap();
        retrieved.push(user);
    }

    assert_eq!(users.len(), retrieved.len());
    for user in &users {
        assert!(retrieved.contains(user));
    }
}

#[test]
fn test_tree_len_and_is_empty() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    assert!(user_tree.is_empty());
    assert_eq!(0, user_tree.len());

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    user_tree.put(alice).unwrap();

    assert!(!user_tree.is_empty());
    assert_eq!(1, user_tree.len());
}

#[test]
fn test_clear() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    user_tree.put(alice).unwrap();

    assert_eq!(1, user_tree.len());

    user_tree.clear().unwrap();
    assert!(user_tree.is_empty());
}

#[test]
fn test_secondary_key_lookup() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let bob = User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    user_tree.put(alice.clone()).unwrap();
    user_tree.put(bob.clone()).unwrap();

    // Look up by secondary key (email)
    let results = user_tree
        .get_by_secondary_key(UserSecondaryKeys::Email(EmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .unwrap();

    assert_eq!(1, results.len());
    assert_eq!(alice, results[0]);
}

#[test]
fn test_secondary_key_multiple_results() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let post_tree = store.open_tree::<Post>();

    let post1 = Post {
        id: 1,
        title: "Post 1".to_string(),
        content: "Content 1".to_string(),
        author_id: 1,
        published: true,
    };
    let post2 = Post {
        id: 2,
        title: "Post 2".to_string(),
        content: "Content 2".to_string(),
        author_id: 1,
        published: true,
    };
    let post3 = Post {
        id: 3,
        title: "Post 3".to_string(),
        content: "Content 3".to_string(),
        author_id: 2,
        published: true,
    };

    post_tree.put(post1.clone()).unwrap();
    post_tree.put(post2.clone()).unwrap();
    post_tree.put(post3.clone()).unwrap();

    // Look up posts by author_id = 1
    let results = post_tree
        .get_by_secondary_key(PostSecondaryKeys::AuthorId(AuthorIdSecondaryKey(1)))
        .unwrap();

    // Should find 2 posts
    assert_eq!(2, results.len());
    assert!(results.contains(&post1));
    assert!(results.contains(&post2));
}

#[test]
fn test_update_model() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user_tree.put(alice.clone()).unwrap();

    let updated_alice = User {
        id: 1,
        username: "alice_updated".to_string(),
        email: "alice_new@example.com".to_string(),
    };

    user_tree.put(updated_alice.clone()).unwrap();

    let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert_eq!(Some(updated_alice), retrieved);
}

#[test]
fn test_flush() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user_tree.put(alice).unwrap();

    // Flush should succeed
    let result = store.flush();
    assert!(result.is_ok());
}

#[test]
fn test_trait_implementations() {
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Test NetabaseModel trait
    let primary_key = user.primary_key();
    assert_eq!(UserPrimaryKey(1), primary_key);

    let secondary_keys = user.secondary_keys();
    assert_eq!(1, secondary_keys.len());

    assert_eq!("User", User::discriminant_name());
}
