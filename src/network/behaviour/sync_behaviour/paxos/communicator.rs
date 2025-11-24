use super::{log_entry::LogEntry, CoordNum, NodeId, PaxosBehaviour, RoundNum};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use paxakos::communicator::{Acceptance, Committed, Communicator, Vote};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// Marker types for paxakos Communicator trait associated types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Abstain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Yea;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nay;

/// Communication error wrapper
#[derive(Debug, thiserror::Error)]
pub enum CommError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Peer not reachable: {0}")]
    PeerUnreachable(NodeId),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Future type for sending prepare messages - returns a Vote
pub type SendPrepareFuture<D> = Pin<
    Box<
        dyn Future<Output = Result<Vote<RoundNum, CoordNum, LogEntry<D>, Abstain>, CommError>>
            + Send
            + 'static,
    >,
>;

/// Future type for sending proposal messages - returns an Acceptance
pub type SendProposalFuture<D> = Pin<
    Box<
        dyn Future<Output = Result<Acceptance<CoordNum, LogEntry<D>, Yea, Nay>, CommError>>
            + Send
            + 'static,
    >,
>;

/// Future type for sending commit messages - returns Committed
pub type SendCommitFuture =
    Pin<Box<dyn Future<Output = Result<Committed, CommError>> + Send + 'static>>;

/// Future type for sending commit-by-id messages - returns Committed
pub type SendCommitByIdFuture =
    Pin<Box<dyn Future<Output = Result<Committed, CommError>> + Send + 'static>>;

// Implement Communicator trait for PaxosBehaviour
impl<D> Communicator for PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
    D: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> D: Deserialize<'d>,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
    <D as NetabaseDefinitionTrait>::Keys: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> <D as NetabaseDefinitionTrait>::Keys: Deserialize<'d>,
{
    type Node = NodeId;
    type RoundNum = RoundNum;
    type CoordNum = CoordNum;
    type LogEntry = LogEntry<D>;
    type Error = CommError;

    type SendPrepare = SendPrepareFuture<D>;
    type SendProposal = SendProposalFuture<D>;
    type SendCommit = SendCommitFuture;
    type SendCommitById = SendCommitByIdFuture;

    type Abstain = Abstain;
    type Yea = Yea;
    type Nay = Nay;

    /// Send a prepare message to all receivers
    fn send_prepare<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        _round_num: Self::RoundNum,
        _coord_num: Self::CoordNum,
    ) -> Vec<(&'a Self::Node, Self::SendPrepare)> {
        receivers
            .iter()
            .map(|peer| {
                // Create future that sends the prepare request
                let fut = Box::pin(async move {
                    // TODO: Actually send the request via the request_response protocol
                    // For now, return a placeholder vote (abstain)
                    Ok(Vote::Abstained(Abstain))
                }) as SendPrepareFuture<D>;

                (peer, fut)
            })
            .collect()
    }

    /// Send a proposal message to all receivers
    fn send_proposal<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        _round_num: Self::RoundNum,
        _coord_num: Self::CoordNum,
        _log_entry: std::sync::Arc<Self::LogEntry>,
    ) -> Vec<(&'a Self::Node, Self::SendProposal)> {
        receivers
            .iter()
            .map(|peer| {
                // Create future that sends the proposal request
                let fut = Box::pin(async move {
                    // TODO: Actually send the request via the request_response protocol
                    // For now, return a placeholder acceptance (given)
                    Ok(Acceptance::Given(Yea))
                }) as SendProposalFuture<D>;

                (peer, fut)
            })
            .collect()
    }

    /// Send a commit message to all receivers
    fn send_commit<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        _round_num: Self::RoundNum,
        _coord_num: Self::CoordNum,
        _log_entry: std::sync::Arc<Self::LogEntry>,
    ) -> Vec<(&'a Self::Node, Self::SendCommit)> {
        receivers
            .iter()
            .map(|peer| {
                // Create future that sends the commit request
                let fut = Box::pin(async move {
                    // TODO: Actually send the request via the request_response protocol
                    // For now, return a placeholder committed response
                    Ok(Committed)
                }) as SendCommitFuture;

                (peer, fut)
            })
            .collect()
    }

    /// Send a commit-by-id message to all receivers
    fn send_commit_by_id<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        _round_num: Self::RoundNum,
        _coord_num: Self::CoordNum,
        _log_entry_id: <Self::LogEntry as paxakos::LogEntry>::Id,
    ) -> Vec<(&'a Self::Node, Self::SendCommitById)> {
        receivers
            .iter()
            .map(|peer| {
                // Create future that sends the commit-by-id request
                let fut = Box::pin(async move {
                    // TODO: Actually send the request via the request_response protocol
                    // For now, return a placeholder committed response
                    Ok(Committed)
                }) as SendCommitByIdFuture;

                (peer, fut)
            })
            .collect()
    }
}
