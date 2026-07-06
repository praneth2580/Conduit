use crate::quality::{LinkQuality, SignalQuality};
use conduit_core::ids::NodeId;
use conduit_discovery::{DiscoveredPeer, PeerEndpoint};
use serde::{Deserialize, Serialize};

/// Broadcast destination for mesh heartbeats.
pub const BROADCAST_NODE_ID: NodeId = NodeId([0u8; 16]);

/// Lifecycle state of a neighbor entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeighborState {
  Discovered,
  Active,
  Stale,
}

/// A nearby node tracked by the mesh engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Neighbor {
  pub node_id: NodeId,
  pub node_name: String,
  pub capabilities: u32,
  pub endpoint: PeerEndpoint,
  pub state: NeighborState,
  pub signal_strength: Option<i8>,
  pub signal_quality: SignalQuality,
  pub link_quality: LinkQuality,
  pub discovered_at_ms: u64,
  pub last_seen_ms: u64,
  pub last_heartbeat_received_ms: Option<u64>,
  pub heartbeats_received: u32,
  pub missed_heartbeats: u32,
}

impl Neighbor {
  pub fn from_discovered(peer: &DiscoveredPeer) -> Self {
    let signal_quality = peer
      .signal_strength
      .map(SignalQuality::from_rssi)
      .unwrap_or(SignalQuality::UNKNOWN);

    Self {
      node_id: peer.node_id,
      node_name: peer.node_name.clone(),
      capabilities: peer.capabilities,
      endpoint: peer.endpoint.clone(),
      state: NeighborState::Discovered,
      signal_strength: peer.signal_strength,
      signal_quality,
      link_quality: LinkQuality::UNKNOWN,
      discovered_at_ms: peer.discovered_at_ms,
      last_seen_ms: peer.last_seen_ms,
      last_heartbeat_received_ms: None,
      heartbeats_received: 0,
      missed_heartbeats: 0,
    }
  }

  pub fn touch(&mut self, now_ms: u64) {
    self.last_seen_ms = now_ms;
  }

  pub fn is_stale(&self, timeout_ms: u64, now_ms: u64) -> bool {
    now_ms.saturating_sub(self.last_seen_ms) > timeout_ms
  }

  pub fn promote_to_active(&mut self) {
    self.state = NeighborState::Active;
  }

  pub fn mark_stale(&mut self) {
    self.state = NeighborState::Stale;
  }

  pub fn record_heartbeat(&mut self, now_ms: u64, alpha: f32) {
    self.last_heartbeat_received_ms = Some(now_ms);
    self.heartbeats_received += 1;
    self.last_seen_ms = now_ms;
    self.link_quality = self.link_quality.record_success(alpha);
    if self.state == NeighborState::Discovered {
      self.promote_to_active();
    }
  }

  pub fn record_missed_heartbeat(&mut self, alpha: f32) {
    self.missed_heartbeats += 1;
    self.link_quality = self.link_quality.record_failure(alpha);
    self.mark_stale();
  }

  pub fn update_signal(&mut self, rssi: i8) {
    self.signal_strength = Some(rssi);
    self.signal_quality = SignalQuality::from_rssi(rssi);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_discovery::DriverKind;

  #[test]
  fn neighbor_from_discovered_peer() {
    let peer = DiscoveredPeer::new(
      NodeId::random(),
      "peer".into(),
      1,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 1 },
    );
    let neighbor = Neighbor::from_discovered(&peer);
    assert_eq!(neighbor.state, NeighborState::Discovered);
  }
}
