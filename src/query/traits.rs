use serde::{Serialize, Deserialize, de::DeserializeOwned};
use netabase_store::{
    prelude::{NetabaseDefinition, NetabaseModel},
    traits::registry::models::NetabaseModelKeys,
};
use strum::IntoDiscriminant;

use crate::node::{
    capabilities::{Capability, CapabilityRange},
    primitives::Operation,
};

use super::messages::QueryEnvelope;
use super::validation::ValidationError;

/// The result of a query execution
pub type QueryResult<T> = Result<T, QueryError>;

#[derive(Debug)]
pub enum QueryError {
    Validation(ValidationError),
    Storage(String),
    Network(String),
    Protocol(String),
}

impl From<ValidationError> for QueryError {
    fn from(e: ValidationError) -> Self {
        Self::Validation(e)
    }
}

// =========================================================================
//  1. The Guardrail (Middleware)
// =========================================================================

/// A Guard intercepts a query before execution to enforce rules.
/// Can be chained.
pub trait QueryGuard<D, M, Q>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    fn check(&self, envelope: &QueryEnvelope<D, M, Q>) -> Result<(), QueryError>;
}

// =========================================================================
//  2. The Protocol (Flow Control)
// =========================================================================

/// Defines the interaction pattern (e.g., Simple RPC, Challenge-Response, Stream)
/// Users can implement this to define custom handshakes or message ordering.
pub trait QueryProtocol {
    type State;
    type Request;
    type Response;

    /// Initialize the protocol state
    fn init(&self) -> Self::State;

    /// Handle an incoming message, potentially transitioning state
    fn on_message(
        &self, 
        state: &mut Self::State, 
        msg: Self::Request
    ) -> QueryResult<Option<Self::Response>>;
}

// =========================================================================
//  3. The Executor (Business Logic)
// =========================================================================

/// Trait for the system that actually runs the query against the DB or Network
pub trait QueryExecutor<D, M, Q>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    type Output;

    fn execute(&self, query: Q) -> QueryResult<Self::Output>;
}
