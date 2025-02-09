use gdelt_fetcher::models::gdelt::{PrimaryKey, event::Event};
use libp2p::{
    Swarm,
    kad::{self, Record, RecordKey},
    swarm::SwarmEvent,
};
use tokio::sync::mpsc::{self, Receiver};

use crate::config::{
    behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent},
    database::LocalDatabase,
};

use super::DatabaseOperation;

pub enum EventUpdate {
    AddMention {
        event_key: PrimaryKey,
        mention_key: PrimaryKey,
    },
    RemoveMention {
        event_key: PrimaryKey,
        mention_key: PrimaryKey,
    },
}

impl DatabaseOperation<Event> for LocalDatabase {
    type UpdateEnum = EventUpdate;
    async fn create(
        rx: Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        item: Event,
    ) -> std::result::Result<kad::QueryId, kad::store::Error> {
        let res = swarm
            .behaviour_mut()
            .event_kad
            .put_record(Record::from(item), libp2p::kad::Quorum::Majority);

        //     if let kad::Event::OutboundQueryProgressed { id, result, stats, step } = rx.recv()

        res
    }

    async fn read(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        key: gdelt_fetcher::models::gdelt::PrimaryKey,
    ) -> kad::QueryId {
        swarm
            .behaviour_mut()
            .event_kad
            .get_record(RecordKey::from(key))
    }
    async fn update(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        operation: Self::UpdateEnum,
    ) {
        match operation {
            EventUpdate::AddMention {
                event_key,
                mention_key,
            } => {
                let mut old_event = swarm
                    .behaviour_mut()
                    .event_kad
                    .get_record(RecordKey::from(event_key));
            }
            EventUpdate::RemoveMention {
                event_key,
                mention_key,
            } => todo!(),
        }
    }

    async fn delete(
        rx: mpsc::Receiver<SwarmEvent<NetabaseBehaviourEvent>>,
        swarm: &mut Swarm<NetabaseBehaviour>,
        key: gdelt_fetcher::models::gdelt::PrimaryKey,
    ) {
        todo!()
    }
}
