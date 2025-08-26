use netabase::NetabaseSchema;
use netabase_macros::NetabaseSchema;

#[derive(NetabaseSchema)]
struct Me {
    #[key]
    hi: String,
    you: String,
}
