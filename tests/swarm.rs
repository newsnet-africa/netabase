#[cfg(test)]
mod swarm_test {
    use libp2p::kad::Record;
    use netabase::{
        config::behaviour::NetEvent,
        swarm::{
            handlers::run_swarm, messages::swarm_actions::SwarmAction, swarm_config::swarm_init,
        },
    };
    use tokio::{
        select,
        sync::{broadcast, mpsc},
    };
    #[tokio::test]
    async fn run_swarm_test() {
        let mut swarm = swarm_init().expect("Swarm erruh");
        let (mut broadcast_sender, mut broadcast_receiver) = broadcast::channel::<NetEvent>(300);
        let (mut action_sender, mut action_receiver) = mpsc::channel::<SwarmAction>(300);
        let run_handle = tokio::spawn(async move {
            run_swarm(&mut swarm, &mut broadcast_sender, &mut action_receiver).await;
        });
        let test_handle = tokio::spawn(async move {
            loop {
                action_sender.send(SwarmAction::Test);
                select! {
                    broad_message = broadcast_receiver.recv() => {
                        println!("Got Broadcast Message: {broad_message:?}");
                    }
                }
            }
        });

        run_handle.await;
        test_handle.await;
    }

    #[tokio::test]
    async fn put_record_test() {
        let mut swarm = swarm_init().expect("Swarm erruh");

        let (mut broadcast_sender, mut broadcast_receiver) = broadcast::channel::<NetEvent>(300);
        let (mut action_sender, mut action_receiver) = mpsc::channel::<SwarmAction>(300);
        let run_handle = tokio::spawn(async move {
            run_swarm(&mut swarm, &mut broadcast_sender, &mut action_receiver).await;
        });
        let test_handle =
            tokio::spawn(async move {
                loop {
                    let record =
                        Record::new(b"key".as_slice().to_vec(), "This Value".as_bytes().to_vec());
                    action_sender.send(SwarmAction::KadAction(
                    netabase::swarm::messages::swarm_actions::KadBehaviourMethods::PutRecord(
                        record,
                        libp2p::kad::Quorum::One,
                    ),
                )).await;
                    select! {
                        broad_message = broadcast_receiver.recv() => {
                            println!("Got Broadcast Message: {broad_message:?}");
                        }
                    }
                }
            });

        run_handle.await;
        test_handle.await;
    }

    #[tokio::test]
    async fn get_record_test() {
        let mut swarm = swarm_init().expect("Swarm erruh");

        let (mut broadcast_sender, mut broadcast_receiver) = broadcast::channel::<NetEvent>(300);
        let (mut action_sender, mut action_receiver) = mpsc::channel::<SwarmAction>(300);
        let run_handle = tokio::spawn(async move {
            run_swarm(&mut swarm, &mut broadcast_sender, &mut action_receiver).await;
        });

        let test_handle =
            tokio::spawn(async move {
                let record =
                    Record::new(b"key".as_slice().to_vec(), "This Value".as_bytes().to_vec());
                loop {
                    let key = record.key.clone();
                    // action_sender.send(SwarmAction::KadAction(
                    // netabase::swarm::messages::swarm_actions::KadBehaviourMethods::PutRecord(
                    //     record.clone(),
                    //     libp2p::kad::Quorum::One,
                    // ),
                    // )).await.expect("Put Erruh");
                    action_sender.send(SwarmAction::KadAction(
                    netabase::swarm::messages::swarm_actions::KadBehaviourMethods::GetRecord(
                        key,
                    ),
                )).await.expect("Get Erruh");
                    select! {
                        broad_message = broadcast_receiver.recv() => {
                            println!("Got Broadcast Message: {broad_message:?}");
                        }
                    }
                }
            });

        run_handle.await;
        test_handle.await;
    }
}
