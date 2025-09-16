use core::slice;
use std::{borrow::Cow, iter::Map};

use libp2p::{
    PeerId,
    kad::{ProviderRecord, Record, store::RecordStore},
};
use native_db::{Database, ToInput, transaction::query::PrimaryScanIterator};

use crate::NetabaseCatalog;

pub struct NativeDBStore<'a> {
    local_key: PeerId,
    database: Database<'a>,
}

impl<'a> RecordStore for NativeDBStore<'a> {
    // Not a fan of what is happening here, but converting the granular types seems like a pain
    type RecordsIter<'iter>
        = Map<slice::Iter<'iter, Record>, fn(&Record) -> Cow<'iter, Record>>
    where
        Self: 'iter;

    type ProvidedIter<'iter>
        = Map<slice::Iter<'iter, ProviderRecord>, fn(&ProviderRecord) -> Cow<'iter, ProviderRecord>>
    where
        Self: 'iter;

    fn get(&self, k: &libp2p::kad::RecordKey) -> Option<std::borrow::Cow<'_, libp2p::kad::Record>> {
        todo!()
    }

    fn put(&mut self, r: libp2p::kad::Record) -> libp2p::kad::store::Result<()> {
        todo!()
    }

    fn remove(&mut self, k: &libp2p::kad::RecordKey) {
        todo!()
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        todo!()
    }

    fn add_provider(
        &mut self,
        record: libp2p::kad::ProviderRecord,
    ) -> libp2p::kad::store::Result<()> {
        todo!()
    }

    fn providers(&self, key: &libp2p::kad::RecordKey) -> Vec<libp2p::kad::ProviderRecord> {
        todo!()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        todo!()
    }

    fn remove_provider(&mut self, k: &libp2p::kad::RecordKey, p: &PeerId) {
        todo!()
    }
}
