use bincode::{Decode, Encode};
use netabase_store::definition::NetabaseDefinitionTrait;

/// A log entry type for Paxos consensus operations.
///
/// This enum represents different database operations that can be logged and replicated
/// through the Paxos consensus protocol. The generic parameter `D` represents the
/// database definition containing all models.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Encode)]
pub enum LogEntryType<D: NetabaseDefinitionTrait>
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
{
    PutRecord { record: D },
    GetRecord { key: D::Keys },
    RemoveRecord { key: D::Keys },
}

/// A unique identifier for log entries based on timestamp and content hash.
///
/// The ordering is based primarily on the timestamp, allowing for chronological
/// ordering of log entries. The hash provides uniqueness and content verification.
#[derive(PartialEq, Eq, bincode::Encode, bincode::Decode, Debug, Clone)]
pub struct NetabaseLogID(
    #[bincode(with_serde)] chrono::NaiveDateTime,
    #[bincode(with_serde)] blake3::Hash,
);

impl Ord for NetabaseLogID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for NetabaseLogID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => None,
            ord => return ord,
        }
    }
}

/// A Paxos log entry containing a database operation.
///
/// This struct wraps a `LogEntryType<D>` with a unique identifier, making it suitable
/// for use in the Paxos consensus protocol. Each entry represents a single database
/// operation that needs to be agreed upon by the cluster.
///
/// Note: This is a generic meta-type and should not be used with `netabase_definition_module`.
/// Instead, use it as a field within concrete models that are part of your schema.
#[derive(Debug, Clone, Encode)]
pub struct LogEntry<D: NetabaseDefinitionTrait>
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
{
    pub id: NetabaseLogID,
    pub entry: LogEntryType<D>,
}

impl<D: NetabaseDefinitionTrait + std::marker::Unpin> paxakos::LogEntry for LogEntry<D>
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
    <D as netabase_store::NetabaseDefinitionTrait>::Keys: std::marker::Unpin,
{
    type Id = NetabaseLogID;
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}
