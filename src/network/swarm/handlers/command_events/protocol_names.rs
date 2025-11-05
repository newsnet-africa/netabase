use libp2p::{StreamProtocol, Swarm};
use log::debug;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_protocol_names<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    response_channel: Sender<StreamProtocol>,
)where
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send, {
    debug!("ProtocolNames command received");

    // Call the libp2p Kademlia API
    let protocol_names = swarm.behaviour().kad.protocol_names();

    // Send the first protocol name (Kademlia typically has one main protocol)
    if let Some(protocol) = protocol_names.first() {
        if let Err(_) = response_channel.send(protocol.clone()) {
            debug!("Failed to send ProtocolNames response - receiver dropped");
        }
    } else {
        debug!("No protocol names available");
    }
}
