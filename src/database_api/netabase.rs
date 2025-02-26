use libp2p::{Swarm, futures::channel::mpsc};

use crate::{
    config::behaviour::NetabaseBehaviour,
    swarm::{run_swarm, swarm_config::swarm_init},
};

pub struct Netabase {
    swarm: Swarm<NetabaseBehaviour>,
    command_channel: (mpsc::Sender<Command>, mpsc::Receiver<Command>),
}

impl Netabase {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (command_sender, command_receiver) = mpsc::channel::<Command>(10);
        let swarm = swarm_init()?;

        Ok(Netabase {
            swarm,
            command_channel: (command_sender, command_receiver),
        })
    }

    pub fn init(&mut self) {
        run_swarm(&mut self.swarm, self.command_channel.1);
    }
}
