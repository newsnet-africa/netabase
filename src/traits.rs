use bincode::{Decode, Encode};
use libp2p::kad::{Record, RecordKey};

use crate::errors::NetabaseError;

pub trait NetabaseModel: Encode + Decode<()> + Sized + Clone + Send + Sync {
    type Key: NetabaseModelKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchema:
    Encode
    + Decode<()>
    + Sized
    + TryInto<Record>
    + TryFrom<Record>
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
    + Send
    + Sync
{
    fn to_record(&self) -> Result<Record, NetabaseError>
    where
        <Self as TryInto<libp2p::kad::Record>>::Error: std::marker::Send,
        <Self as TryInto<libp2p::kad::Record>>::Error: std::marker::Sync,
        <Self as TryInto<libp2p::kad::Record>>::Error: 'static,
    {
        Ok(Record {
            key: RecordKey::new(&bincode::encode_to_vec(self, bincode::config::standard())?),
            value: bincode::encode_to_vec(self, bincode::config::standard())?,
            publisher: None,
            expires: None,
        })
    }

    fn from_record(record: Record) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&record.value, bincode::config::standard())?.0)
    }

    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

pub trait NetabaseModelKey: Encode + Decode<()> + Clone + Sized + Send + Sync + 'static {}

pub trait NetabaseKeys:
    Encode
    + Decode<()>
    + TryInto<RecordKey>
    + TryFrom<RecordKey>
    + TryInto<sled::IVec>
    + TryFrom<sled::IVec>
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

    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        Ok(sled::IVec::from(bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )?))
    }

    fn from_ivec(ivec: sled::IVec) -> Result<Self, NetabaseError> {
        Ok(bincode::decode_from_slice::<Self, _>(&ivec, bincode::config::standard())?.0)
    }
}

pub mod database_traits {
    pub trait NetabaseDatabase<Schema: strum::VariantNames> {
        fn new(name: &str) -> Self;
        // fn open_tree()
    }
}
