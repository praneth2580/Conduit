use conduit_mesh::{Neighbor, NeighborState};
use serde::Serialize;

/// Read-only neighbor summary for application diagnostics.
#[derive(Debug, Clone, Serialize)]
pub struct NeighborInfo {
  pub node_id: String,
  pub name: String,
  pub signal_strength: Option<i8>,
  pub link_quality: f32,
  pub signal_quality: u8,
  pub state: String,
  pub last_seen_ms: u64,
}

impl From<&Neighbor> for NeighborInfo {
  fn from(neighbor: &Neighbor) -> Self {
    Self {
      node_id: neighbor.node_id.to_string(),
      name: neighbor.node_name.clone(),
      signal_strength: neighbor.signal_strength,
      link_quality: neighbor.link_quality.value(),
      signal_quality: neighbor.signal_quality.value(),
      state: match neighbor.state {
        NeighborState::Discovered => "discovered",
        NeighborState::Active => "active",
        NeighborState::Stale => "stale",
      }
      .into(),
      last_seen_ms: neighbor.last_seen_ms,
    }
  }
}

/// Snapshot of node and mesh state for debugging UIs.
#[derive(Debug, Clone, Serialize)]
pub struct NodeDiagnostics {
  pub node_id: String,
  pub node_name: String,
  pub joined: bool,
  pub neighbor_count: usize,
  pub neighbors: Vec<NeighborInfo>,
  pub route_count: usize,
  pub discovery_peer_count: usize,
  /// Active discovery backend (e.g. `udp_broadcast`, `wifi_aware`).
  pub discovery_driver: String,
}
