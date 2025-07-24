use std::{path::PathBuf, time::Duration};

pub struct MaxStorage(u128);

pub struct NetabaseConfig {
    storage_path: PathBuf,
    max_storage: MaxStorage,
    web: bool,
    persistent: bool,
    timeout: Duration,
}
