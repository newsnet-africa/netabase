#![feature(duration_constructors_lite)]
#![feature(mpmc_channel)]
pub mod database;
pub mod network;

/// Initialize logging for tests across the library.
/// This function ensures that logging is initialized only once using `std::sync::Once`
/// to prevent multiple initialization attempts when running multiple tests.
///
/// The logger is configured with:
/// - Test mode enabled (`.is_test(true)`)
/// - Info level filtering to show relevant debug information during tests
/// - Graceful handling of re-initialization attempts
#[cfg(test)]
pub fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

#[cfg(test)]
mod test_network {
    use libp2p::futures::StreamExt;

    use crate::network::{
        behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent},
        swarm::{generate_swarm, handle_event},
    };

    #[tokio::test]
    async fn test_swarm() {
        crate::init_logging();
        let mut swarm = generate_swarm("./tmp1").expect("Swarm Generation erruh");
        let (tx, _rx) = tokio::sync::broadcast::channel::<NetabaseBehaviourEvent>(1);
        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse Erruh"))
            .expect("Listen Erruh");
        handle_event(&mut swarm, tx).await;
    }
}
