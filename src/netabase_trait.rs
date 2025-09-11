use std::fmt::Debug;

use bincode::{Decode, Encode};

use crate::{Netabase, NetabaseError};

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

    fn put<R>(&self, netabase: Netabase<R::KeyRegistry, R>) -> Result<(), NetabaseError>
    where
        R: NetabaseRegistery + TryInto<Self> + From<Self>;
}

pub trait NetabaseSchemaKey:
    Clone
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::RecordKey>
{
    fn get<R, T>(&self, netabase: Netabase<R::KeyRegistry, R>) -> Result<Option<T>, NetabaseError>
    where
        R: NetabaseRegistery,
        T: NetabaseSchema + TryFrom<R> + Into<R>;

    fn delete<R, T>(&self, netabase: Netabase<R::KeyRegistry, R>)
    where
        R: NetabaseRegistery,
        T: NetabaseSchema + TryFrom<R> + Into<R>;
}

pub trait NetabaseRegistery: Debug + Clone + Send {
    type KeyRegistry: NetabaseRegistryKey;

    fn unwrap<T>(&self) -> T
    where
        T: NetabaseSchema + TryFrom<Self> + From<Self>;
}

pub trait NetabaseRegistryKey: Debug + Clone + Send {
    fn unwrap<T>(&self) -> T::Key
    where
        T: NetabaseSchema,
        T::Key: Into<Self> + TryFrom<Self>;
}
