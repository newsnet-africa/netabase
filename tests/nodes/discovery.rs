use libp2p::identity::ed25519::Keypair;
use netabase::{Netabase, config::NetabaseConfig, init_logging};

#[tokio::test]
async fn creation() {
    let netabase_config = NetabaseConfig::default();
    let mut netabase = Netabase::try_new(netabase_config, "netabase_test_creation")
        .await
        .expect("faileed to start netabase");
    println!("Starting Swarm");
    netabase.start_swarm();

    netabase.close_swarm().await;
}

#[tokio::test]
async fn connection1() {
    let mut netabase1 = Netabase::try_new_default("/netabase_test_connection1")
        .await
        .expect("Failed");
    netabase1.start_swarm();
    let mut listener_options = netabase1
        .swarm_event_listener
        .unwrap()
        .resubscribe()
        .clone();
    eprint!("{:?}", listener_options.recv().await);
    netabase1.close_swarm();
}
#[tokio::test]
async fn connection2() {
    let mut netabase2 = Netabase::try_new_default("/netabase_test_connection2")
        .await
        .expect("Failed");
    netabase2.start_swarm();
    let mut listener_options = netabase2.swarm_event_listener.unwrap().resubscribe();
    eprint!("{:?}", listener_options.recv().await);
    netabase2.close_swarm();
}
