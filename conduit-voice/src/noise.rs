/// Noise suppression stage (replaceable stub).
pub trait NoiseSuppressor: Send {
  fn process(&mut self, pcm: &mut [i16]);
}

/// Pass-through noise suppressor — placeholder for a real DSP implementation.
#[derive(Debug, Default)]
pub struct PassthroughNoiseSuppressor;

impl NoiseSuppressor for PassthroughNoiseSuppressor {
  fn process(&mut self, _pcm: &mut [i16]) {}
}
