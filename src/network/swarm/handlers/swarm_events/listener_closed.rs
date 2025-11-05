use libp2p::{Multiaddr, core::transport::ListenerId};
use log::debug;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

pub fn handle_listener_closed<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    listener_id: ListenerId,
    addresses: Vec<Multiaddr>,
    reason: Result<(), std::io::Error>,
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
    // TODO: Implement listener closed handling
    debug!(
        "Listener closed: listener_id: {:?}, addresses: {:?}",
        listener_id, &addresses
    );

    match reason {
        Ok(()) => {
            debug!("Listener closed gracefully");
        }
        Err(error) => {
            debug!("Listener closed due to error: {:?}", error);
        }
    }
}
