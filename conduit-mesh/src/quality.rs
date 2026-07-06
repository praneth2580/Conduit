/// Normalized signal strength on a 0–100 scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalQuality(pub u8);

impl SignalQuality {
  pub const MIN_RSSI: i8 = -90;
  pub const MAX_RSSI: i8 = -30;
  pub const UNKNOWN: Self = Self(50);

  pub fn from_rssi(rssi: i8) -> Self {
    let clamped = rssi.clamp(Self::MIN_RSSI, Self::MAX_RSSI);
    let range = (Self::MAX_RSSI - Self::MIN_RSSI) as f32;
    let normalized = (clamped - Self::MIN_RSSI) as f32 / range;
    Self((normalized * 100.0).round() as u8)
  }

  pub fn value(self) -> u8 {
    self.0
  }

  pub fn as_fraction(self) -> f32 {
    self.0 as f32 / 100.0
  }
}

/// Normalized link health on a 0.0–1.0 scale (EWMA).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinkQuality(pub f32);

impl LinkQuality {
  pub const PERFECT: Self = Self(1.0);
  pub const UNKNOWN: Self = Self(0.5);

  pub fn new(value: f32) -> Self {
    Self(value.clamp(0.0, 1.0))
  }

  pub fn value(self) -> f32 {
    self.0
  }

  pub fn update(self, sample: f32, alpha: f32) -> Self {
    let sample = sample.clamp(0.0, 1.0);
    Self::new(self.0 * (1.0 - alpha) + sample * alpha)
  }

  pub fn record_success(self, alpha: f32) -> Self {
    self.update(1.0, alpha)
  }

  pub fn record_failure(self, alpha: f32) -> Self {
    self.update(0.0, alpha)
  }
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rssi_maps_to_signal_quality() {
    assert_eq!(SignalQuality::from_rssi(-30).value(), 100);
    assert_eq!(SignalQuality::from_rssi(-90).value(), 0);
    assert!(SignalQuality::from_rssi(-60).value() > 40);
  }

  #[test]
  fn link_quality_ewma() {
    let q = LinkQuality::UNKNOWN;
    let better = q.record_success(0.3);
    assert!(better.value() > q.value());
    let worse = better.record_failure(0.3);
    assert!(worse.value() < better.value());
  }
}
