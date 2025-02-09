use std::{sync::mpmc::Sender, time::Duration};

use libp2p::{
    StreamProtocol, Swarm,
    futures::StreamExt,
    identity,
    kad::{self, Mode},
    noise,
    swarm::{self, SwarmEvent},
    tcp, yamux,
};
use std::sync::mpmc::Receiver;
use tokio::select;

use crate::config::swarm_event_handler;

use super::behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent};

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

    Ok(swarm)
}

pub async fn run_swarm(
    swarm: &mut Swarm<NetabaseBehaviour>,
    sender: &mut Sender<SwarmEvent<NetabaseBehaviourEvent>>,
) {
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().expect("Parse erruh"))
        .expect("Listen Erruh");
    swarm.behaviour_mut().event_kad.set_mode(Some(Mode::Server));
    println!("Running Swarm");
    loop {
        select! {
            swarm_event = swarm.select_next_some() => {
                swarm_event_handler(swarm, swarm_event, sender).await;
            }
        }
    }
}

#[cfg(test)]
mod run_swarm_test {
    use std::{sync::mpmc, time::Duration};

    use libp2p::{
        PeerId,
        ping::{self, Event},
        swarm::{ConnectionId, SwarmEvent},
    };

    use crate::config::{
        behaviour::NetabaseBehaviourEvent,
        swarm::{run_swarm, swarm_init},
    };
    // This test should show:
    // 1. The sender from the run_swarm -> swarm_handler is working
    // 2. Once recieved by ANY thread, the message is consumed
    #[tokio::test]
    async fn swarm_test() {
        let mut swarm = swarm_init().expect("Swarm init error");
        let (mut sender, mut receiver) = mpmc::channel();
        let mut handles = vec![];

        // Use Tokio's executor to spawn the async sender task
        let s_handle = tokio::spawn(async move {
            println!("Sender Started loop");
            run_swarm(&mut swarm, &mut sender).await;
            loop {
                println!("Sender Started loop");
                let _ = sender.try_send(SwarmEvent::Behaviour(NetabaseBehaviourEvent::Ping(
                    ping::Event {
                        peer: PeerId::random(),
                        connection: ConnectionId::new_unchecked(0),
                        result: Ok(Duration::from_secs(5)),
                    },
                )));
                // Avoid burning CPU in the loop
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        for i in 0..5 {
            let receiver = receiver.clone();
            // Since receiver is blocking, it's okay to use std::thread::spawn
            let handle = std::thread::spawn(move || {
                println!("Receiver {} started", i);
                while let Ok(msg) = receiver.recv() {
                    println!("Receiver {} received: {:?}", i, msg);
                }
                loop {
                    if let Ok(msg) = receiver.recv() {
                        println!("Receiver {} received: {:?}", i, msg);
                        break;
                    }
                }
                println!("Receiver {} finished", i);
            });
            handles.push(handle);
        }

        // Wait for the sender task
        s_handle.await.unwrap();
        for handle in handles {
            handle.join();
        }
    }
}
