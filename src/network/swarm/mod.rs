use std::time::Duration;

use libp2p::{Swarm, SwarmBuilder, yamux};
use netabase_store::traits::NetabaseSchema;

use crate::network::{
    behaviour::{NetabaseBehaviour, clone_impl::NetabaseSwarmEvent},
    swarm::handlers::{command_events::Command, start_swarm_loop},
};

pub mod handlers;

pub fn generate_swarm<S: NetabaseSchema>() -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    generate_swarm_with_name::<S>(None)
}

pub fn generate_swarm_with_name<S: NetabaseSchema>(
    name: Option<String>,
) -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    Ok(SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            Default::default(),
            (libp2p::tls::Config::new, libp2p::noise::Config::new),
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|kp| {
            NetabaseBehaviour::new_with_name(kp, name.clone()).expect("Failed to build behaviour")
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(120)))
        .build())
}

pub async fn start_swarm<S: NetabaseSchema>(
    mut swarm: Swarm<NetabaseBehaviour<S>>,
    swarm_event_sender: tokio::sync::broadcast::Sender<NetabaseSwarmEvent<S>>,
    command_event_listener: tokio::sync::mpsc::Receiver<Command<S>>,
) -> anyhow::Result<()> {
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    start_swarm_loop(swarm, swarm_event_sender, command_event_listener).await;
    Ok(())
}
