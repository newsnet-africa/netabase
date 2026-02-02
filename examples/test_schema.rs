//! Test schema for examples

use netabase_store::prelude::*;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(TestNetwork)]
pub mod test_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct UserProfile {
        #[primary_key]
        pub id: String,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub bio: String,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub author_id: String,
        pub content: String,
        #[secondary_key]
        pub created_at: u64,
    }
}
