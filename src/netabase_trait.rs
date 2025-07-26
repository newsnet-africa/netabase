use bincode::{Decode, Encode};

pub trait NetabaseSchema:
    Clone
    + From<libp2p::kad::Record>
    + From<crate::database::wrappers::RecordWrapper>
    + Encode
    + Decode<()>
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> &Self::Key;
}

pub trait NetabaseSchemaKey:
    Clone
    + From<libp2p::kad::Record>
    + From<crate::database::wrappers::RecordWrapper>
    + Encode
    + Decode<()>
{
}
