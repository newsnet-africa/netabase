use bincode::{Decode, Encode};

pub trait NetabaseSchema:
    Clone
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::Record>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::Record>
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchemaKey:
    Clone
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::RecordKey>
{
}
