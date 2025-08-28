use bincode::Decode;
use bincode::Encode;
use netabase::netabase_trait::NetabaseSchema;
use netabase_macros::NetabaseSchema;
use netabase_macros::NetabaseSchemaKey;
use netabase_macros::schema_module;

#[schema_module]
pub mod schemas {
    use bincode::Decode;
    use bincode::Encode;
    use netabase::netabase_trait::NetabaseSchema as NetabaseSchemaTrait;
    use netabase_macros::NetabaseSchema;
    use netabase_macros::NetabaseSchemaKey;

    #[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
    pub struct Me {
        #[key]
        first: String,
        second: u128,
    }

    #[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
    pub enum You {
        First(#[key] String, u128),
        Second(#[key] u128),
        Third {
            #[key]
            name: String,
        },
    }
}
