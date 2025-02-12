use libp2p::{
    Swarm,
    kad::{self, GetRecordOk, PeerRecord, Record, RecordKey},
};
use tokio::sync::broadcast;

use crate::{config::behaviour::NetabaseBehaviour, gdelt_operations::KadAction};

#[derive(Clone, Debug)]
pub enum SwarmAction {
    Kad(KadAction),
}

impl SwarmAction {
    pub async fn execute(
        &self,
        swarm: &mut Swarm<NetabaseBehaviour>,
        rx: &mut broadcast::Receiver<kad::Event>,
    ) {
        println!("Executing {self:?}");
        match self {
            SwarmAction::Kad(kad_action) => {
                match kad_action {
                    KadAction::AddAddress(peer_id, multiaddr) => {
                        swarm
                            .behaviour_mut()
                            .event_kad
                            .add_address(peer_id, multiaddr.clone());
                        swarm
                            .behaviour_mut()
                            .mentions_kad
                            .add_address(peer_id, multiaddr.clone());
                        swarm
                            .behaviour_mut()
                            .gkg_kad
                            .add_address(peer_id, multiaddr.clone());
                    }
                    KadAction::Create(database_table_enum) => {
                        let record = Record::from(database_table_enum);
                        match database_table_enum {
                            gdelt_fetcher::models::gdelt::DatabaseTableEnum::Mentions(mentions) => {
                                swarm.behaviour_mut().mentions_kad.put_record(record, kad::Quorum::One).expect("Failure");
                            }
                            gdelt_fetcher::models::gdelt::DatabaseTableEnum::GlobalKnowledgeGraph(global_knowledge_graph) => {
                                swarm.behaviour_mut().gkg_kad.put_record(record, kad::Quorum::One).expect("Failure");
                            }
                            gdelt_fetcher::models::gdelt::DatabaseTableEnum::Event(event) => {
                                swarm.behaviour_mut().event_kad.put_record(record, kad::Quorum::One).expect("Failure");
                            }
                        }
                    }
                    KadAction::Read(primary_key) => {
                        let key = RecordKey::from(primary_key);
                        match primary_key {
                            gdelt_fetcher::models::gdelt::PrimaryKey::EventKey(_) => {
                                swarm.behaviour_mut().event_kad.get_record(key);
                            }
                            gdelt_fetcher::models::gdelt::PrimaryKey::MentionKey(_) => {
                                swarm.behaviour_mut().mentions_kad.get_record(key);
                            }
                            gdelt_fetcher::models::gdelt::PrimaryKey::GKGKey(_) => {
                                swarm.behaviour_mut().gkg_kad.get_record(key);
                            }
                        }
                    }
                    // TODO: This implementation blocks for sime reason (might be the loop), avoid using update but i think the create item already updates in place.
                    KadAction::Update(primary_key, database_table_enum) => {
                        println!("Match update...\n Heres Key: {primary_key:?}");
                        let key = RecordKey::from(primary_key);
                        match primary_key {
                            gdelt_fetcher::models::gdelt::PrimaryKey::EventKey(_) => {
                                println!("GETTING RECORD BEFORE LOOP");
                                let item = loop {
                                    let q_id =
                                        swarm.behaviour_mut().event_kad.get_record(key.clone());
                                    println!("Waiting for Out.. Heres the query ID: {q_id:?}");
                                    let rec = rx.recv().await;
                                    println!("RECEIVED: {:?}", rec);
                                    if let Ok(kad::Event::OutboundQueryProgressed {
                                        id,
                                        result,
                                        stats,
                                        step,
                                    }) = rec
                                    {
                                        println!("Result: {result:?}");
                                        // TODO: Allow for specific updates for different data types, as they grow and improve

                                        println!("ID AND QUEUE {id:?}, {q_id:?}");
                                        if id.eq(&q_id) {
                                            if let kad::QueryResult::GetRecord(Ok(
                                                GetRecordOk::FoundRecord(PeerRecord {
                                                    peer,
                                                    record,
                                                }),
                                            )) = result
                                            {
                                                println!("THIS IS THE RESULT\n\n\n");
                                                break record;
                                            }
                                        } else {
                                            println!("Skipped");
                                        }
                                    }
                                };
                                println!("{item:?}");
                                swarm
                                    .behaviour_mut()
                                    .event_kad
                                    .put_record(item, kad::Quorum::One)
                                    .expect("Update fail");
                            }
                            gdelt_fetcher::models::gdelt::PrimaryKey::MentionKey(_) => {
                                let q_id = swarm.behaviour_mut().mentions_kad.get_record(key);
                                let item = loop {
                                    if let Ok(kad::Event::OutboundQueryProgressed {
                                        id,
                                        result,
                                        stats,
                                        step,
                                    }) = rx.recv().await
                                    {
                                        println!("\n\n\n\n HERE \n\n\n\n\n");
                                        // TODO: Allow for specific updates for different data types, as they grow and improve
                                        if id.eq(&q_id) {
                                            if let kad::QueryResult::GetRecord(Ok(
                                                GetRecordOk::FoundRecord(PeerRecord {
                                                    peer,
                                                    record,
                                                }),
                                            )) = result
                                            {
                                                println!("THIS IS THE RESULT");
                                                break record;
                                            }
                                        }
                                    }
                                };
                                swarm
                                    .behaviour_mut()
                                    .mentions_kad
                                    .put_record(item, kad::Quorum::One)
                                    .expect("Update fail");
                            }
                            gdelt_fetcher::models::gdelt::PrimaryKey::GKGKey(_) => {
                                let q_id = swarm.behaviour_mut().mentions_kad.get_record(key);
                                let item = loop {
                                    if let Ok(kad::Event::OutboundQueryProgressed {
                                        id,
                                        result,
                                        stats,
                                        step,
                                    }) = rx.recv().await
                                    {
                                        println!("\n\n\n\n HERE \n\n\n\n\n");
                                        // TODO: Allow for specific updates for different data types, as they grow and improve
                                        if id.eq(&q_id) {
                                            if let kad::QueryResult::GetRecord(Ok(
                                                GetRecordOk::FoundRecord(PeerRecord {
                                                    peer,
                                                    record,
                                                }),
                                            )) = result
                                            {
                                                break record;
                                            }
                                        }
                                    }
                                };
                                swarm
                                    .behaviour_mut()
                                    .mentions_kad
                                    .put_record(item, kad::Quorum::One)
                                    .expect("Update fail");
                            }
                        }
                    }
                }
            }
        }
    }
}
