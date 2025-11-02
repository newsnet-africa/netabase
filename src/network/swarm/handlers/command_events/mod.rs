#[cfg(feature = "native")]
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm,
    kad::{
        self, EntryView, KBucketKey, Mode, NoKnownPeers, QueryResult, Quorum, RoutingUpdate,
        store::RecordStore,
    },
};
#[cfg(feature = "native")]
use netabase_store::traits::{convert::ToIVec, definition::{NetabaseDefinitionTrait, RecordStoreExt}};
#[cfg(feature = "native")]
use strum::IntoDiscriminant;
#[cfg(feature = "native")]
use tokio::sync::oneshot::Sender;

#[cfg(feature = "native")]
use crate::network::behaviour::NetabaseBehaviour;

// Handler modules - only available for native builds
#[cfg(feature = "native")]
pub(crate) mod add_address;
#[cfg(feature = "native")]
pub(crate) mod bootstrap;
#[cfg(feature = "native")]
pub(crate) mod fallback;
#[cfg(feature = "native")]
pub(crate) mod get_providers;
#[cfg(feature = "native")]
pub(crate) mod get_record;
#[cfg(feature = "native")]
pub(crate) mod mode;
#[cfg(feature = "native")]
pub(crate) mod protocol_names;
#[cfg(feature = "native")]
pub(crate) mod put_record;
#[cfg(feature = "native")]
pub(crate) mod put_record_to;
#[cfg(feature = "native")]
pub(crate) mod remove_address;
#[cfg(feature = "native")]
pub(crate) mod remove_peer;
#[cfg(feature = "native")]
pub(crate) mod remove_record;
#[cfg(feature = "native")]
pub(crate) mod set_mode;
#[cfg(feature = "native")]
pub(crate) mod start_providing;
#[cfg(feature = "native")]
pub(crate) mod stop_providing;

#[cfg(feature = "native")]
#[derive(Debug)]
pub(crate) enum Command<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>
where
    D: netabase_store::convert::ToIVec,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
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
    <D as IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
{
    Kademlia(KademliaCommand<D>),
}

#[cfg(feature = "native")]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum KademliaCommand<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>
where
    D: netabase_store::convert::ToIVec,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
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
    <D as IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
{
    AddAddress {
        peer: PeerId,
        address: Multiaddr,
        response_channel: Sender<RoutingUpdate>,
    },
    Bootstrap {
        response_channel: Sender<Result<QueryResult, NoKnownPeers>>,
    },
    GetProviders {
        key: D::Keys,
        response_channel: Sender<QueryResult>,
    },
    GetRecord {
        key: D::Keys,
        response_channel: Sender<QueryResult>,
    },
    Mode {
        response_channel: Sender<kad::Mode>,
    },
    ProtocolNames {
        response_channel: Sender<StreamProtocol>,
    },
    PutRecord {
        record: D,
        response_channel: Sender<Result<QueryResult, kad::store::Error>>,
    },
    PutRecordTo {
        record: D,
        peers: Vec<PeerId>,
        quorum: Quorum,
        response_channel: Sender<QueryResult>,
    },
    RemoveAddress {
        peer: PeerId,
        address: Multiaddr,
        response_channel: Sender<Option<EntryView<KBucketKey<PeerId>, kad::Addresses>>>,
    },
    RemovePeer {
        peer: PeerId,
        response_channel: Sender<Option<EntryView<KBucketKey<PeerId>, kad::Addresses>>>,
    },
    RemoveRecord {
        key: D::Keys,
    },
    SetMode {
        mode: Option<Mode>,
    },
    StartProviding {
        key: D::Keys,
        response_channel: Sender<Result<QueryResult, kad::store::Error>>,
    },
    StopProviding {
        key: D::Keys,
    },

    LocalStore(LocalStoreCommand<D>),
}

#[cfg(feature = "native")]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum LocalStoreCommand<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>
where
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
    QueryRecords {
        limit: Option<usize>,
        response_channel: Sender<Result<Vec<D>, String>>,
    },
}

#[cfg(feature = "native")]
pub(crate) fn handle_command_events<D>(swarm: &mut Swarm<NetabaseBehaviour<D>>, command: Command<D>)
where
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
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    match command {
        Command::Kademlia(kad_command) => {
            handle_kademlia_command(swarm, kad_command);
        }
    }
}

#[cfg(feature = "native")]
pub(crate) fn handle_kademlia_command<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    command: KademliaCommand<D>,
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
    match command {
        KademliaCommand::AddAddress {
            peer,
            address,
            response_channel,
        } => {
            add_address::handle_add_address(swarm, peer, address, response_channel);
        }
        KademliaCommand::Bootstrap { response_channel } => {
            bootstrap::handle_bootstrap(swarm, response_channel);
        }
        KademliaCommand::GetProviders {
            key,
            response_channel,
        } => {
            get_providers::handle_get_providers(swarm, key, response_channel);
        }
        KademliaCommand::GetRecord {
            key,
            response_channel,
        } => {
            get_record::handle_get_record(swarm, key, response_channel);
        }
        KademliaCommand::Mode { response_channel } => {
            mode::handle_mode(swarm, response_channel);
        }
        KademliaCommand::ProtocolNames { response_channel } => {
            protocol_names::handle_protocol_names(swarm, response_channel);
        }
        KademliaCommand::PutRecord {
            record,
            response_channel,
        } => {
            put_record::handle_put_record(swarm, record, response_channel);
        }
        KademliaCommand::PutRecordTo {
            record,
            peers,
            quorum,
            response_channel,
        } => {
            put_record_to::handle_put_record_to(swarm, record, peers, quorum, response_channel);
        }
        KademliaCommand::RemoveAddress {
            peer,
            address,
            response_channel,
        } => {
            remove_address::handle_remove_address(swarm, peer, address, response_channel);
        }
        KademliaCommand::RemovePeer {
            peer,
            response_channel,
        } => {
            remove_peer::handle_remove_peer(swarm, peer, response_channel);
        }
        KademliaCommand::RemoveRecord { key } => {
            remove_record::handle_remove_record(swarm, key);
        }
        KademliaCommand::SetMode { mode } => {
            set_mode::handle_set_mode(swarm, mode);
        }
        KademliaCommand::StartProviding {
            key,
            response_channel,
        } => {
            start_providing::handle_start_providing(swarm, key, response_channel);
        }
        KademliaCommand::StopProviding { key } => {
            stop_providing::handle_stop_providing(swarm, key);
        }
        KademliaCommand::LocalStore(local_store_command) => {
            handle_local_store_command(swarm, local_store_command);
        }
    }
}

#[cfg(feature = "native")]
pub(crate) fn handle_local_store_command<
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static + ToIVec,
>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    command: LocalStoreCommand<D>,
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
    match command {
        LocalStoreCommand::QueryRecords {
            limit,
            response_channel,
        } => {
            // Access the Kademlia store directly
            let store = swarm.behaviour_mut().kad.store_mut();

            // Collect records from the store
            let mut records = Vec::new();
            let mut count = 0;

            for record in store.records() {
                if let Some(max_count) = limit
                    && count >= max_count
                {
                    break;
                }

                // Try to decode the record value
                // Convert Vec<u8> to IVec - use owned conversion
                let ivec = record.value.as_slice().to_vec().into();
                match D::from_ivec(&ivec) {
                    Ok(definition) => {
                        records.push(definition);
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to decode record: {}", e);
                    }
                }
            }

            let _ = response_channel.send(Ok(records));
        }
    }
}
