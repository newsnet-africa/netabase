pub mod swarm_action;

use std::time::Duration;

use libp2p::{
    Swarm,
    futures::StreamExt,
    identity,
    kad::{self, GetRecordOk, Mode, PeerRecord, PutRecordOk, Record, RecordKey},
    noise,
    swarm::SwarmEvent,
    tcp, yamux,
};
use swarm_action::SwarmAction;
use tokio::{
    select,
    sync::{
        broadcast::{self, Sender},
        mpsc::{self, Receiver},
    },
};

use crate::gdelt_operations::KadAction;

use super::{
    EventSent,
    behaviour::{NetEvent, NetabaseBehaviour},
};

pub fn swarm_init() -> Result<Swarm<NetabaseBehaviour>, Box<dyn std::error::Error>> {
    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(NetabaseBehaviour::new)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(500)))
        .build();

    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().expect("Parse erruh"))
        .expect("Listen Erruh");
    swarm.behaviour_mut().event_kad.set_mode(Some(Mode::Server));
    swarm
        .behaviour_mut()
        .mentions_kad
        .set_mode(Some(Mode::Server));
    swarm.behaviour_mut().gkg_kad.set_mode(Some(Mode::Server));
    Ok(swarm)
}

pub async fn run_swarm(
    swarm: &mut Swarm<NetabaseBehaviour>,
    sender: &mut Sender<kad::Event>,
    receiver: &mut mpsc::Receiver<SwarmAction>,
    b_receiver: &mut broadcast::Receiver<kad::Event>,
) {
    println!("Running Swarm");
    loop {
        select! {
            swarm_event = swarm.select_next_some() => {
                swarm_event_handler(swarm, swarm_event, sender).await;
            },
            swarm_action = receiver.recv() => {
                if let Some(sa) = swarm_action {
                    sa.execute(swarm, b_receiver).await;
                }
            }
        }
    }
}

pub async fn swarm_event_handler(
    swarm: &mut Swarm<NetabaseBehaviour>,
    event: SwarmEvent<NetEvent>,
    sender: &mut Sender<kad::Event>,
) -> EventSent {
    println!("{:?}", &event);
    match event {
        SwarmEvent::Behaviour(e) => match e {
            NetEvent::Mdns(m) => match m {
                libp2p::mdns::Event::Discovered(items) => {
                    for (peer_id, multiaddr) in items {
                        swarm
                            .behaviour_mut()
                            .event_kad
                            .add_address(&peer_id, multiaddr.clone());
                        swarm
                            .behaviour_mut()
                            .gkg_kad
                            .add_address(&peer_id, multiaddr.clone());
                        swarm
                            .behaviour_mut()
                            .mentions_kad
                            .add_address(&peer_id, multiaddr);
                    }
                    EventSent::Sent
                }
                libp2p::mdns::Event::Expired(items) => EventSent::NotSent,
            },
            NetEvent::Kad(event) => match sender.send(event) {
                Ok(_) => EventSent::Sent,
                Err(_) => EventSent::NotSent,
            },
            _ => EventSent::NotSent,
        },
        _ => EventSent::NotSent,
    }
}

#[cfg(test)]
mod run_swarm_test {

    use std::{thread, time::Duration};

    use gdelt_fetcher::models::gdelt::{DatabaseTableEntry, event::Event};
    use libp2p::{
        Multiaddr, PeerId,
        kad::{self, Record, RecordKey},
    };
    use tokio::sync::{broadcast, mpsc};

    use crate::config::swarm::{SwarmAction, run_swarm, swarm_init};
    // This test should show:
    // 1. The sender from the run_swarm -> swarm_handler is working
    // 2. Once recieved by ANY thread, the message is consumed
    #[tokio::test]
    async fn swarm_test() {
        let mut swarm = swarm_init().expect("Swarm init error");
        let (mut sender, _) = broadcast::channel(1000);
        let (mut ac_sender, mut ac_reciever) = mpsc::channel(1000);

        let mut handles = vec![];

        let mut rec = sender.subscribe();
        let ac_se = ac_sender.clone();

        let handle = tokio::spawn(async move {
            loop {
                let _ = ac_se
                    .send(SwarmAction::Kad(
                        crate::gdelt_operations::KadAction::Create(
                            gdelt_fetcher::models::gdelt::DatabaseTableEnum::Event(Some(
                                Event::blank(),
                            )),
                        ),
                    ))
                    .await;
                thread::sleep_ms(5);
            }
        });

        handles.push(handle);

        let mut rec = sender.subscribe();
        println!("Sender Started loop");
        run_swarm(&mut swarm, &mut sender, &mut ac_reciever, &mut rec).await;

        for handle in handles {
            let res = handle.await;

            println!("mpsc::Receiver {:?} finished", res);
        }
        // Wait for the sender task
    }
}
