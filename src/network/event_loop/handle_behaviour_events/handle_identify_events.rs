use libp2p::identify;

pub fn handle_identify_event(event: identify::Event) {
    println!("Identify Event: {event:?}");
    match event {
        identify::Event::Received {
            connection_id,
            peer_id,
            info,
        } => {}
        identify::Event::Sent {
            connection_id,
            peer_id,
        } => {}
        identify::Event::Pushed {
            connection_id,
            peer_id,
            info,
        } => {}
        identify::Event::Error {
            connection_id,
            peer_id,
            error,
        } => {}
    }
}
