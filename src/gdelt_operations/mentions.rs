use gdelt_fetcher::models::gdelt::{DatabaseTableEntry, event::Event, mentions::Mentions};
use libp2p::Swarm;

use crate::config::{behaviour::NetabaseBehaviour, database::LocalDatabase};

use super::DatabaseOperation;

// impl DatabaseOperation<Mentions> for LocalDatabase {
//     fn create(
//         swarm: &mut libp2p::Swarm<crate::config::behaviour::NetabaseBehaviour>,
//         item: Mentions,
//     ) {
//         swarm
//             .behaviour_mut()
//             .mentions_kad
//             .put_record(record, quorum)
//     }

//     fn read(
//         swarm: &mut libp2p::Swarm<crate::config::behaviour::NetabaseBehaviour>,
//         key: gdelt_fetcher::models::gdelt::PrimaryKey,
//     ) {
//         todo!()
//     }

//     fn update(
//         swarm: &mut libp2p::Swarm<crate::config::behaviour::NetabaseBehaviour>,
//         item: Mentions,
//     ) {
//         todo!()
//     }

//     fn delete(
//         swarm: &mut libp2p::Swarm<crate::config::behaviour::NetabaseBehaviour>,
//         key: gdelt_fetcher::models::gdelt::PrimaryKey,
//     ) {
//         todo!()
//     }
// }
