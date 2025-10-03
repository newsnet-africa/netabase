use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm,
    kad::{self, EntryView, KBucketKey, Mode, NoKnownPeers, QueryResult, Quorum, RoutingUpdate},
};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

// Handler modules
pub mod add_address;
pub mod bootstrap;
pub mod fallback;
pub mod get_providers;
pub mod get_record;
pub mod mode;
pub mod protocol_names;
pub mod put_record;
pub mod put_record_to;
pub mod remove_address;
pub mod remove_peer;
pub mod remove_record;
pub mod set_mode;
pub mod start_providing;
pub mod stop_providing;

#[derive(Debug)]
pub enum Command<S: NetabaseSchema> {
    Kademlia(KademliaCommand<S>),
}

#[derive(Debug)]
pub enum KademliaCommand<S: NetabaseSchema> {
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

#[derive(Debug)]
pub enum LocalStoreCommand {}

pub fn handle_command_events<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    command: Command<S>,
) {
    match command {
        Command::Kademlia(kad_command) => {
            handle_kademlia_command(swarm, kad_command);
        }
    }
}

pub fn handle_kademlia_command<S: NetabaseSchema>(
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
        KademliaCommand::LocalStore(_local_store_command) => {
            // TODO: Implement local store command handlers when LocalStoreCommand is populated
            todo!("LocalStore command received - not yet implemented");
        }
    }
}
