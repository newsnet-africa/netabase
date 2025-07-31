use bincode::Encode;

pub trait NetabaseSchema:
    Clone
    + From<libp2p::kad::Record>
    + Encode
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::Record>
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchemaKey:
    Clone
    + From<libp2p::kad::RecordKey>
    + Encode
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::RecordKey>
{
}
