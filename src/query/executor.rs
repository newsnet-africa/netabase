use serde::{Serialize, Deserialize, de::DeserializeOwned};
use netabase_store::{
    prelude::{NetabaseDefinition, NetabaseModel},
    traits::registery::models::{
        NetabaseModelKeys,
        keys::blob::NetabaseModelBlobKey,
        model::RedbNetbaseModel,
    },
    databases::redb::transaction::{
        RedbTransaction,
        RedbModelCrud,
        options::CrudOptions,
    },
};
use strum::IntoDiscriminant;

use super::messages::{DatabaseQuery, DatabaseQueryResult};
use super::traits::{QueryExecutor, QueryError, QueryResult};

impl<'a, 'db, D, M> QueryExecutor<D, M, DatabaseQuery<D, M>> for RedbTransaction<'db, D>
where
    D: netabase_store::traits::registery::definition::redb_definition::RedbDefinition + Clone + 'static,
    M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + netabase_store::prelude::NetabaseModel<D> + redb::Key + Clone + 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    
    // Key bounds required by RedbTransaction::prepare_model and RedbModelCrud
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd + redb::Key + 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd + redb::Key + 'static,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
    
    // Additional bounds for RedbModelCrud
    D::SubscriptionKeys: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    for<'b> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'b>>,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'b>>,
    for<'b> M::TableV: redb::Value<SelfType<'b> = M>,
    for<'b> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'b>>,
{
    type Output = DatabaseQueryResult<M>;

    fn execute(&self, query: DatabaseQuery<D, M>) -> QueryResult<Self::Output> {
        match query {
            DatabaseQuery::Get { key } => {
                let tables = self.prepare_model::<M>()
                    .map_err(|e: netabase_store::errors::NetabaseError| QueryError::Storage(e.to_string()))?;
                
                let res = M::read_default(&key, &tables)
                    .map_err(|e: netabase_store::errors::NetabaseError| QueryError::Storage(e.to_string()))?;
                Ok(DatabaseQueryResult::Record(res))
            },
            DatabaseQuery::Exists { key } => {
                let tables = self.prepare_model::<M>()
                    .map_err(|e: netabase_store::errors::NetabaseError| QueryError::Storage(e.to_string()))?;

                let exists = M::read_default(&key, &tables)
                    .map_err(|e: netabase_store::errors::NetabaseError| QueryError::Storage(e.to_string()))?
                    .is_some();
                Ok(DatabaseQueryResult::Exists(exists))
            },
            DatabaseQuery::Range { start, end, limit } => {
                let range = if let (Some(s), Some(e)) = (start, end) {
                    s..=e
                } else {
                    return Err(QueryError::Validation(super::validation::ValidationError::OutOfScope { 
                        required_key: "Open-ended ranges not yet supported via this API".into() 
                    }));
                };

                let tables = self.prepare_model::<M>()
                    .map_err(|e: netabase_store::errors::NetabaseError| QueryError::Storage(e.to_string()))?;

                let config = CrudOptions::default();
                
                let results_guards = M::list_range(&tables, range, config)
                    .map_err(|e: netabase_store::errors::NetabaseError| QueryError::Storage(e.to_string()))?;
                
                let mut results = Vec::with_capacity(results_guards.len());
                for guard in results_guards {
                    results.push(guard.value());
                }

                let limited = if let Some(l) = limit {
                    results.into_iter().take(l as usize).collect()
                } else {
                    results
                };

                Ok(DatabaseQueryResult::Range(limited))
            },
            DatabaseQuery::GetBlob { key: _, field_index: _ } => {
                Err(QueryError::Storage("Blob fetching not yet implemented in executor".into()))
            }
        }
    }
}
