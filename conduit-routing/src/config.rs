/// Default duplicate detection window in milliseconds.
pub const DEFAULT_DUPLICATE_WINDOW_MS: u64 = 30_000;

/// Maximum entries in the duplicate cache.
pub const DEFAULT_DUPLICATE_CACHE_SIZE: usize = 4_096;

/// Route entries older than this are considered stale.
pub const DEFAULT_ROUTE_STALE_MS: u64 = 60_000;

/// Maximum in-flight packets per neighbor before congestion backoff.
pub const DEFAULT_CONGESTION_LIMIT: u16 = 32;

use conduit_core::ids::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingConfig {
  pub local_node_id: NodeId,
  pub duplicate_window_ms: u64,
  pub duplicate_cache_size: usize,
  pub route_stale_ms: u64,
  pub congestion_limit: u16,
  pub enable_flood_fallback: bool,
}

impl RoutingConfig {
  pub fn builder() -> RoutingConfigBuilder {
    RoutingConfigBuilder::default()
  }

  pub fn validate(&self) -> crate::Result<()> {
    if self.duplicate_window_ms == 0 {
      return Err(conduit_core::ConduitError::Configuration(
        "duplicate_window_ms must be greater than zero".into(),
      ));
    }
    if self.duplicate_cache_size == 0 {
      return Err(conduit_core::ConduitError::Configuration(
        "duplicate_cache_size must be greater than zero".into(),
      ));
    }
    Ok(())
  }
}

impl Default for RoutingConfig {
  fn default() -> Self {
    RoutingConfigBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct RoutingConfigBuilder {
  local_node_id: Option<NodeId>,
  duplicate_window_ms: u64,
  duplicate_cache_size: usize,
  route_stale_ms: u64,
  congestion_limit: u16,
  enable_flood_fallback: bool,
}

impl Default for RoutingConfigBuilder {
  fn default() -> Self {
    Self {
      local_node_id: None,
      duplicate_window_ms: DEFAULT_DUPLICATE_WINDOW_MS,
      duplicate_cache_size: DEFAULT_DUPLICATE_CACHE_SIZE,
      route_stale_ms: DEFAULT_ROUTE_STALE_MS,
      congestion_limit: DEFAULT_CONGESTION_LIMIT,
      enable_flood_fallback: true,
    }
  }
}

impl RoutingConfigBuilder {
  pub fn local_node_id(mut self, id: NodeId) -> Self {
    self.local_node_id = Some(id);
    self
  }

  pub fn duplicate_window_ms(mut self, ms: u64) -> Self {
    self.duplicate_window_ms = ms;
    self
  }

  pub fn congestion_limit(mut self, limit: u16) -> Self {
    self.congestion_limit = limit;
    self
  }

  pub fn enable_flood_fallback(mut self, enabled: bool) -> Self {
    self.enable_flood_fallback = enabled;
    self
  }

  pub fn build(self) -> RoutingConfig {
    RoutingConfig {
      local_node_id: self.local_node_id.unwrap_or_default(),
      duplicate_window_ms: self.duplicate_window_ms,
      duplicate_cache_size: self.duplicate_cache_size,
      route_stale_ms: self.route_stale_ms,
      congestion_limit: self.congestion_limit,
      enable_flood_fallback: self.enable_flood_fallback,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    assert!(RoutingConfig::default().validate().is_ok());
  }
}
