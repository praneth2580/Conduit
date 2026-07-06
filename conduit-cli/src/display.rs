use conduit_sdk::{EmergencyKind, NodeId, SdkEvent};
use ride_intercom::RideEvent;

pub fn print_sdk_event(event: &SdkEvent) {
  match event {
    SdkEvent::NetworkJoined => println!("network joined"),
    SdkEvent::NetworkLeft => println!("network left"),
    SdkEvent::PeerDiscovered { node_id, name } => {
      println!("peer discovered: {name} ({node_id})");
    }
    SdkEvent::PeerLost { node_id } => println!("peer lost: {node_id}"),
    SdkEvent::NeighborAdded(n) => {
      println!("neighbor added: {} ({})", n.node_name, n.node_id);
    }
    SdkEvent::NeighborRemoved { node_id } => println!("neighbor removed: {node_id}"),
    SdkEvent::LocationReceived { from, location } => {
      println!(
        "location from {from}: lat={} lon={} alt={}m accuracy={}m",
        location.latitude_microdeg,
        location.longitude_microdeg,
        location.altitude_m,
        location.accuracy_m
      );
    }
    SdkEvent::EmergencyReceived { from, emergency } => {
      println!(
        "SOS from {from}: {} — {}",
        emergency_kind_label(emergency.kind),
        emergency.message
      );
    }
    SdkEvent::MessageReceived { from, message } => {
      println!("message from {from}: {}", message.content);
    }
    SdkEvent::VoiceFrameReceived { from } => println!("voice frame from {from}"),
  }
}

pub fn print_ride_event(event: &RideEvent) {
  match event {
    RideEvent::SessionStarted { group } => println!("ride session started: {group}"),
    RideEvent::SessionStopped => println!("ride session stopped"),
    RideEvent::MemberJoined(m) => println!("rider joined: {} ({})", m.name, m.node_id),
    RideEvent::MemberLeft { node_id, name } => println!("rider left: {name} ({node_id})"),
    RideEvent::MemberLocationUpdated { node_id, location } => {
      println!(
        "rider {node_id} location: lat={} lon={}",
        location.latitude_microdeg, location.longitude_microdeg
      );
    }
    RideEvent::VoiceReceived { from } => println!("voice from rider {from}"),
    RideEvent::SosReceived { from, emergency } => {
      println!("SOS from rider {from}: {}", emergency.message);
    }
    RideEvent::TextMessage { from, content } => {
      println!("text from rider {from}: {content}");
    }
  }
}

pub fn short_node_id(id: &NodeId) -> String {
  let bytes = id.as_bytes();
  format!(
    "{:02x}{:02x}{:02x}{:02x}",
    bytes[0], bytes[1], bytes[2], bytes[3]
  )
}

fn emergency_kind_label(kind: EmergencyKind) -> &'static str {
  match kind {
    EmergencyKind::Medical => "medical",
    EmergencyKind::Mechanical => "mechanical",
    EmergencyKind::Lost => "lost",
    EmergencyKind::General => "general",
  }
}
