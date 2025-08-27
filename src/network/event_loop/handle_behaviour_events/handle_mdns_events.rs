use libp2p::mdns;

pub fn handle_mdns_event(event: mdns::Event) {
    println!("MDNS Event: {event:?}");
    match event {
        mdns::Event::Discovered(items) => {}
        mdns::Event::Expired(items) => {}
    }
}
