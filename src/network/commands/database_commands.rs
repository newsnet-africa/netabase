use libp2p::PeerId;
use libp2p::kad::{GetRecordOk, Quorum};

use crate::PutRecordOk;
use crate::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};

pub trait Database {
    async fn put<V: NetabaseSchema, I: ExactSizeIterator<Item = PeerId>>(
        &mut self,
        value: V,
        put_to: Option<I>,
        quorum: Quorum,
    ) -> anyhow::Result<PutRecordOk>;

    /// Get a record by its wrapped key type
    async fn get<K: NetabaseSchemaKey>(&mut self, key: K) -> anyhow::Result<GetRecordOk>;
}
