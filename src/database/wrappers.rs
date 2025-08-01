use std::collections::HashSet;

use anyhow::Error;
use bincode::{Decode, Encode, config::Configuration};
use libp2p::{
    Multiaddr, PeerId,
    kad::{K_VALUE, ProviderRecord, Record, RecordKey},
};

use sled::IVec;
use smallvec::SmallVec;

#[derive(Encode, Decode)]
pub(crate) struct RecordWrapper {
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>,
    pub(crate) publisher: Option<Vec<u8>>,
}

impl From<Record> for RecordWrapper {
    fn from(value: Record) -> Self {
        Self {
            key: value.key.to_vec(),
            value: value.value,
            publisher: value.publisher.map(|p| p.to_bytes()),
        }
    }
}

impl From<RecordWrapper> for Record {
    fn from(value: RecordWrapper) -> Self {
        Self {
            key: RecordKey::new(&value.key),
            value: value.value,
            publisher: value
                .publisher
                .and_then(|bytes| PeerId::from_bytes(&bytes).ok()),
            expires: None,
        }
    }
}

impl TryFrom<&IVec> for RecordWrapper {
    type Error = anyhow::Error;

    fn try_from(value: &IVec) -> Result<Self, Error> {
        Ok(bincode::decode_from_slice(&value.to_vec(), bincode::config::standard())?.0)
    }
}

impl TryFrom<RecordWrapper> for IVec {
    type Error = anyhow::Error;

    fn try_from(value: RecordWrapper) -> Result<Self, Error> {
        Ok(bincode::encode_to_vec(value, bincode::config::standard())?.into())
    }
}

pub fn try_record_to_ivec(r: Record) -> Result<IVec, bincode::error::EncodeError> {
    bincode::encode_to_vec(RecordWrapper::from(r), bincode::config::standard())
        .map(|vec| IVec::from(vec))
}
pub fn try_ivec_to_record(ivec: IVec) -> Result<Record, bincode::error::DecodeError> {
    bincode::decode_from_slice::<RecordWrapper, Configuration>(
        &ivec.to_vec(),
        bincode::config::standard(),
    )
    .map(|(r, _)| Record::from(r))
}

#[derive(Decode, Encode)]
pub struct ProviderRecordWrapper {
    pub(crate) key: Vec<u8>,
    pub(crate) provider: Vec<u8>,
    pub(crate) addresses: Vec<Vec<u8>>,
}

impl From<ProviderRecord> for ProviderRecordWrapper {
    fn from(value: ProviderRecord) -> Self {
        Self {
            key: value.key.to_vec(),
            provider: value.provider.to_bytes(),
            addresses: value
                .addresses
                .into_iter()
                .map(|addr| addr.to_vec())
                .collect(),
        }
    }
}

impl From<ProviderRecordWrapper> for ProviderRecord {
    fn from(value: ProviderRecordWrapper) -> Self {
        Self {
            key: RecordKey::new(&value.key),
            provider: PeerId::from_bytes(&value.provider).unwrap_or_else(|_| PeerId::random()),
            expires: None,
            addresses: value
                .addresses
                .into_iter()
                .filter_map(|bytes| Multiaddr::try_from(bytes).ok())
                .collect(),
        }
    }
}

impl From<&ProviderRecordWrapper> for ProviderRecord {
    fn from(value: &ProviderRecordWrapper) -> Self {
        Self {
            key: RecordKey::new(&value.key),
            provider: PeerId::from_bytes(&value.provider).unwrap_or_else(|_| PeerId::random()),
            expires: None,
            addresses: value
                .addresses
                .iter()
                .filter_map(|bytes| Multiaddr::try_from(bytes.clone()).ok())
                .collect(),
        }
    }
}

impl TryFrom<&IVec> for ProviderRecordWrapper {
    type Error = anyhow::Error;

    fn try_from(value: &IVec) -> Result<Self, Self::Error> {
        Ok(bincode::decode_from_slice(&value.to_vec(), bincode::config::standard())?.0)
    }
}

impl TryFrom<ProviderRecordWrapper> for IVec {
    type Error = anyhow::Error;

    fn try_from(value: ProviderRecordWrapper) -> Result<Self, Self::Error> {
        Ok(bincode::encode_to_vec(value, bincode::config::standard())?.into())
    }
}

pub fn try_provider_record_to_ivec(r: ProviderRecord) -> Result<IVec, bincode::error::EncodeError> {
    bincode::encode_to_vec(ProviderRecordWrapper::from(r), bincode::config::standard())
        .map(|vec| IVec::from(vec))
}
pub fn try_ivec_to_provider_record(
    ivec: IVec,
) -> Result<ProviderRecord, bincode::error::DecodeError> {
    bincode::decode_from_slice::<ProviderRecordWrapper, Configuration>(
        &ivec.to_vec(),
        bincode::config::standard(),
    )
    .map(|(r, _)| ProviderRecord::from(r))
}

pub fn try_ivec_to_providers_smallvec(
    ivec: IVec,
) -> Result<SmallVec<[ProviderRecord; K_VALUE.get()]>, bincode::error::DecodeError> {
    bincode::decode_from_slice::<SmallVec<[ProviderRecordWrapper; K_VALUE.get()]>, Configuration>(
        &ivec.to_vec(),
        bincode::config::standard(),
    )
    .map(|(v, _)| {
        v.iter()
            .map(|wrapper| ProviderRecord::from(wrapper))
            .collect()
    })
} // TODO: I think that serialization is possible with smallvec but it lowkey feels like its shit I have a feeling.

pub fn try_providers_smallvec_to_ivec(
    providers: SmallVec<[ProviderRecord; K_VALUE.get()]>,
) -> Result<IVec, bincode::error::EncodeError> {
    let wrapped: SmallVec<[ProviderRecordWrapper; K_VALUE.get()]> = providers
        .into_iter()
        .map(ProviderRecordWrapper::from)
        .collect();
    bincode::encode_to_vec(wrapped, bincode::config::standard()).map(|vec| IVec::from(vec))
}

pub fn try_ivec_to_provided_hashset(
    ivec: IVec,
) -> Result<HashSet<ProviderRecord>, bincode::error::DecodeError> {
    bincode::decode_from_slice::<SmallVec<[ProviderRecordWrapper; K_VALUE.get()]>, Configuration>(
        &ivec.to_vec(),
        bincode::config::standard(),
    )
    .map(|(v, _)| {
        v.iter()
            .map(|wrapper| ProviderRecord::from(wrapper))
            .collect()
    })
}

pub fn try_provided_hashset_to_ivec(
    provided: &HashSet<ProviderRecord>,
) -> Result<IVec, bincode::error::EncodeError> {
    let wrapped: SmallVec<[ProviderRecordWrapper; K_VALUE.get()]> = provided
        .iter()
        .cloned()
        .map(ProviderRecordWrapper::from)
        .collect();
    bincode::encode_to_vec(wrapped, bincode::config::standard()).map(|vec| IVec::from(vec))
}
