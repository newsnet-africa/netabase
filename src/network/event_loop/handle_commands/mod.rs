use libp2p::Swarm;
use libp2p::kad::QueryId;
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::{
    netabase_trait::{self, NetabaseRegistery, NetabaseSchema, NetabaseSchemaKey},
    network::{
        behaviour::NetabaseBehaviour,
        event_messages::command_messages::{CommandResponse, NetabaseCommand},
    },
};

pub mod configuration_commands;
pub mod database_commands;
pub mod network_commands;
pub mod system_commands;

use configuration_commands::handle_configuration_command;
use database_commands::{DatabaseOperationContext, handle_database_command};

use system_commands::handle_system_command;

pub fn handle_command<R: NetabaseRegistery>(
    command: NetabaseCommand<R::RegistryKey, R::RegistrySchema>,
    response_sender: Option<oneshot::Sender<CommandResponse<R::RegistryKey, R::RegistrySchema>>>,
    query_queue: &mut HashMap<
        QueryId,
        oneshot::Sender<CommandResponse<R::RegistryKey, R::RegistrySchema>>,
    >,
    database_context: &mut HashMap<QueryId, DatabaseOperationContext>,
    swarm: &mut Swarm<NetabaseBehaviour>,
) where
    <R as netabase_trait::NetabaseRegistery>::RegistrySchema:
        netabase_trait::NetabaseSchema + std::fmt::Debug,
    <R as netabase_trait::NetabaseRegistery>::RegistryKey:
        netabase_trait::NetabaseSchemaKey + std::fmt::Debug,
{
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
    log::info!("Close command received - shutting down");

    if let Some(sender) = response_sender {
        let _ = sender.send(CommandResponse::Success);
    }
}
