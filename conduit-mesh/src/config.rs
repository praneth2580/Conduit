/// Default heartbeat interval in milliseconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 5_000;

/// Neighbor considered stale after this many milliseconds without contact.
pub const DEFAULT_NEIGHBOR_TIMEOUT_MS: u64 = 15_000;

/// EWMA smoothing factor for link quality (0.0–1.0).
pub const DEFAULT_LINK_QUALITY_ALPHA: f32 = 0.3;

use conduit_core::ids::NodeId;
use serde::{Deserialize, Serialize};

/// Runtime configuration for the mesh engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshConfig {
  pub local_node_id: NodeId,
  pub heartbeat_interval_ms: u64,
  pub neighbor_timeout_ms: u64,
  pub link_quality_alpha: f32,
  /// Neighbors below this link quality are removed during tick.
  pub min_link_quality: f32,
}

impl MeshConfig {
  pub fn builder() -> MeshConfigBuilder {
    MeshConfigBuilder::default()
  }

  pub fn validate(&self) -> crate::Result<()> {
    if self.heartbeat_interval_ms == 0 {
      return Err(ConduitError::Configuration(
        "heartbeat_interval_ms must be greater than zero".into(),
      ));
    }
    if self.neighbor_timeout_ms < self.heartbeat_interval_ms {
      return Err(ConduitError::Configuration(
        "neighbor_timeout_ms must be >= heartbeat_interval_ms".into(),
      ));
    }
    if !(0.0..=1.0).contains(&self.link_quality_alpha) {
      return Err(ConduitError::Configuration(
        "link_quality_alpha must be between 0.0 and 1.0".into(),
      ));
    }
    if !(0.0..=1.0).contains(&self.min_link_quality) {
      return Err(ConduitError::Configuration(
        "min_link_quality must be between 0.0 and 1.0".into(),
      ));
    }
    Ok(())
  }
}

impl Default for MeshConfig {
  fn default() -> Self {
    MeshConfigBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct MeshConfigBuilder {
  local_node_id: Option<NodeId>,
  heartbeat_interval_ms: u64,
  neighbor_timeout_ms: u64,
  link_quality_alpha: f32,
  min_link_quality: f32,
}

impl Default for MeshConfigBuilder {
  fn default() -> Self {
    Self {
      local_node_id: None,
      heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
      neighbor_timeout_ms: DEFAULT_NEIGHBOR_TIMEOUT_MS,
      link_quality_alpha: DEFAULT_LINK_QUALITY_ALPHA,
      min_link_quality: 0.1,
    }
  }
}

impl MeshConfigBuilder {
  pub fn local_node_id(mut self, id: NodeId) -> Self {
    self.local_node_id = Some(id);
    self
  }

  pub fn heartbeat_interval_ms(mut self, ms: u64) -> Self {
    self.heartbeat_interval_ms = ms;
    self
  }

  pub fn neighbor_timeout_ms(mut self, ms: u64) -> Self {
    self.neighbor_timeout_ms = ms;
    self
  }

  pub fn link_quality_alpha(mut self, alpha: f32) -> Self {
    self.link_quality_alpha = alpha;
    self
  }

  pub fn min_link_quality(mut self, quality: f32) -> Self {
    self.min_link_quality = quality;
    self
  }

  pub fn build(self) -> MeshConfig {
    MeshConfig {
      local_node_id: self.local_node_id.unwrap_or_default(),
      heartbeat_interval_ms: self.heartbeat_interval_ms,
      neighbor_timeout_ms: self.neighbor_timeout_ms,
      link_quality_alpha: self.link_quality_alpha,
      min_link_quality: self.min_link_quality,
    }
  }
}

use conduit_core::ConduitError;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    assert!(MeshConfig::default().validate().is_ok());
  }
}
