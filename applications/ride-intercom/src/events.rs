use crate::member::{RideMember, RidePosition};
use conduit_sdk::{EmergencyPayload, LocationPayload, NodeId};

/// Application-level events for ride intercom UIs.
#[derive(Debug, Clone, PartialEq)]
pub enum RideEvent {
  SessionStarted { group: String },
  SessionStopped,
  MemberJoined(RideMember),
  MemberLeft { node_id: NodeId, name: String },
  MemberLocationUpdated {
    node_id: NodeId,
    location: LocationPayload,
  },
  VoiceReceived { from: NodeId },
  SosReceived {
    from: NodeId,
    emergency: EmergencyPayload,
  },
  TextMessage { from: NodeId, content: String },
}

impl RideEvent {
  pub fn member_name<'a>(&'a self, members: &'a [RideMember]) -> Option<&'a str> {
    let id = match self {
      Self::MemberJoined(m) => return Some(&m.name),
      Self::MemberLeft { name, .. } => return Some(name),
      Self::MemberLocationUpdated { node_id, .. }
      | Self::VoiceReceived { from: node_id }
      | Self::SosReceived { from: node_id, .. }
      | Self::TextMessage { from: node_id, .. } => node_id,
      _ => return None,
    };
    members
      .iter()
      .find(|m| m.node_id == *id)
      .map(|m| m.name.as_str())
  }
}

/// Last-known rider position used for periodic GPS sharing.
#[derive(Debug, Clone, Copy)]
pub struct SharedPosition(pub RidePosition);
