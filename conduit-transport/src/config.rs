/// Transport protocol version embedded in every frame.
pub const TRANSPORT_VERSION: u8 = 1;

/// Fixed transport header size: magic + version + flags + fragment metadata + length.
pub const TRANSPORT_FRAME_HEADER_SIZE: usize = 16;

/// Default MTU for a single on-wire frame (including transport header).
pub const DEFAULT_MTU: usize = 1_500;

/// Default minimum payload size before compression is attempted.
pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 256;

/// Maximum fragments allowed per packet.
pub const MAX_FRAGMENTS: u16 = 256;

use crate::encryption::KEY_SIZE;

/// Runtime configuration for the transport engine.
#[derive(Debug, Clone)]
pub struct TransportConfig {
  pub mtu: usize,
  pub compression_threshold: usize,
  pub enable_compression: bool,
  pub enable_encryption: bool,
  pub encryption_key: Option<[u8; KEY_SIZE]>,
  pub max_fragments: u16,
}

impl TransportConfig {
  pub fn builder() -> TransportConfigBuilder {
    TransportConfigBuilder::default()
  }

  pub fn max_frame_payload(&self) -> usize {
    self.mtu.saturating_sub(TRANSPORT_FRAME_HEADER_SIZE)
  }

  pub fn validate(&self) -> crate::Result<()> {
    if self.mtu <= TRANSPORT_FRAME_HEADER_SIZE {
      return Err(ConduitError::Configuration(
        "mtu must be larger than transport frame header".into(),
      ));
    }
    if self.enable_encryption && self.encryption_key.is_none() {
      return Err(ConduitError::Configuration(
        "encryption enabled but no key configured".into(),
      ));
    }
    Ok(())
  }
}

impl Default for TransportConfig {
  fn default() -> Self {
    TransportConfigBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct TransportConfigBuilder {
  mtu: usize,
  compression_threshold: usize,
  enable_compression: bool,
  enable_encryption: bool,
  encryption_key: Option<[u8; KEY_SIZE]>,
  max_fragments: u16,
}

impl Default for TransportConfigBuilder {
  fn default() -> Self {
    Self {
      mtu: DEFAULT_MTU,
      compression_threshold: DEFAULT_COMPRESSION_THRESHOLD,
      enable_compression: true,
      enable_encryption: false,
      encryption_key: None,
      max_fragments: MAX_FRAGMENTS,
    }
  }
}

impl TransportConfigBuilder {
  pub fn mtu(mut self, mtu: usize) -> Self {
    self.mtu = mtu;
    self
  }

  pub fn compression_threshold(mut self, threshold: usize) -> Self {
    self.compression_threshold = threshold;
    self
  }

  pub fn enable_compression(mut self, enabled: bool) -> Self {
    self.enable_compression = enabled;
    self
  }

  pub fn enable_encryption(mut self, key: [u8; KEY_SIZE]) -> Self {
    self.enable_encryption = true;
    self.encryption_key = Some(key);
    self
  }

  pub fn max_fragments(mut self, max: u16) -> Self {
    self.max_fragments = max;
    self
  }

  pub fn build(self) -> TransportConfig {
    TransportConfig {
      mtu: self.mtu,
      compression_threshold: self.compression_threshold,
      enable_compression: self.enable_compression,
      enable_encryption: self.enable_encryption,
      encryption_key: self.encryption_key,
      max_fragments: self.max_fragments,
    }
  }
}

use conduit_core::ConduitError;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    assert!(TransportConfig::default().validate().is_ok());
  }

  #[test]
  fn encryption_requires_key() {
    let config = TransportConfig {
      enable_encryption: true,
      ..TransportConfig::default()
    };
    assert!(config.validate().is_err());
  }
}
