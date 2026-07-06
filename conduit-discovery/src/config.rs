/// Magic bytes for discovery beacons on the wire.
pub const BEACON_MAGIC: [u8; 4] = *b"CDSC";

/// Beacon protocol version.
pub const BEACON_VERSION: u8 = 1;

/// Default UDP port for LAN discovery broadcasts.
pub const DEFAULT_DISCOVERY_PORT: u16 = 4_219;

/// Default interval between discovery broadcasts in milliseconds.
pub const DEFAULT_ANNOUNCE_INTERVAL_MS: u64 = 3_000;

/// Peers not seen within this window are considered lost.
pub const DEFAULT_PEER_TIMEOUT_MS: u64 = 12_000;

use conduit_core::ids::NodeId;
use serde::{Deserialize, Serialize};

/// Runtime configuration for the discovery engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryConfig {
  pub node_id: NodeId,
  pub node_name: String,
  pub capabilities: u32,
  pub announce_interval_ms: u64,
  pub peer_timeout_ms: u64,
  pub udp_port: u16,
  pub enable_udp_broadcast: bool,
}

impl DiscoveryConfig {
  pub fn builder() -> DiscoveryConfigBuilder {
    DiscoveryConfigBuilder::default()
  }

  pub fn announcement(&self) -> crate::DiscoveryAnnouncement {
    crate::DiscoveryAnnouncement {
      node_id: self.node_id,
      node_name: self.node_name.clone(),
      capabilities: self.capabilities,
    }
  }

  pub fn validate(&self) -> crate::Result<()> {
    if self.node_name.is_empty() {
      return Err(ConduitError::Configuration(
        "node_name must not be empty".into(),
      ));
    }
    if self.announce_interval_ms == 0 {
      return Err(ConduitError::Configuration(
        "announce_interval_ms must be greater than zero".into(),
      ));
    }
    if self.peer_timeout_ms < self.announce_interval_ms {
      return Err(ConduitError::Configuration(
        "peer_timeout_ms must be >= announce_interval_ms".into(),
      ));
    }
    Ok(())
  }
}

impl Default for DiscoveryConfig {
  fn default() -> Self {
    DiscoveryConfigBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfigBuilder {
  node_id: Option<NodeId>,
  node_name: String,
  capabilities: u32,
  announce_interval_ms: u64,
  peer_timeout_ms: u64,
  udp_port: u16,
  enable_udp_broadcast: bool,
}

impl Default for DiscoveryConfigBuilder {
  fn default() -> Self {
    Self {
      node_id: None,
      node_name: "conduit-node".into(),
      capabilities: crate::peer::capabilities::VOICE | crate::peer::capabilities::RELAY,
      announce_interval_ms: DEFAULT_ANNOUNCE_INTERVAL_MS,
      peer_timeout_ms: DEFAULT_PEER_TIMEOUT_MS,
      udp_port: DEFAULT_DISCOVERY_PORT,
      enable_udp_broadcast: true,
    }
  }
}

impl DiscoveryConfigBuilder {
  pub fn node_id(mut self, id: NodeId) -> Self {
    self.node_id = Some(id);
    self
  }

  pub fn node_name(mut self, name: impl Into<String>) -> Self {
    self.node_name = name.into();
    self
  }

  pub fn capabilities(mut self, caps: u32) -> Self {
    self.capabilities = caps;
    self
  }

  pub fn announce_interval_ms(mut self, ms: u64) -> Self {
    self.announce_interval_ms = ms;
    self
  }

  pub fn peer_timeout_ms(mut self, ms: u64) -> Self {
    self.peer_timeout_ms = ms;
    self
  }

  pub fn udp_port(mut self, port: u16) -> Self {
    self.udp_port = port;
    self
  }

  pub fn enable_udp_broadcast(mut self, enabled: bool) -> Self {
    self.enable_udp_broadcast = enabled;
    self
  }

  pub fn build(self) -> DiscoveryConfig {
    DiscoveryConfig {
      node_id: self.node_id.unwrap_or_default(),
      node_name: self.node_name,
      capabilities: self.capabilities,
      announce_interval_ms: self.announce_interval_ms,
      peer_timeout_ms: self.peer_timeout_ms,
      udp_port: self.udp_port,
      enable_udp_broadcast: self.enable_udp_broadcast,
    }
  }
}

use conduit_core::ConduitError;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    assert!(DiscoveryConfig::default().validate().is_ok());
  }
}
