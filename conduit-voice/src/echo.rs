/// Echo cancellation stage (replaceable stub).
pub trait EchoCanceller: Send {
  fn process(&mut self, pcm: &mut [i16]);
}

/// Pass-through echo canceller — placeholder for a real AEC implementation.
#[derive(Debug, Default)]
pub struct PassthroughEchoCanceller;

impl EchoCanceller for PassthroughEchoCanceller {
  fn process(&mut self, _pcm: &mut [i16]) {}
}
