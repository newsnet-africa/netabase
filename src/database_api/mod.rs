use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub mod commands;
pub mod database_functions;
pub mod netabase;

pub trait DataType: DeserializeOwned + Serialize + CommandParam {
    type Key: CommandParam;

    fn key(&self) -> Self::Key;
}

pub(crate) trait CommandParam {}
