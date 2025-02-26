use serde::{Deserialize, Serialize};

use super::DataType;

pub enum NetabaseCommand<T: DataType> {
    Put(T),
    Read(T::Key),
    Delete(T::Key),
    Update(T::Key, T),
    Refresh,
}
