//! Paxos consensus implementation
//!
//! This module implements the Paxos consensus algorithm for distributed agreement.
//! Paxos ensures that a distributed system can agree on a single value even in the
//! presence of failures.
//!
//! ## Paxos Roles
//!
//! - **Proposers**: Initiate proposals with unique proposal numbers
//! - **Acceptors**: Vote on proposals (promise and accept phases)
//! - **Learners**: Learn the chosen value once consensus is reached
//!
//! ## Paxos Phases
//!
//! 1. **Prepare (Phase 1a)**: Proposer sends PREPARE(n) to acceptors
//! 2. **Promise (Phase 1b)**: Acceptors respond with PROMISE(n, accepted_value)
//! 3. **Accept (Phase 2a)**: Proposer sends ACCEPT(n, value) to acceptors
//! 4. **Accepted (Phase 2b)**: Acceptors respond with ACCEPTED(n, value)
//!
//! ## Configuration
//!
//! For n acceptors and f failures: need f+1 promises and f+1 accepts for consensus

use anyhow::{anyhow, Result};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Proposal number (ballot number)
/// Consists of (round, proposer_id) for uniqueness and total ordering
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProposalNumber {
    /// Round number (higher is newer)
    pub round: u64,

    /// Proposer ID for tie-breaking
    #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
    #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
    pub proposer: PeerId,
}

impl ProposalNumber {
    /// Create a new proposal number
    pub fn new(round: u64, proposer: PeerId) -> Self {
        Self { round, proposer }
    }

    /// Increment to next round
    pub fn next(&self) -> Self {
        Self {
            round: self.round + 1,
            proposer: self.proposer,
        }
    }
}

/// Paxos message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaxosMessage {
    /// Phase 1a: Proposer → Acceptors
    Prepare {
        proposal_number: ProposalNumber,
    },

    /// Phase 1b: Acceptors → Proposer
    Promise {
        proposal_number: ProposalNumber,
        accepted_proposal: Option<ProposalNumber>,
        accepted_value: Option<Vec<u8>>,
    },

    /// Phase 2a: Proposer → Acceptors
    Accept {
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    },

    /// Phase 2b: Acceptors → Learners
    Accepted {
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    },
}

/// Acceptor state
#[derive(Clone, Debug)]
struct AcceptorState {
    /// Highest proposal number promised
    promised_number: Option<ProposalNumber>,

    /// Highest proposal number accepted
    accepted_number: Option<ProposalNumber>,

    /// Value accepted
    accepted_value: Option<Vec<u8>>,
}

impl Default for AcceptorState {
    fn default() -> Self {
        Self {
            promised_number: None,
            accepted_number: None,
            accepted_value: None,
        }
    }
}

/// Proposer state for a proposal
#[derive(Clone, Debug)]
struct ProposerState {
    /// Proposal number
    proposal_number: ProposalNumber,

    /// Proposed value
    value: Vec<u8>,

    /// Promises received
    promises: HashMap<PeerId, (Option<ProposalNumber>, Option<Vec<u8>>)>,

    /// Accepts received
    accepts: HashSet<PeerId>,

    /// Whether proposal was successful
    succeeded: bool,
}

/// Paxos configuration
#[derive(Clone, Debug)]
pub struct PaxosConfig {
    /// Total number of acceptors
    pub num_acceptors: usize,

    /// Number of failures to tolerate
    pub max_failures: usize,
}

impl PaxosConfig {
    /// Create a new Paxos configuration
    pub fn new(num_acceptors: usize, max_failures: usize) -> Self {
        assert!(
            num_acceptors >= 2 * max_failures + 1,
            "Need at least 2f+1 acceptors to tolerate f failures"
        );
        Self {
            num_acceptors,
            max_failures,
        }
    }

    /// Get quorum size (f+1)
    pub fn quorum_size(&self) -> usize {
        self.max_failures + 1
    }
}

/// Paxos instance manager
pub struct PaxosInstance {
    /// Local peer ID
    local_peer_id: PeerId,

    /// Configuration
    config: PaxosConfig,

    /// Acceptor state (if this peer is an acceptor)
    acceptor: AcceptorState,

    /// Active proposals (if this peer is a proposer)
    proposals: HashMap<ProposalNumber, ProposerState>,

    /// Learned values
    learned_values: Vec<Vec<u8>>,

    /// Current round number
    current_round: u64,
}

impl PaxosInstance {
    /// Create a new Paxos instance
    pub fn new(local_peer_id: PeerId, config: PaxosConfig) -> Self {
        Self {
            local_peer_id,
            config,
            acceptor: AcceptorState::default(),
            proposals: HashMap::new(),
            learned_values: Vec::new(),
            current_round: 0,
        }
    }

    /// Propose a value (Phase 1a: Prepare)
    pub fn propose(&mut self, value: Vec<u8>) -> ProposalNumber {
        self.current_round += 1;
        let proposal_number = ProposalNumber::new(self.current_round, self.local_peer_id);

        let state = ProposerState {
            proposal_number,
            value,
            promises: HashMap::new(),
            accepts: HashSet::new(),
            succeeded: false,
        };

        self.proposals.insert(proposal_number, state);
        proposal_number
    }

    /// Handle incoming prepare message (Phase 1a)
    /// Returns a Promise message if accepted
    pub fn handle_prepare(&mut self, proposal_number: ProposalNumber) -> Result<PaxosMessage> {
        // Check if we've already promised a higher number
        if let Some(promised) = self.acceptor.promised_number {
            if proposal_number < promised {
                return Err(anyhow!(
                    "Already promised higher proposal: {:?}",
                    promised
                ));
            }
        }

        // Promise this proposal
        self.acceptor.promised_number = Some(proposal_number);

        // Return promise with previously accepted value (if any)
        Ok(PaxosMessage::Promise {
            proposal_number,
            accepted_proposal: self.acceptor.accepted_number,
            accepted_value: self.acceptor.accepted_value.clone(),
        })
    }

    /// Handle incoming promise message (Phase 1b)
    pub fn handle_promise(
        &mut self,
        from: PeerId,
        proposal_number: ProposalNumber,
        accepted_proposal: Option<ProposalNumber>,
        accepted_value: Option<Vec<u8>>,
    ) -> Result<Option<PaxosMessage>> {
        // Get proposer state
        let state = self
            .proposals
            .get_mut(&proposal_number)
            .ok_or_else(|| anyhow!("Unknown proposal"))?;

        // Add promise
        state.promises.insert(from, (accepted_proposal, accepted_value));

        // Check if we have quorum
        if state.promises.len() >= self.config.quorum_size() {
            // Find highest accepted value from promises
            let highest_accepted: Option<(ProposalNumber, Option<Vec<u8>>)> = state
                .promises
                .values()
                .filter_map(|(prop, val)| prop.as_ref().map(|p| (*p, val.clone())))
                .max_by_key(|(prop, _)| *prop);

            // Use highest accepted value, or our proposed value
            let value = if let Some((_, Some(val))) = highest_accepted {
                val
            } else {
                state.value.clone()
            };

            // Move to Phase 2a: Accept
            return Ok(Some(PaxosMessage::Accept {
                proposal_number,
                value,
            }));
        }

        Ok(None)
    }

    /// Handle incoming accept message (Phase 2a)
    pub fn handle_accept(
        &mut self,
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    ) -> Result<PaxosMessage> {
        // Check if we've promised a higher number
        if let Some(promised) = self.acceptor.promised_number {
            if proposal_number < promised {
                return Err(anyhow!("Already promised higher proposal"));
            }
        }

        // Accept the proposal
        self.acceptor.accepted_number = Some(proposal_number);
        self.acceptor.accepted_value = Some(value.clone());

        // Send accepted message
        Ok(PaxosMessage::Accepted {
            proposal_number,
            value,
        })
    }

    /// Handle incoming accepted message (Phase 2b)
    pub fn handle_accepted(
        &mut self,
        from: PeerId,
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    ) -> Result<bool> {
        // Get proposer state
        let state = self
            .proposals
            .get_mut(&proposal_number)
            .ok_or_else(|| anyhow!("Unknown proposal"))?;

        // Add accept
        state.accepts.insert(from);

        // Check if we have quorum
        if state.accepts.len() >= self.config.quorum_size() && !state.succeeded {
            state.succeeded = true;
            self.learned_values.push(value);
            return Ok(true); // Consensus reached!
        }

        Ok(false)
    }

    /// Get learned values
    pub fn learned_values(&self) -> &[Vec<u8>] {
        &self.learned_values
    }

    /// Get local peer ID
    pub fn peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Check if a proposal succeeded
    pub fn is_proposal_successful(&self, proposal_number: &ProposalNumber) -> bool {
        self.proposals
            .get(proposal_number)
            .map(|s| s.succeeded)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_number_ordering() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let p1 = ProposalNumber::new(1, peer1);
        let p2 = ProposalNumber::new(2, peer1);
        let p3 = ProposalNumber::new(2, peer2);

        assert!(p1 < p2);
        assert!(p2 < p3 || p2 > p3); // Depends on PeerId ordering
    }

    #[test]
    fn test_proposal_number_next() {
        let peer = PeerId::random();
        let p1 = ProposalNumber::new(1, peer);
        let p2 = p1.next();

        assert_eq!(p2.round, 2);
        assert_eq!(p2.proposer, peer);
    }

    #[test]
    fn test_paxos_config() {
        let config = PaxosConfig::new(5, 2);
        assert_eq!(config.quorum_size(), 3);
    }

    #[test]
    #[should_panic(expected = "Need at least 2f+1 acceptors")]
    fn test_paxos_config_invalid() {
        PaxosConfig::new(3, 2); // Need at least 5 for f=2
    }

    #[test]
    fn test_paxos_propose() {
        let peer = PeerId::random();
        let config = PaxosConfig::new(3, 1);
        let mut instance = PaxosInstance::new(peer, config);

        let value = b"test value".to_vec();
        let proposal = instance.propose(value.clone());

        assert_eq!(proposal.round, 1);
        assert_eq!(proposal.proposer, peer);
    }

    #[test]
    fn test_paxos_handle_prepare() {
        let peer = PeerId::random();
        let config = PaxosConfig::new(3, 1);
        let mut instance = PaxosInstance::new(peer, config);

        let proposal = ProposalNumber::new(1, peer);
        let result = instance.handle_prepare(proposal);

        assert!(result.is_ok());
        match result.unwrap() {
            PaxosMessage::Promise {
                proposal_number,
                accepted_proposal,
                accepted_value,
            } => {
                assert_eq!(proposal_number, proposal);
                assert!(accepted_proposal.is_none());
                assert!(accepted_value.is_none());
            }
            _ => panic!("Expected Promise message"),
        }
    }

    #[test]
    fn test_paxos_reject_lower_proposal() {
        let peer = PeerId::random();
        let config = PaxosConfig::new(3, 1);
        let mut instance = PaxosInstance::new(peer, config);

        let proposal1 = ProposalNumber::new(2, peer);
        instance.handle_prepare(proposal1).unwrap();

        let proposal2 = ProposalNumber::new(1, peer);
        let result = instance.handle_prepare(proposal2);

        assert!(result.is_err());
    }

    #[test]
    fn test_paxos_basic_consensus() {
        let proposer = PeerId::random();
        let acceptor1 = PeerId::random();
        let acceptor2 = PeerId::random();

        let config = PaxosConfig::new(3, 1);

        // Proposer
        let mut proposer_instance = PaxosInstance::new(proposer, config.clone());
        let value = b"test value".to_vec();
        let proposal = proposer_instance.propose(value.clone());

        // Acceptors handle prepare
        let mut acceptor1_instance = PaxosInstance::new(acceptor1, config.clone());
        let mut acceptor2_instance = PaxosInstance::new(acceptor2, config.clone());

        let promise1 = acceptor1_instance.handle_prepare(proposal).unwrap();
        let promise2 = acceptor2_instance.handle_prepare(proposal).unwrap();

        // Proposer handles promises
        if let PaxosMessage::Promise {
            proposal_number,
            accepted_proposal,
            accepted_value,
        } = promise1
        {
            proposer_instance.handle_promise(
                acceptor1,
                proposal_number,
                accepted_proposal,
                accepted_value,
            ).unwrap();
        }

        if let PaxosMessage::Promise {
            proposal_number,
            accepted_proposal,
            accepted_value,
        } = promise2
        {
            let accept_msg = proposer_instance
                .handle_promise(acceptor2, proposal_number, accepted_proposal, accepted_value)
                .unwrap();

            // Should get Accept message after quorum
            assert!(accept_msg.is_some());

            if let Some(PaxosMessage::Accept {
                proposal_number: accept_proposal,
                value: accept_value,
            }) = accept_msg
            {
                // Acceptors handle accept
                let accepted1 = acceptor1_instance.handle_accept(accept_proposal, accept_value.clone()).unwrap();
                let accepted2 = acceptor2_instance.handle_accept(accept_proposal, accept_value.clone()).unwrap();

                // Proposer handles accepted
                if let PaxosMessage::Accepted { proposal_number, value } = accepted1 {
                    proposer_instance.handle_accepted(acceptor1, proposal_number, value).unwrap();
                }

                if let PaxosMessage::Accepted { proposal_number, value } = accepted2 {
                    let consensus = proposer_instance.handle_accepted(acceptor2, proposal_number, value).unwrap();
                    assert!(consensus); // Consensus reached!
                }
            }
        }

        assert_eq!(proposer_instance.learned_values().len(), 1);
        assert_eq!(proposer_instance.learned_values()[0], b"test value");
    }
}
