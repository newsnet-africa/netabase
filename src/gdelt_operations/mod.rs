use std::sync::Arc;

use gdelt_fetcher::models::gdelt::{
    DatabaseTableEntry, DatabaseTableEnum, PrimaryKey, event::Event,
};
use libp2p::{
    Multiaddr, PeerId, Swarm,
    kad::{self, PutRecordError, PutRecordOk, QueryId, Record},
    swarm::SwarmEvent,
};
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::config::behaviour::{NetEvent, NetabaseBehaviour};

#[derive(Clone, Debug)]
pub enum KadAction {
    AddAddress(PeerId, Multiaddr),
    Create(DatabaseTableEnum),
    Read(PrimaryKey),
    Update(PrimaryKey, DatabaseTableEnum),
}
