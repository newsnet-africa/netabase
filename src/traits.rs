use bincode::{Decode, Encode};
use libp2p::kad::{Record, RecordKey};

use crate::errors::NetabaseError;

pub trait NetabaseModel:
    Encode + Decode<()> + Sized + TryFrom<Record> + TryInto<Record> + Send + Sync
where
    Self::Key: NetabaseModelKey,
    Record: TryFrom<Self> + TryInto<Self>,
    libp2p::kad::RecordKey: TryFrom<<Self as NetabaseModel>::Key>,
{
    type Key: NetabaseModelKey;
    fn key(&self) -> Self::Key;

    fn to_record(&self) -> Result<Record, NetabaseError>
    where
        <Self::Key as TryInto<RecordKey>>::Error: std::fmt::Debug,
        Record: std::convert::TryFrom<<Self as NetabaseModel>::Key>,
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

pub trait NetabaseModelKey:
    Encode
    + AsRef<[u8]>
    + Decode<()>
    + TryFrom<RecordKey>
    + TryInto<RecordKey>
    + Clone
    + Sized
    + Send
    + Sync
    + 'static
where
    libp2p::kad::RecordKey: TryFrom<Self> + TryInto<Self>,
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

pub trait NetabaseSchema: Send + Sync + Encode + Decode<()> + TryFrom<sled::IVec>
where
    sled::IVec: TryFrom<Self>,
{
    type Discriminants: NetabaseDiscriminants;
    type Keys: NetabaseKeys;
}
pub trait NetabaseKeys {
    type Discriminants: NetabaseDiscriminants;
}

pub trait NetabaseDiscriminants: Into<&'static str> {}

pub mod database_traits {
    use libp2p::kad::{Record, RecordKey};

    use crate::{
        errors::NetabaseError,
        traits::{NetabaseDiscriminants, NetabaseModel, NetabaseModelKey, NetabaseSchema},
    };

    pub trait NetabaseSledDatabase<Schema: NetabaseSchema>
    where
        sled::IVec: TryFrom<Schema>,
    {
        fn new(name: &str) -> Self;
        fn db(&self) -> &sled::Db;
        fn open_tree<K: NetabaseModelKey, V: NetabaseModel, T: NetabaseSledTree<K, V>>(
            &self,
            name: Schema::Discriminants,
        ) -> Result<T, NetabaseError>
        where
            sled::IVec: std::convert::TryFrom<V>,
            libp2p::kad::RecordKey: std::convert::TryFrom<K>,
            libp2p::kad::Record: std::convert::TryFrom<V>,
            libp2p::kad::RecordKey: std::convert::TryFrom<<V as NetabaseModel>::Key>,
        {
            match self.db().open_tree(name.into()) {
                Ok(k) => T::try_from(k).map_err(|_| {
                    NetabaseError::Conversion(
                        crate::errors::conversion::ConversionError::TraitConversion,
                    )
                }),
                Err(e) => Err(NetabaseError::Database),
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
        fn insert(&self, key: K, value: V) -> sled::Result<Option<V>> {
            self.tree().insert(key, value)
        }
    }
}
