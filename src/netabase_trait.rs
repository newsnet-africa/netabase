use bincode::{Decode, Encode};

pub trait NetabaseSchema:
    Clone
    + Send
    + From<::macro_exports::__netabase_libp2p_kad::Record>
    + Encode
    + Decode<()>
    + Into<::macro_exports::__netabase_libp2p_kad::Record> // TODO: TryFrom
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchemaKey:
    Clone
    + Send
    + From<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + Encode
    + Decode<()>
    + Into<::macro_exports::__netabase_libp2p_kad::RecordKey>
{
}
