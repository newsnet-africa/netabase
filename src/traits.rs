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
    sled::IVec: TryFrom<Self>,
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

pub trait NetabaseDiscriminants: Into<&'static str> {}

pub mod database_traits {
    use libp2p::kad::{Record, RecordKey};

    use crate::{
        errors::NetabaseError,
        traits::{NetabaseModel, NetabaseModelKey},
    };

    pub trait NetabaseSledDatabase {
        fn new(name: &str) -> Self;
        fn db(&self) -> &sled::Db;
        fn open_tree<K: NetabaseModelKey, V: NetabaseModel, T: NetabaseSledTree<K, V>>(
            &self,
            name: &'static str,
        ) -> Result<T, NetabaseError>
        where
            sled::IVec: std::convert::TryFrom<V>,
            libp2p::kad::RecordKey: std::convert::TryFrom<K>,
            libp2p::kad::Record: std::convert::TryFrom<V>,
            libp2p::kad::RecordKey: std::convert::TryFrom<<V as NetabaseModel>::Key>,
        {
            match self.db().open_tree(name) {
                Ok(k) => T::try_from(k).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                }),
                Err(_e) => Err(NetabaseError::Database),
            }
        }
    }
    pub trait NetabaseSledTree<K, V>: TryFrom<sled::Tree>
    where
        RecordKey: TryFrom<K>,
        Record: TryFrom<V>,
        RecordKey: TryFrom<<V as NetabaseModel>::Key>,
        sled::IVec: std::convert::TryFrom<V>,
        K: NetabaseModelKey,
        V: NetabaseModel,
    {
        fn tree(&self) -> &sled::Tree;
        fn insert(&self, key: K, value: V) -> Result<Option<V>, NetabaseError>
        where
            sled::IVec: TryFrom<K>,
            sled::IVec: TryFrom<V>,
            V: TryFrom<sled::IVec>,
        {
            let key_ivec: sled::IVec = key.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;
            let value_ivec: sled::IVec = value.try_into().map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })?;

            match self.tree().insert(key_ivec, value_ivec)? {
                Some(old_ivec) => {
                    let old_value = V::try_from(old_ivec).map_err(|_| {
                        NetabaseError::Conversion(
                            crate::errors::conversion::ConversionError::TraitConversion,
                        )
                    })?;
                    Ok(Some(old_value))
                }
                None => Ok(None),
            }
        }
    }
}
