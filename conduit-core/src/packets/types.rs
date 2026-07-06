use serde::{Deserialize, Serialize};

/// All supported Conduit packet types.
///
/// New types can be added without changing routing (per README extensibility goal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
  Voice = 1,
  Location = 2,
  Heartbeat = 3,
  Emergency = 4,
  Messaging = 5,
  Discovery = 6,
  Routing = 7,
  Telemetry = 8,
  Control = 9,
}

impl PacketType {
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      1 => Some(Self::Voice),
      2 => Some(Self::Location),
      3 => Some(Self::Heartbeat),
      4 => Some(Self::Emergency),
      5 => Some(Self::Messaging),
      6 => Some(Self::Discovery),
      7 => Some(Self::Routing),
      8 => Some(Self::Telemetry),
      9 => Some(Self::Control),
      _ => None,
    }
  }

  pub fn name(self) -> &'static str {
    match self {
      Self::Voice => "voice",
      Self::Location => "location",
      Self::Heartbeat => "heartbeat",
      Self::Emergency => "emergency",
      Self::Messaging => "messaging",
      Self::Discovery => "discovery",
      Self::Routing => "routing",
      Self::Telemetry => "telemetry",
      Self::Control => "control",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip_all_types() {
    let types = [
      PacketType::Voice,
      PacketType::Location,
      PacketType::Heartbeat,
      PacketType::Emergency,
      PacketType::Messaging,
      PacketType::Discovery,
      PacketType::Routing,
      PacketType::Telemetry,
      PacketType::Control,
    ];
    for t in types {
      assert_eq!(PacketType::from_u8(t as u8), Some(t));
    }
  }
}
