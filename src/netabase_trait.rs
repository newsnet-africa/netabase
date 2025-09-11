use std::fmt::Debug;

use bincode::{Decode, Encode};

use crate::{Netabase, NetabaseError};

pub trait NetabaseSchema<R: NetabaseRegistery>:
    Clone
    + Debug
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::Record>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::Record>
    + TryInto<R>
    + From<R>
{
    type Key: NetabaseSchemaKey<R::KeyRegistry>;
    fn key(&self) -> Self::Key;

    fn put(&self, netabase: Netabase<R>) -> Result<(), NetabaseError>
    where
        R: NetabaseRegistery + TryInto<Self> + From<Self>;
}

pub trait NetabaseSchemaKey<K: NetabaseRegistryKey>:
    Clone
    + Debug
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + TryInto<K>
    + From<K>
{
    fn get<T, R>(&self, netabase: Netabase<R>) -> Result<Option<T>, NetabaseError>
    where
        R: NetabaseRegistery,
        T: NetabaseSchema<R> + TryFrom<R> + Into<R>;

    fn delete<R>(&self, netabase: Netabase<R>)
    where
        R: NetabaseRegistery;
}

pub trait NetabaseRegistery: Debug + Clone + Send {
    type KeyRegistry: NetabaseRegistryKey;

    fn unwrap(self) -> impl NetabaseSchema<Self> + TryFrom<Self> + From<Self>;
}

pub trait NetabaseRegistryKey: Debug + Clone + Send {
    fn unwrap(self) -> impl NetabaseSchemaKey<Self> + TryFrom<Self> + From<Self>;
}
