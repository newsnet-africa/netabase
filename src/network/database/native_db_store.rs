use libp2p::{kad::store::RecordStore, PeerId};
use native_db::{transaction::query::PrimaryScanIterator, Database};

pub struct NativeDBStore<'a> {
    local_key: PeerId,
    database: Database<'a>,
}

impl<'a, Registry> RecordStore for NativeDBStore<'a> {
    type RecordsIter<'a>
    where
        Self: 'a = PrimaryScanIterator<'a, >

    type ProvidedIter<'a>
    where
        Self: 'a;

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

    fn add_provider(&mut self, record: libp2p::kad::ProviderRecord) -> libp2p::kad::store::Result<()> {
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
