#[cfg(feature = "native")]
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm,
    kad::{self, EntryView, KBucketKey, Mode, NoKnownPeers, QueryResult, Quorum, RoutingUpdate},
};
#[cfg(feature = "native")]
use netabase_store::traits::NetabaseSchema;
#[cfg(feature = "native")]
use tokio::sync::oneshot::Sender;

#[cfg(feature = "native")]
use crate::network::behaviour::NetabaseBehaviour;

// Handler modules - only available for native builds
#[cfg(feature = "native")]
pub mod add_address;
#[cfg(feature = "native")]
pub mod bootstrap;
#[cfg(feature = "native")]
pub mod fallback;
#[cfg(feature = "native")]
pub mod get_providers;
#[cfg(feature = "native")]
pub mod get_record;
#[cfg(feature = "native")]
pub mod mode;
#[cfg(feature = "native")]
pub mod protocol_names;
#[cfg(feature = "native")]
pub mod put_record;
#[cfg(feature = "native")]
pub mod put_record_to;
#[cfg(feature = "native")]
pub mod remove_address;
#[cfg(feature = "native")]
pub mod remove_peer;
#[cfg(feature = "native")]
pub mod remove_record;
#[cfg(feature = "native")]
pub mod set_mode;
#[cfg(feature = "native")]
pub mod start_providing;
#[cfg(feature = "native")]
pub mod stop_providing;

#[cfg(feature = "native")]
#[derive(Debug)]
pub(crate) enum Command<S: NetabaseSchema> {
    Kademlia(KademliaCommand<S>),
}

#[cfg(feature = "native")]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum KademliaCommand<S: NetabaseSchema> {
    AddAddress {
        peer: PeerId,
        address: Multiaddr,
        response_channel: Sender<RoutingUpdate>,
    },
    Bootstrap {
        response_channel: Sender<Result<QueryResult, NoKnownPeers>>,
    },
    GetProviders {
        key: S::Keys,
        response_channel: Sender<QueryResult>,
    },
    GetRecord {
        key: S::Keys,
        response_channel: Sender<QueryResult>,
    },
    Mode {
        response_channel: Sender<kad::Mode>,
    },
    ProtocolNames {
        response_channel: Sender<StreamProtocol>,
    },
    PutRecord {
        record: S,
        response_channel: Sender<Result<QueryResult, kad::store::Error>>,
    },
    PutRecordTo {
        record: S,
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
        key: S::Keys,
    },
    SetMode {
        mode: Option<Mode>,
    },
    StartProviding {
        key: S::Keys,
        response_channel: Sender<Result<QueryResult, kad::store::Error>>,
    },
    StopProviding {
        key: S::Keys,
    },

    LocalStore(LocalStoreCommand),
}

#[cfg(feature = "native")]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum LocalStoreCommand {}

#[cfg(feature = "native")]
pub(crate) fn handle_command_events<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    command: Command<S>,
) {
    match command {
        Command::Kademlia(kad_command) => {
            handle_kademlia_command(swarm, kad_command);
        }
    }
}

#[cfg(feature = "native")]
pub(crate) fn handle_kademlia_command<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    command: KademliaCommand<S>,
) {
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
    }
}
