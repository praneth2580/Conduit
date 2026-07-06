use crate::constants::{DEFAULT_HEARTBEAT_INTERVAL_MS, DEFAULT_NEIGHBOR_TIMEOUT_MS, DEFAULT_TTL};
use crate::ids::NodeId;
use crate::logging::LogLevel;
use crate::version::ProtocolVersion;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Runtime configuration for a Conduit node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConduitConfig {
  pub node_id: NodeId,
  pub protocol_version: ProtocolVersion,
  pub log_level: LogLevel,
  pub default_ttl: u8,
  pub heartbeat_interval_ms: u64,
  pub neighbor_timeout_ms: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config_path: Option<PathBuf>,
}

impl ConduitConfig {
  pub fn builder() -> ConduitConfigBuilder {
    ConduitConfigBuilder::default()
  }

  pub fn validate(&self) -> crate::error::Result<()> {
    if self.default_ttl == 0 {
      return Err(crate::error::ConduitError::Configuration(
        "default_ttl must be greater than zero".into(),
      ));
    }
    if self.heartbeat_interval_ms == 0 {
      return Err(crate::error::ConduitError::Configuration(
        "heartbeat_interval_ms must be greater than zero".into(),
      ));
    }
    if self.neighbor_timeout_ms < self.heartbeat_interval_ms {
      return Err(crate::error::ConduitError::Configuration(
        "neighbor_timeout_ms must be >= heartbeat_interval_ms".into(),
      ));
    }
    Ok(())
  }

  pub fn to_json(&self) -> crate::error::Result<String> {
    serde_json::to_string_pretty(self)
      .map_err(|e| crate::error::ConduitError::Serialization(e.to_string()))
  }

  pub fn from_json(json: &str) -> crate::error::Result<Self> {
    let config: Self = serde_json::from_str(json)
      .map_err(|e| crate::error::ConduitError::Deserialization(e.to_string()))?;
    config.validate()?;
    Ok(config)
  }

  pub fn load_from_file(path: impl Into<PathBuf>) -> crate::error::Result<Self> {
    let path = path.into();
    let contents = std::fs::read_to_string(&path)?;
    let mut config = Self::from_json(&contents)?;
    config.config_path = Some(path);
    Ok(config)
  }

  pub fn save_to_file(&self, path: impl Into<PathBuf>) -> crate::error::Result<()> {
    let path = path.into();
    std::fs::write(&path, self.to_json()?)?;
    Ok(())
  }
}

impl Default for ConduitConfig {
  fn default() -> Self {
    ConduitConfigBuilder::default().build()
  }
}

/// Builder for [`ConduitConfig`].
#[derive(Debug, Clone)]
pub struct ConduitConfigBuilder {
  node_id: Option<NodeId>,
  protocol_version: ProtocolVersion,
  log_level: LogLevel,
  default_ttl: u8,
  heartbeat_interval_ms: u64,
  neighbor_timeout_ms: u64,
}

impl Default for ConduitConfigBuilder {
  fn default() -> Self {
    Self {
      node_id: None,
      protocol_version: ProtocolVersion::CURRENT,
      log_level: LogLevel::Info,
      default_ttl: DEFAULT_TTL,
      heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
      neighbor_timeout_ms: DEFAULT_NEIGHBOR_TIMEOUT_MS,
    }
  }
}

impl ConduitConfigBuilder {
  pub fn node_id(mut self, id: NodeId) -> Self {
    self.node_id = Some(id);
    self
  }

  pub fn protocol_version(mut self, version: ProtocolVersion) -> Self {
    self.protocol_version = version;
    self
  }

  pub fn log_level(mut self, level: LogLevel) -> Self {
    self.log_level = level;
    self
  }

  pub fn default_ttl(mut self, ttl: u8) -> Self {
    self.default_ttl = ttl;
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

  pub fn build(self) -> ConduitConfig {
    ConduitConfig {
      node_id: self.node_id.unwrap_or_default(),
      protocol_version: self.protocol_version,
      log_level: self.log_level,
      default_ttl: self.default_ttl,
      heartbeat_interval_ms: self.heartbeat_interval_ms,
      neighbor_timeout_ms: self.neighbor_timeout_ms,
      config_path: None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    let config = ConduitConfig::default();
    assert!(config.validate().is_ok());
  }

  #[test]
  fn rejects_invalid_ttl() {
    let config = ConduitConfig::builder().default_ttl(0).build();
    assert!(config.validate().is_err());
  }

  #[test]
  fn json_round_trip() {
    let config = ConduitConfig::builder()
      .log_level(LogLevel::Debug)
      .build();
    let json = config.to_json().unwrap();
    let restored = ConduitConfig::from_json(&json).unwrap();
    assert_eq!(config.node_id, restored.node_id);
    assert_eq!(config.log_level, restored.log_level);
  }

  #[test]
  fn file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("conduit.json");
    let config = ConduitConfig::default();
    config.save_to_file(&path).unwrap();
    let loaded = ConduitConfig::load_from_file(&path).unwrap();
    assert_eq!(config.node_id, loaded.node_id);
    assert_eq!(loaded.config_path, Some(path));
  }
}
