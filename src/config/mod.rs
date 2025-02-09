use behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent};
use libp2p::{
    Swarm,
    futures::StreamExt,
    kad::{Record, RecordKey},
    swarm::SwarmEvent,
};
use std::sync::mpmc::{Receiver, Sender};
pub mod behaviour;
pub mod database;
pub mod swarm;

pub enum EventSent {
    Sent,
    NotSent,
}

pub async fn swarm_event_handler(
    swarm: &mut Swarm<NetabaseBehaviour>,
    event: SwarmEvent<NetabaseBehaviourEvent>,
    sender: &mut Sender<SwarmEvent<NetabaseBehaviourEvent>>,
) -> EventSent {
    match &event {
        SwarmEvent::Behaviour(e) => {
            println!("{e:?}");
            match e {
                NetabaseBehaviourEvent::Mdns(m) => match m {
                    libp2p::mdns::Event::Discovered(items) => {
                        for (p, m) in items {
                            swarm.dial(m.clone());
                            swarm.behaviour_mut().event_kad.add_address(p, m.clone());
                        }
                    }
                    libp2p::mdns::Event::Expired(items) => todo!(),
                },
                e => println!("{e:?}\n\n\n"),
            }
        }
        e => {
            println!("{e:?}");
            if let SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } = e
            {
                let record =
                    Record::new(RecordKey::new(b"key"), "Values bitch".as_bytes().to_vec());
                let behaviour = &mut swarm.behaviour_mut().event_kad;
                let _ = behaviour.put_record(record, libp2p::kad::Quorum::One);
            }
        }
    }

    match sender.send(event) {
        Ok(_) => EventSent::Sent,
        Err(_) => EventSent::NotSent,
    }
}
