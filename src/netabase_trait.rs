use std::fmt::Debug;

use bincode::{Decode, Encode};

use crate::NetabaseError;

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
    fn put<R: TryInto<Self> + From<Self>>(value: Self) -> Result<(), NetabaseError>;

}

pub trait NetabaseSchemaKey:
    Clone
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::RecordKey>
{
    fn
}

pub trait NetabaseRegistery: Debug + Clone + Send {
    type KeyRegistry: NetabaseRegistryKey;

    fn unwrap<T: NetabaseSchema>(&self) -> T;
}
pub trait NetabaseRegistryKey: Debug + Clone + Send {
    fn unwrap<T: NetabaseSchema>(&self) -> T;
}
