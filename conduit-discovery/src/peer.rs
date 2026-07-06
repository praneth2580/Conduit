use crate::driver::{DriverKind, PeerEndpoint};
use conduit_core::ids::NodeId;
use conduit_core::utils::unix_timestamp_ms;
use serde::{Deserialize, Serialize};

/// Capability flags advertised during discovery.
pub mod capabilities {
  pub const VOICE: u32 = 1 << 0;
  pub const MESSAGING: u32 = 1 << 1;
  pub const LOCATION: u32 = 1 << 2;
  pub const RELAY: u32 = 1 << 3;
  pub const EMERGENCY: u32 = 1 << 4;
}

/// A peer discovered nearby.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredPeer {
  pub node_id: NodeId,
  pub node_name: String,
  pub capabilities: u32,
  pub driver: DriverKind,
  pub endpoint: PeerEndpoint,
  pub signal_strength: Option<i8>,
  pub discovered_at_ms: u64,
  pub last_seen_ms: u64,
}

impl DiscoveredPeer {
  pub fn new(
    node_id: NodeId,
    node_name: String,
    capabilities: u32,
    driver: DriverKind,
    endpoint: PeerEndpoint,
  ) -> Self {
    let now = unix_timestamp_ms();
    Self {
      node_id,
      node_name,
      capabilities,
      driver,
      endpoint,
      signal_strength: None,
      discovered_at_ms: now,
      last_seen_ms: now,
    }
  }

  pub fn touch(&mut self) {
    self.last_seen_ms = unix_timestamp_ms();
  }

  pub fn is_stale(&self, timeout_ms: u64, now_ms: u64) -> bool {
    now_ms.saturating_sub(self.last_seen_ms) > timeout_ms
  }

  pub fn supports_voice(&self) -> bool {
    self.capabilities & capabilities::VOICE != 0
  }

  pub fn supports_relay(&self) -> bool {
    self.capabilities & capabilities::RELAY != 0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn capability_flags() {
    let peer = DiscoveredPeer::new(
      NodeId::random(),
      "peer".into(),
      capabilities::VOICE | capabilities::RELAY,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 1 },
    );
    assert!(peer.supports_voice());
    assert!(peer.supports_relay());
    assert!(!peer.is_stale(5_000, peer.last_seen_ms + 4_000));
    assert!(peer.is_stale(5_000, peer.last_seen_ms + 6_000));
  }
}
