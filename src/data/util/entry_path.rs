use netabase_store::{
    libp2p::PeerId,
    prelude::NetabaseDefinition,
};

use crate::data::store::model::NetworkModel;


pub struct EntryPath<D: NetabaseDefinition, M: NetworkModel<D>> (pub Vec<EntryPathNode<D, M>>)
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static, <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static
;
pub enum EntryPathNode<D: NetabaseDefinition, M: NetworkModel<D>>
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static, <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static
{
    Owner(PeerId),
    Directory(M::PathNodes),
    Entry(EntryKey<D, M>),
}

pub enum EntryKey<D: NetabaseDefinition, M: NetworkModel<D>>
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static, <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static
{
    Root(M::EntryKey),
    Shadow(M::EntryKey),
}
