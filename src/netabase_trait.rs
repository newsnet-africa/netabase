use std::fmt::Debug;

use async_trait::async_trait;
use bincode::{Decode, Encode};

use crate::{Netabase, NetabaseError};

#[async_trait]
pub trait NetabaseSchema<R: 'static + NetabaseRegistery>:
    Clone
    + Debug
    + Send
    + TryFrom<::macro_exports::__netabase_libp2p_kad::Record>
    + Encode
    + Decode<()>
    + TryInto<::macro_exports::__netabase_libp2p_kad::Record>
    + TryInto<R>
    + From<R>
where
    R::KeyRegistry: NetabaseRegistryKey,
{
    type Key: NetabaseSchemaKey<R::KeyRegistry, Schema = Self>;
    fn key(&self) -> Self::Key;

    async fn put(self, netabase: Netabase<R>) -> Result<(), NetabaseError>
    where
        R: NetabaseRegistery + TryInto<Self> + From<Self>,
    {
        netabase.put(self.into()).await
    }
}

#[async_trait]
pub trait NetabaseSchemaKey<K: 'static + NetabaseRegistryKey>:
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
    type Schema: NetabaseSchema<K::SchemaRegistry, Key = Self>;
}

pub trait NetabaseRegistery: NetabaseSchema<Self>
where
    Self: 'static,
{
    type KeyRegistry: NetabaseRegistryKey<SchemaRegistry = Self>;
}

#[async_trait]
pub trait NetabaseRegistryKey: NetabaseSchemaKey<Self>
where
    Self: 'static,
{
    type SchemaRegistry: NetabaseRegistery<KeyRegistry = Self>;
    fn get<T, R>(&self, netabase: Netabase<R>) -> Result<Option<T>, NetabaseError>
    where
        R: NetabaseRegistery,
        T: NetabaseSchema<R> + TryFrom<R> + Into<R>;

    async fn delete<R>(self, netabase: Netabase<R>) -> Result<(), NetabaseError>
    where
        R: NetabaseRegistery<KeyRegistry = Self>,
    {
        netabase.delete(self).await
    }
}
