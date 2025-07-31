use bincode::Encode;

pub trait NetabaseSchema:
    Clone
    + From<libp2p::kad::Record>
    + Encode
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::Record>
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchemaKey:
<<<<<<< HEAD
    Clone + From<libp2p::kad::RecordKey> + Encode + Decode<()> + Into<libp2p::kad::RecordKey>
=======
    Clone
    + From<libp2p::kad::RecordKey>
    + Encode
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::RecordKey>
>>>>>>> 4740b930844447b717a06adb472169f5fb202c37
{
}
