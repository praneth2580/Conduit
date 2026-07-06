use crate::echo::{EchoCanceller, PassthroughEchoCanceller};
use crate::noise::{NoiseSuppressor, PassthroughNoiseSuppressor};
use crate::vad::VoiceActivityDetector;

/// Ordered capture pipeline stages before encoding.
pub struct VoicePipeline {
  noise: Box<dyn NoiseSuppressor>,
  echo: Box<dyn EchoCanceller>,
  vad: VoiceActivityDetector,
}

impl VoicePipeline {
  pub fn new(vad: VoiceActivityDetector) -> Self {
    Self {
      noise: Box::new(PassthroughNoiseSuppressor),
      echo: Box::new(PassthroughEchoCanceller),
      vad,
    }
  }

  pub fn with_noise_suppressor(mut self, stage: Box<dyn NoiseSuppressor>) -> Self {
    self.noise = stage;
    self
  }

  pub fn with_echo_canceller(mut self, stage: Box<dyn EchoCanceller>) -> Self {
    self.echo = stage;
    self
  }

  pub fn vad_mut(&mut self) -> &mut VoiceActivityDetector {
    &mut self.vad
  }

  /// Run capture-side processing. Returns `true` when voice is active.
  pub fn process_capture(&mut self, pcm: &mut [i16]) -> bool {
    self.noise.process(pcm);
    self.echo.process(pcm);
    self.vad.is_active(pcm)
  }
}
