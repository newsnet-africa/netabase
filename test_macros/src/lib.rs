use bincode::Encode;
use netabase::NetabaseSchema;
use netabase_macros::NetabaseSchema;

#[derive(NetabaseSchema, Encode, Decode)]
enum You {
    First(String, #[key] u128),
    Second(#[key] u128),
}
