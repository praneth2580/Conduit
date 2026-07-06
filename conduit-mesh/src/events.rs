use crate::neighbor::Neighbor;
use conduit_core::ids::NodeId;
use conduit_packets::TypedPacket;

/// Why a neighbor was removed from the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborRemovalReason {
  DiscoveryLost,
  TimedOut,
  LowLinkQuality,
  Stopped,
}

/// Events emitted by the mesh engine.
#[derive(Debug, Clone, PartialEq)]
pub enum MeshEvent {
  NeighborAdded(Neighbor),
  NeighborUpdated {
    node_id: NodeId,
    link_quality: f32,
    signal_quality: u8,
    state: crate::neighbor::NeighborState,
  },
  NeighborRemoved {
    node_id: NodeId,
    reason: NeighborRemovalReason,
  },
  HeartbeatReady(TypedPacket),
}

/// Result of a single mesh tick.
#[derive(Debug, Default)]
pub struct MeshTick {
  pub events: Vec<MeshEvent>,
}
