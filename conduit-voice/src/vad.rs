/// Energy-based voice activity detection.
#[derive(Debug, Clone)]
pub struct VoiceActivityDetector {
  threshold: f32,
  push_to_talk: bool,
  ptt_active: bool,
}

impl VoiceActivityDetector {
  pub fn new(threshold: f32, push_to_talk: bool) -> Self {
    Self {
      threshold,
      push_to_talk,
      ptt_active: false,
    }
  }

  pub fn set_push_to_talk(&mut self, active: bool) {
    self.ptt_active = active;
  }

  pub fn is_active(&self, pcm: &[i16]) -> bool {
    if self.push_to_talk {
      return self.ptt_active;
    }
    Self::rms_energy(pcm) >= self.threshold
  }

  fn rms_energy(pcm: &[i16]) -> f32 {
    if pcm.is_empty() {
      return 0.0;
    }
    let sum: f64 = pcm
      .iter()
      .map(|&s| {
        let n = f64::from(s) / i16::MAX as f64;
        n * n
      })
      .sum();
    (sum / pcm.len() as f64).sqrt() as f32
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_loud_frame() {
    let vad = VoiceActivityDetector::new(0.01, false);
    let loud: Vec<i16> = vec![10_000; 960];
    assert!(vad.is_active(&loud));
  }

  #[test]
  fn push_to_talk_overrides_energy() {
    let mut vad = VoiceActivityDetector::new(0.99, true);
    let quiet = vec![0i16; 960];
    assert!(!vad.is_active(&quiet));
    vad.set_push_to_talk(true);
    assert!(vad.is_active(&quiet));
  }
}
