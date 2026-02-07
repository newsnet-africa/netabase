use netabase_store::prelude::{NetabaseDefinition, NetabaseModel};

pub trait NetworkModel<D: NetabaseDefinition>: NetabaseModel<D>
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<Self as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, Self>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<Self as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, Self>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<Self as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, Self>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static
{
    type EntryKey;
    type PathNodes = String;
}
