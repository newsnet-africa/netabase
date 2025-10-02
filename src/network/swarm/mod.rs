use std::time::Duration;

use libp2p::{Swarm, SwarmBuilder, yamux};
use netabase_store::traits::NetabaseSchema;

use crate::network::behaviour::NetabaseBehaviour;

pub mod handlers;

pub fn generate_swarm<S: NetabaseSchema>() -> anyhow::Result<Swarm<NetabaseBehaviour<S>>> {
    Ok(SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            Default::default(),
            (libp2p::tls::Config::new, libp2p::noise::Config::new),
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|kp| NetabaseBehaviour::new(&kp).expect("Failed to build behaviour"))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(120)))
        .build())
}

pub fn start_swarm<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
) -> anyhow::Result<()> {
    swarm.listen_on("/ip4/0.0.0.0/udp/quic-v1".parse()?)?;
    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));
    Ok(())
}
