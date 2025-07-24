use libp2p::swarm::SwarmEvent;

use crate::network::behaviour::NetabaseBehaviour;

pub async fn handle_command(
    command: NetabaseCommand,
    event_listner: std::sync::mpmc::Receiver<SwarmEvent<NetabaseBehaviour>>,
) -> anyhow::Result<()> {
}
