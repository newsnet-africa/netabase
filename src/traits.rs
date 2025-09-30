use anyhow::anyhow;
use bincode::{Decode, Encode};
use libp2p::kad::{Record, RecordKey};

use crate::errors::NetabaseError;

pub trait NetabaseModels:
    Encode + Decode<()> + Sized + TryInto<Record> + TryFrom<Record> + Send + Sync
where
    Self::Key: NetabaseSchemaKey,
    <<Self as NetabaseModels>::Key as TryInto<libp2p::kad::RecordKey>>::Error: std::marker::Send,
    <<Self as NetabaseModels>::Key as TryInto<libp2p::kad::RecordKey>>::Error: std::marker::Sync,
    libp2p::kad::RecordKey: TryFrom<<Self as NetabaseModels>::Key>,
    <<Self as NetabaseModels>::Key as std::convert::TryInto<libp2p::kad::RecordKey>>::Error:
        std::error::Error,
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;

    fn to_record(&self) -> Result<Record, NetabaseError>
    where
        <Self as TryInto<libp2p::kad::Record>>::Error: std::marker::Send,
        <Self as TryInto<libp2p::kad::Record>>::Error: std::marker::Sync,
        <Self as TryInto<libp2p::kad::Record>>::Error: 'static,
        <<Self as NetabaseModels>::Key as TryInto<libp2p::kad::RecordKey>>::Error:
            std::error::Error,
        <<Self as NetabaseModels>::Key as TryInto<libp2p::kad::RecordKey>>::Error:
            std::marker::Send,
        <<Self as NetabaseModels>::Key as TryInto<libp2p::kad::RecordKey>>::Error:
            std::marker::Sync,
        <<Self as NetabaseModels>::Key as TryInto<libp2p::kad::RecordKey>>::Error: 'static,
    {
        let key = match self.key().try_into() {
            Ok(t) => t,
            Err(e) => todo!("Fix this error conversion: {e:?}"),
        };

        Ok(Record {
            key,
            value: bincode::encode_to_vec(self, bincode::config::standard())?,
            publisher: None,
            expires: None,
        })
    }
    fn from_record(record: Record) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.value, bincode::config::standard())?.0)
    }
}

pub trait NetabaseSchemaKey:
    Encode
    + Decode<()>
    + TryInto<RecordKey>
    + TryFrom<RecordKey>
    + Clone
    + Sized
    + Send
    + Sync
    + 'static
where
    libp2p::kad::RecordKey: TryFrom<Self>,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: std::error::Error,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: std::marker::Send,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: std::marker::Sync,
    <Self as TryInto<libp2p::kad::RecordKey>>::Error: 'static,
{
    fn to_record_key(&self) -> Result<RecordKey, NetabaseError> {
        Ok(RecordKey::new(&bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }
    fn from_record_key(record: RecordKey) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.to_vec(), bincode::config::standard())?.0)
    }
}

pub mod database_traits {
    pub trait NetabaseDatabase<Schema: strum::VariantNames> {
        fn new(name: &str) -> Self;
        fn open_tree()
    }
}
