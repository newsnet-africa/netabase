use crate::{
    netabase_trait::{NetabaseSchema, NetabaseSchemaKey},
    network::event_messages::command_messages::database_commands::DatabaseCommand,
};

pub enum NetabaseCommand<Key: NetabaseSchemaKey, Value: NetabaseSchema> {
    Close,
    Database(DatabaseCommand<Key, Value>),
}

pub mod database_commands {
    use crate::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};

    pub enum DatabaseCommand<K: NetabaseSchemaKey, V: NetabaseSchema> {
        PutCommand(PutCommand<V>),
        GetCommand(GetCommand<K>),
    }

    pub enum PutCommand<V: NetabaseSchema> {
        PutRecord(V),
    }

    pub enum GetCommand<K: NetabaseSchemaKey> {
        GetRecord(K),
    }
}
