use std::time::Duration;

pub struct NetabaseConfig {
    dht_discovery: DHTDiscoveryConfig,
}

pub struct DHTDiscoveryConfig {
    mdns_discovery: MDNSDiscoveryConfig,
}

pub struct MDNSDiscoveryConfig {
    auto_connect: Option<Duration>,
}
