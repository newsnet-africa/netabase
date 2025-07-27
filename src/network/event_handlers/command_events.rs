use libp2p::swarm::SwarmEvent;

use crate::{NetabaseCommand, network::behaviour::NetabaseBehaviour};

pub async fn handle_command(
    _command: NetabaseCommand,
    _event_listner: tokio::sync::mpsc::Receiver<SwarmEvent<NetabaseBehaviour>>,
) -> anyhow::Result<()> {
    Ok(())
}
