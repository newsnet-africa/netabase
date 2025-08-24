use libp2p::kad;

pub(super) fn handle_kad_events(kad_event: kad::Event) {
    match kad_event {
        kad::Event::InboundRequest { request } => todo!(),
        kad::Event::OutboundQueryProgressed {
            id,
            result,
            stats,
            step,
        } => todo!(),
        kad::Event::RoutingUpdated {
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer,
        } => todo!(),
        kad::Event::UnroutablePeer { peer } => todo!(),
        kad::Event::RoutablePeer { peer, address } => todo!(),
        kad::Event::PendingRoutablePeer { peer, address } => todo!(),
        kad::Event::ModeChanged { new_mode } => todo!(),
    }
}
