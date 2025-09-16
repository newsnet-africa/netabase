use crate::data::v1;
use netabase::NetabaseCatalog;
use netabase_macros::NetabaseCatalog;
use netabase_macros::netabase_schema;

#[netabase_schema(MySchema)]
pub mod data {
    use native_db::{ToKey, native_db};
    use native_model::{Model, native_model};
    use serde::{Deserialize, Serialize};

    pub type Person = v1::Person;

    pub mod v1 {
        use super::*;

        #[derive(Serialize, Deserialize, Debug)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        pub struct Person {
            #[primary_key]
            pub name: String,
            #[secondary_key]
            pub email: String,
            #[secondary_key]
            pub job: String,
        }
        #[derive(Serialize, Deserialize, Debug)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        pub struct Person2 {
            #[primary_key]
            pub name: String,
            #[secondary_key]
            pub email: String,
            #[secondary_key]
            pub job: String,
        }

        pub mod third {

            use crate::data::Deserialize;
            use crate::data::Serialize;
            use crate::data::native_db;
            use crate::data::native_model;
            use native_db::ToKey;
            use native_model::Model;

            #[derive(Serialize, Deserialize, Debug)]
            #[native_model(id = 1, version = 1)]
            #[native_db]
            pub struct Person3 {
                #[primary_key]
                pub name: String,
                #[secondary_key]
                pub email: String,
                #[secondary_key]
                pub job: String,
            }
        }
    }
}
