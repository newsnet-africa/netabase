pub mod database_functions;
pub mod netabase;

pub trait DataType {
    type Key;

    fn key(&self) -> Self::Key;
}
