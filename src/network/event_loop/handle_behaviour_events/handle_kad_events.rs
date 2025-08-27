use libp2p::kad;

pub fn handle_kad_event(event: kad::Event) {
    println!("Kademlia Event: {event:?}");
    match event {
        kad::Event::InboundRequest { request } => {}
        kad::Event::OutboundQueryProgressed {
            id,
            result,
            stats,
            step,
        } => {}
        kad::Event::RoutingUpdated {
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer,
        } => {}
        kad::Event::UnroutablePeer { peer } => {}
        kad::Event::RoutablePeer { peer, address } => {}
        kad::Event::PendingRoutablePeer { peer, address } => {}
        kad::Event::ModeChanged { new_mode } => {}
    }
}
