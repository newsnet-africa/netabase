use std::borrow::Cow;

use libp2p::{
    Multiaddr, PeerId,
    kad::{ProviderRecord, Record, RecordKey},
};
use serde::{Deserialize, Serialize};
use sled::IVec;

#[derive(Serialize, Deserialize)]
pub struct RecordWrapper {
    pub key: RecordKey,
    pub value: Vec<u8>,
    pub publisher: Option<PeerId>,
    // pub expires: Option<Instant>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRecordWrapper {
    pub key: RecordKey,
    pub provider: PeerId,
    // pub expires: Option<Instant>,
    pub addresses: Vec<Multiaddr>,
}

impl RecordWrapper {
    pub fn record_to_ivec(record: Record) -> IVec {
        IVec::from(RecordWrapper::from(record))
    }

    pub fn ivec_to_record(ivec: IVec) -> Record {
        Record::from(RecordWrapper::from(ivec))
    }
}
impl ProviderRecordWrapper {
    pub fn record_to_ivec(provider_record: ProviderRecord) -> IVec {
        IVec::from(ProviderRecordWrapper::from(provider_record))
    }

    pub fn ivec_to_record(ivec: IVec) -> ProviderRecord {
        ProviderRecord::from(ProviderRecordWrapper::from(ivec))
    }
}

impl From<Record> for RecordWrapper {
    fn from(value: Record) -> Self {
        Self {
            key: value.key,
            value: value.value,
            publisher: value.publisher,
        }
    }
}

impl From<ProviderRecordWrapper> for ProviderRecord {
    fn from(value: ProviderRecordWrapper) -> Self {
        Self {
            key: value.key,
            provider: value.provider,
            addresses: value.addresses,
            expires: None,
        }
    }
}
impl From<&ProviderRecordWrapper> for ProviderRecord {
    fn from(value: &ProviderRecordWrapper) -> Self {
        Self {
            key: value.key.clone(),
            provider: value.provider.clone(),
            addresses: value.addresses.clone(),
            expires: None,
        }
    }
}

impl From<RecordWrapper> for Record {
    fn from(value: RecordWrapper) -> Self {
        Self {
            key: value.key,
            value: value.value,
            publisher: value.publisher,
            expires: None,
        }
    }
}

impl From<ProviderRecord> for ProviderRecordWrapper {
    fn from(value: ProviderRecord) -> Self {
        Self {
            key: value.key,
            provider: value.provider,
            addresses: value.addresses,
        }
    }
}

impl From<IVec> for RecordWrapper {
    fn from(value: IVec) -> Self {
        bincode::deserialize::<Self>(&value.to_vec()).expect("Cannot deserialise record")
    }
}

impl From<RecordWrapper> for IVec {
    fn from(value: RecordWrapper) -> Self {
        IVec::from(bincode::serialize(&value).expect("Could not serialise value"))
    }
}

impl<'a> From<RecordWrapper> for Cow<'a, Record> {
    fn from(value: RecordWrapper) -> Self {
        let record = Record::from(value);
        Cow::Owned(record)
    }
}
impl From<IVec> for ProviderRecordWrapper {
    fn from(value: IVec) -> Self {
        bincode::deserialize::<Self>(&value.to_vec()).expect("Cannot deserialise record")
    }
}

impl From<ProviderRecordWrapper> for IVec {
    fn from(value: ProviderRecordWrapper) -> Self {
        IVec::from(bincode::serialize(&value).expect("Could not serialise value"))
    }
}

impl<'a> From<ProviderRecordWrapper> for Cow<'a, ProviderRecord> {
    fn from(value: ProviderRecordWrapper) -> Self {
        let record = ProviderRecord::from(value);
        Cow::Owned(record)
    }
}
