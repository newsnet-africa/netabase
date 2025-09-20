use libp2p::kad::store::RecordStore;
use native_db::Database;

pub struct NativeDBStore<'db, T> {
    database: Database<'db>,
    _phantom: T,
}

// impl RecordStore for NativeDBStore {
//     type RecordsIter<'a>
//     where
//         Self: 'a;

//     type ProvidedIter<'a>
//     where
//         Self: 'a;

//     fn get(&self, k: &libp2p::kad::RecordKey) -> Option<std::borrow::Cow<'_, libp2p::kad::Record>> {
//         todo!()
//     }

//     fn put(&mut self, r: libp2p::kad::Record) -> libp2p::kad::store::Result<()> {
//         todo!()
//     }

//     fn remove(&mut self, k: &libp2p::kad::RecordKey) {
//         todo!()
//     }

//     fn records(&self) -> Self::RecordsIter<'_> {
//         todo!()
//     }

//     fn add_provider(
//         &mut self,
//         record: libp2p::kad::ProviderRecord,
//     ) -> libp2p::kad::store::Result<()> {
//         todo!()
//     }

//     fn providers(&self, key: &libp2p::kad::RecordKey) -> Vec<libp2p::kad::ProviderRecord> {
//         todo!()
//     }

//     fn provided(&self) -> Self::ProvidedIter<'_> {
//         todo!()
//     }

//     fn remove_provider(&mut self, k: &libp2p::kad::RecordKey, p: &libp2p::PeerId) {
//         todo!()
//     }
// }
