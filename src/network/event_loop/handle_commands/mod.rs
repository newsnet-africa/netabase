use libp2p::Swarm;
use libp2p::kad::QueryId;
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::{
    netabase_trait::{NetabaseSchema, NetabaseSchemaKey},
    network::behaviour::NetabaseBehaviour,
    network::event_messages::command_messages::{CommandResponse, NetabaseCommand},
};

pub mod configuration_commands;
pub mod database_commands;
pub mod network_commands;
pub mod system_commands;

use configuration_commands::handle_configuration_command;
use database_commands::{DatabaseOperationContext, handle_database_command};

use system_commands::handle_system_command;

/// Main command handler that dispatches commands to appropriate handlers
pub fn handle_command<
    K: NetabaseSchemaKey + std::fmt::Debug,
    V: NetabaseSchema + std::fmt::Debug,
>(
    command: NetabaseCommand<K, V>,
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
    query_queue: &mut HashMap<QueryId, oneshot::Sender<CommandResponse<K, V>>>,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) {
    match command {
        NetabaseCommand::System(system_command) => {
            handle_system_command(system_command, response_sender);
        }
        NetabaseCommand::Database(database_command) => {
            handle_database_command(
                database_command,
                response_sender,
                query_queue,
                database_context,
                swarm,
            );
        }
        NetabaseCommand::Network(network_command) => {
            network_commands::handle_network_command(
                network_command,
                response_sender,
                query_queue,
                swarm,
            );
        }
        NetabaseCommand::Configuration(configuration_command) => {
            handle_configuration_command(configuration_command, response_sender);
        }
        NetabaseCommand::Close => {
            handle_close(response_sender);
        }
    }
}

fn handle_close<K: NetabaseSchemaKey, V: NetabaseSchema>(
    response_sender: Option<oneshot::Sender<CommandResponse<K, V>>>,
) {
    // TODO: Implement proper close logic
    log::info!("Close command received - shutting down");

    if let Some(sender) = response_sender {
        let _ = sender.send(CommandResponse::Success);
    }

    // The event loop should break when this command is received
}
