use libp2p::{Multiaddr, core::transport::ListenerId};
use log::debug;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

pub fn handle_expired_listen_addr<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    listener_id: ListenerId,
    address: Multiaddr,
) where
    D: netabase_store::convert::ToIVec,
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,{
    // TODO: Implement expired listen address handling
    debug!(
        "Listen address expired: listener_id: {:?}, address: {:?}",
        listener_id, address
    );
}
