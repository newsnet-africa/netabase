use libp2p::identify;

pub(super) fn handle_identify_events(event: identify::Event) {
    match event {
        identify::Event::Received {
            connection_id,
            peer_id,
            info,
        } => todo!(),
        identify::Event::Sent {
            connection_id,
            peer_id,
        } => todo!(),
        identify::Event::Pushed {
            connection_id,
            peer_id,
            info,
        } => todo!(),
        identify::Event::Error {
            connection_id,
            peer_id,
            error,
        } => todo!(),
    }
}
