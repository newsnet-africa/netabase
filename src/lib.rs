use bincode::{Decode, Encode};
use libp2p::kad::{Record, RecordKey};

pub trait NetabaseSchema: Encode + Decode<()> + Sized {
    type Key: Encode + Decode<()> + Into<RecordKey>;
    fn key() -> Self::Key;

    fn to_record(self) -> Result<Record, anyhow::Error> {
        Ok(Record {
            key: Self::key().into(),
            value: bincode::encode_to_vec(&self, bincode::config::standard())?,
            publisher: None,
            expires: None,
        })
    }
}
