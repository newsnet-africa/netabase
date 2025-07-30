use libp2p::kad::Quorum;

use crate::PutRecordOk;
use crate::netabase_trait::NetabaseSchema;

pub(crate) trait Database {
    async fn put<V: NetabaseSchema>(
        &mut self,
        value: V,
        quorum: Quorum,
    ) -> anyhow::Result<PutRecordOk>;
}
