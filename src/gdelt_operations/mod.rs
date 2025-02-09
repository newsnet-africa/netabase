use gdelt_fetcher::models::gdelt::{DatabaseTableEntry, PrimaryKey, event::Event};
use libp2p::{
    Swarm,
    kad::{self, QueryId, Record},
    swarm::SwarmEvent,
};
use tokio::sync::mpsc;

use crate::config::behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent};

pub mod event;
pub mod mentions;

pub trait DatabaseOperation<T>
where
    T: DatabaseTableEntry,
{
    type UpdateEnum;
    async fn create(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        item: T,
    ) -> Result<QueryId, kad::store::Error>;
    async fn read(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        key: PrimaryKey,
    ) -> QueryId;
    async fn update(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        update: Self::UpdateEnum,
    );
    async fn delete(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        key: PrimaryKey,
    );
}
