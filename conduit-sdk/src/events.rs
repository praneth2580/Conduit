use conduit_core::NodeId;
use conduit_mesh::Neighbor;
use conduit_packets::{EmergencyPayload, LocationPayload, MessagingPayload};

/// High-level events surfaced to applications.
#[derive(Debug, Clone, PartialEq)]
pub enum SdkEvent {
  NetworkJoined,
  NetworkLeft,
  PeerDiscovered {
    node_id: NodeId,
    name: String,
  },
  PeerLost {
    node_id: NodeId,
  },
  NeighborAdded(Neighbor),
  NeighborRemoved {
    node_id: NodeId,
  },
  LocationReceived {
    from: NodeId,
    location: LocationPayload,
  },
  EmergencyReceived {
    from: NodeId,
    emergency: EmergencyPayload,
  },
  MessageReceived {
    from: NodeId,
    message: MessagingPayload,
  },
  VoiceFrameReceived {
    from: NodeId,
  },
}
