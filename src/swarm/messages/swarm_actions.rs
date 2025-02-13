use gdelt_fetcher::models::gdelt::PrimaryKey;
use libp2p::{
    Multiaddr, PeerId,
    core::transport::{DialOpts, ListenerId},
    kad::{Mode, Quorum, Record, RecordKey},
    swarm::ConnectionId,
};

pub enum SwarmAction {
    KadAction(KadBehaviourMethods),
    MdnsAction(MdnsBehaviourMethods),
    Ping,
    Test,
}

pub enum KadBehaviourMethods {
    AddAddress(PeerId, Multiaddr),
    BOOTSTRAP,
    GetProviders(RecordKey),
    GetRecord(RecordKey),
    PutRecord(Record, Quorum),
    PutRecordTo(Record, Vec<PeerId>, Quorum),
    RemoveAddress(PeerId, Multiaddr),
    RemovePeer(PeerId),
    SetMode(Option<Mode>),
    StartProviding(RecordKey),
    StopProviding(RecordKey),
}

pub enum MdnsBehaviourMethods {
    DiscoveredNodes,
}

pub enum SwarmMethods {
    AddExternalAddress(Multiaddr),
    AddPeerAddress(PeerId, Multiaddr),
    CloseConnection(ConnectionId),
    DIALPeer(PeerId),
    DIALAddress(Multiaddr),
    DisconnectPeerId(PeerId),
    ListenOn(Multiaddr),
    RemoveExternalAddress(Multiaddr),
    RemoveListener(ListenerId),
}
