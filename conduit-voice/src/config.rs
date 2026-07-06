/// Default sample rate (Opus native rate).
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;

/// Default frame duration in milliseconds.
pub const DEFAULT_FRAME_MS: u16 = 20;

/// Default Opus bitrate in bits per second.
pub const DEFAULT_BITRATE: i32 = 32_000;

/// Default jitter buffer target delay in milliseconds.
pub const DEFAULT_JITTER_MS: u32 = 60;

/// Samples per frame at 48 kHz / 20 ms.
pub const DEFAULT_FRAME_SAMPLES: usize = 960;

use serde::{Deserialize, Serialize};

/// Runtime configuration for the voice engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceConfig {
  pub local_node_id: NodeId,
  pub sample_rate: u32,
  pub frame_duration_ms: u16,
  pub bitrate: i32,
  pub vad_energy_threshold: f32,
  pub jitter_buffer_ms: u32,
  pub push_to_talk: bool,
}

impl VoiceConfig {
  pub fn builder() -> VoiceConfigBuilder {
    VoiceConfigBuilder::default()
  }

  pub fn frame_samples(&self) -> usize {
    (self.sample_rate as usize * self.frame_duration_ms as usize) / 1000
  }

  pub fn validate(&self) -> crate::Result<()> {
    if self.sample_rate == 0 {
      return Err(conduit_core::ConduitError::Configuration(
        "sample_rate must be greater than zero".into(),
      ));
    }
    if self.frame_duration_ms == 0 {
      return Err(conduit_core::ConduitError::Configuration(
        "frame_duration_ms must be greater than zero".into(),
      ));
    }
    if self.frame_samples() == 0 {
      return Err(conduit_core::ConduitError::Configuration(
        "frame_samples must be greater than zero".into(),
      ));
    }
    Ok(())
  }
}

impl Default for VoiceConfig {
  fn default() -> Self {
    VoiceConfigBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct VoiceConfigBuilder {
  local_node_id: Option<NodeId>,
  sample_rate: u32,
  frame_duration_ms: u16,
  bitrate: i32,
  vad_energy_threshold: f32,
  jitter_buffer_ms: u32,
  push_to_talk: bool,
}

impl Default for VoiceConfigBuilder {
  fn default() -> Self {
    Self {
      local_node_id: None,
      sample_rate: DEFAULT_SAMPLE_RATE,
      frame_duration_ms: DEFAULT_FRAME_MS,
      bitrate: DEFAULT_BITRATE,
      vad_energy_threshold: 0.01,
      jitter_buffer_ms: DEFAULT_JITTER_MS,
      push_to_talk: false,
    }
  }
}

impl VoiceConfigBuilder {
  pub fn local_node_id(mut self, id: NodeId) -> Self {
    self.local_node_id = Some(id);
    self
  }

  pub fn push_to_talk(mut self, enabled: bool) -> Self {
    self.push_to_talk = enabled;
    self
  }

  pub fn vad_energy_threshold(mut self, threshold: f32) -> Self {
    self.vad_energy_threshold = threshold;
    self
  }

  pub fn jitter_buffer_ms(mut self, ms: u32) -> Self {
    self.jitter_buffer_ms = ms;
    self
  }

  pub fn build(self) -> VoiceConfig {
    VoiceConfig {
      local_node_id: self.local_node_id.unwrap_or_default(),
      sample_rate: self.sample_rate,
      frame_duration_ms: self.frame_duration_ms,
      bitrate: self.bitrate,
      vad_energy_threshold: self.vad_energy_threshold,
      jitter_buffer_ms: self.jitter_buffer_ms,
      push_to_talk: self.push_to_talk,
    }
  }
}

use conduit_core::ids::NodeId;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_is_valid() {
    let config = VoiceConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.frame_samples(), DEFAULT_FRAME_SAMPLES);
  }
}
