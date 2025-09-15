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
        }
    }
}
