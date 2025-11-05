use libp2p::PeerId;
use log::{debug, info, warn, error};
use libp2p::swarm::ConnectionId;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

pub fn handle_dialing<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    peer_id: Option<PeerId>,
    connection_id: ConnectionId,
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    // TODO: Implement dialing handling
    if let Some(peer) = peer_id {
        debug!(
            "Dialing peer: {:?}, connection_id: {:?}",
            peer, connection_id
        );
    } else {
        debug!("Dialing unknown peer, connection_id: {:?}", connection_id);
    }
}
