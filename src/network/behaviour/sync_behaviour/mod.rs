use netabase_store::definition::NetabaseDefinitionTrait;

pub mod paxakos;

pub struct SyncBehaviour<D>
where
    D: NetabaseDefinitionTrait,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
{
    _marker: std::marker::PhantomData<D>,
}
