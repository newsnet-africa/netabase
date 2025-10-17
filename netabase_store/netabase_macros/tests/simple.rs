use netabase_macros::NetabaseModel;

pub trait NetabaseModel {}

#[derive(NetabaseModel)]
struct NewThing {
    #[primary_key]
    hi: String,
    #[secondary_key]
    you_key: String,
    #[secondary_key]
    there: String,
}
