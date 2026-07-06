/// Magic bytes for secure envelopes on the wire.
pub const ENVELOPE_MAGIC: [u8; 4] = *b"CDSE";

/// Secure envelope protocol version.
pub const SECURITY_VERSION: u8 = 1;

/// Default replay detection window size per peer.
pub const DEFAULT_REPLAY_WINDOW: u64 = 256;

/// Maximum clock skew allowed when validating timestamps (milliseconds).
pub const DEFAULT_MAX_CLOCK_SKEW_MS: u64 = 120_000;

use conduit_core::ids::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityConfig {
  pub local_node_id: NodeId,
  pub replay_window: u64,
  pub max_clock_skew_ms: u64,
  pub require_known_peers: bool,
}

impl SecurityConfig {
  pub fn builder() -> SecurityConfigBuilder {
    SecurityConfigBuilder::default()
  }

  pub fn validate(&self) -> crate::Result<()> {
    if self.replay_window == 0 {
      return Err(ConduitError::Configuration(
        "replay_window must be greater than zero".into(),
      ));
    }
    Ok(())
  }
}

impl Default for SecurityConfig {
  fn default() -> Self {
    SecurityConfigBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct SecurityConfigBuilder {
  local_node_id: Option<NodeId>,
  replay_window: u64,
  max_clock_skew_ms: u64,
  require_known_peers: bool,
}

impl Default for SecurityConfigBuilder {
  fn default() -> Self {
    Self {
      local_node_id: None,
      replay_window: DEFAULT_REPLAY_WINDOW,
      max_clock_skew_ms: DEFAULT_MAX_CLOCK_SKEW_MS,
      require_known_peers: false,
    }
  }
}

impl SecurityConfigBuilder {
  pub fn local_node_id(mut self, id: NodeId) -> Self {
    self.local_node_id = Some(id);
    self
  }

  pub fn replay_window(mut self, window: u64) -> Self {
    self.replay_window = window;
    self
  }

  pub fn require_known_peers(mut self, required: bool) -> Self {
    self.require_known_peers = required;
    self
  }

  pub fn build(self) -> SecurityConfig {
    SecurityConfig {
      local_node_id: self.local_node_id.unwrap_or_default(),
      replay_window: self.replay_window,
      max_clock_skew_ms: self.max_clock_skew_ms,
      require_known_peers: self.require_known_peers,
    }
  }
}

use conduit_core::ConduitError;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    assert!(SecurityConfig::default().validate().is_ok());
  }
}
