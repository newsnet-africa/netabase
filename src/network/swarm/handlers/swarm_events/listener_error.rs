use libp2p::core::transport::ListenerId;
use netabase_store::traits::definition::NetabaseDefinitionTrait;

pub fn handle_listener_error<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    listener_id: ListenerId,
    error: std::io::Error,
) where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,{
    // TODO: Implement listener error handling
    println!(
        "Listener error: listener_id: {:?}, error: {:?}",
        listener_id, error
    );

    // Log the specific error type for debugging
    eprintln!("Listener {:?} encountered error: {}", listener_id, error);
}
