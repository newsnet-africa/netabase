use libp2p::{
    Multiaddr, PeerId, StreamProtocol,
    kad::{
        self, EntryView, KBucketKey, Mode, NoKnownPeers, QueryId, QueryResult, Quorum, RecordKey,
        RoutingUpdate,
    },
};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

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
