#![feature(duration_constructors_lite)]

use rand::Rng;

use crate::config::NetabaseConfig;

pub mod config;
pub mod database;
pub mod network;

pub enum NetabaseCommand {
    Database(DatabaseCommand),
}

pub enum DatabaseCommand<K: Into<Vec<u8>>, V: Into<Vec<u8>>> {
    Put(K, V),
    Get(K),
    Delete(K),
}
pub struct Netabase {
    config: NetabaseConfig,
    swarm_thread: tokio::task::JoinHandle<anyhow::Result<()>>,
}
