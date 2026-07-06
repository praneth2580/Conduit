use crate::codec::{AudioCodec, LinearCodec};
use crate::config::VoiceConfig;
use crate::jitter::JitterBuffer;
use crate::packetizer::VoicePacketizer;
use crate::pipeline::VoicePipeline;
use crate::vad::VoiceActivityDetector;
use conduit_core::error::Result;
use conduit_mesh::BROADCAST_NODE_ID;
use conduit_packets::{PacketBody, TypedPacket};

/// End-to-end voice capture and playback engine.
pub struct VoiceEngine<C: AudioCodec = LinearCodec> {
  config: VoiceConfig,
  codec: C,
  pipeline: VoicePipeline,
  packetizer: VoicePacketizer,
  jitter: JitterBuffer,
  capture_buffer: Vec<i16>,
}

impl VoiceEngine<LinearCodec> {
  pub fn new_linear(config: VoiceConfig) -> Result<Self> {
    config.validate()?;
    Ok(Self {
      pipeline: VoicePipeline::new(VoiceActivityDetector::new(
        config.vad_energy_threshold,
        config.push_to_talk,
      )),
      packetizer: VoicePacketizer::new(config.local_node_id, config.frame_duration_ms),
      jitter: JitterBuffer::new(config.jitter_buffer_ms, config.frame_duration_ms),
      capture_buffer: Vec::with_capacity(config.frame_samples()),
      codec: LinearCodec,
      config,
    })
  }
}

impl<C: AudioCodec> VoiceEngine<C> {
  pub fn new(config: VoiceConfig, codec: C) -> Result<Self> {
    config.validate()?;
    Ok(Self {
      pipeline: VoicePipeline::new(VoiceActivityDetector::new(
        config.vad_energy_threshold,
        config.push_to_talk,
      )),
      packetizer: VoicePacketizer::new(config.local_node_id, config.frame_duration_ms),
      jitter: JitterBuffer::new(config.jitter_buffer_ms, config.frame_duration_ms),
      capture_buffer: Vec::with_capacity(config.frame_samples()),
      codec,
      config,
    })
  }

  pub fn config(&self) -> &VoiceConfig {
    &self.config
  }

  pub fn set_push_to_talk(&mut self, active: bool) {
    self.pipeline.vad_mut().set_push_to_talk(active);
  }

  /// Feed PCM samples from the microphone. Returns a voice packet when a full
  /// frame is ready and voice activity is detected.
  pub fn capture(&mut self, samples: &[i16]) -> Result<Option<TypedPacket>> {
    self.capture_buffer.extend_from_slice(samples);

    let frame_samples = self.config.frame_samples();
    if self.capture_buffer.len() < frame_samples {
      return Ok(None);
    }

    let mut frame: Vec<i16> = self.capture_buffer.drain(..frame_samples).collect();
    if !self.pipeline.process_capture(&mut frame) {
      return Ok(None);
    }

    let encoded = self.codec.encode(&frame)?;
    self
      .packetizer
      .packetize(BROADCAST_NODE_ID, encoded)
      .map(Some)
  }

  /// Accept an inbound voice packet into the jitter buffer.
  pub fn receive(&mut self, packet: &TypedPacket) -> Result<()> {
    if packet.packet_type() != conduit_core::PacketType::Voice {
      return Err(conduit_core::ConduitError::InvalidPacket(
        "expected voice packet".into(),
      ));
    }
    if let PacketBody::Voice(payload) = &packet.body {
      self.jitter.insert(payload.clone());
    }
    Ok(())
  }

  /// Decode the next jitter-buffered frame for speaker playback.
  pub fn playback(&mut self) -> Result<Option<Vec<i16>>> {
    let payload = match self.jitter.pop() {
      Some(p) => p,
      None => return Ok(None),
    };
    let pcm = self.codec.decode(&payload.opus_data)?;
    Ok(Some(pcm))
  }

  pub fn jitter_buffer_len(&self) -> usize {
    self.jitter.len()
  }
}

#[cfg(feature = "opus")]
impl VoiceEngine<crate::codec::OpusCodec> {
  pub fn new_opus(config: VoiceConfig) -> Result<Self> {
    let codec = crate::codec::OpusCodec::new(
      config.sample_rate,
      config.bitrate,
      config.frame_samples(),
    )?;
    Self::new(config, codec)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::DEFAULT_FRAME_SAMPLES;

  fn test_config() -> VoiceConfig {
    VoiceConfig::builder()
      .vad_energy_threshold(0.001)
      .jitter_buffer_ms(40)
      .build()
  }

  fn loud_frame() -> Vec<i16> {
    vec![8000i16; DEFAULT_FRAME_SAMPLES]
  }

  #[test]
  fn capture_produces_voice_packet() {
    let mut engine = VoiceEngine::new_linear(test_config()).unwrap();
    let packet = engine.capture(&loud_frame()).unwrap().unwrap();
    assert_eq!(packet.packet_type(), conduit_core::PacketType::Voice);
  }

  #[test]
  fn receive_and_playback_round_trip() {
    let config = test_config();
    let mut tx = VoiceEngine::new_linear(config).unwrap();
    let mut rx = VoiceEngine::new_linear(test_config()).unwrap();

    let packet_a = tx.capture(&loud_frame()).unwrap().unwrap();
    let packet_b = tx.capture(&loud_frame()).unwrap().unwrap();
    rx.receive(&packet_a).unwrap();
    rx.receive(&packet_b).unwrap();

    let pcm = rx.playback().unwrap();
    assert!(pcm.is_some());
    assert_eq!(pcm.unwrap().len(), DEFAULT_FRAME_SAMPLES);
  }

  #[test]
  fn silent_frame_produces_no_packet() {
    let mut engine = VoiceEngine::new_linear(
      VoiceConfig::builder()
        .vad_energy_threshold(0.5)
        .build(),
    )
    .unwrap();
    let quiet = vec![0i16; DEFAULT_FRAME_SAMPLES];
    assert!(engine.capture(&quiet).unwrap().is_none());
  }
}
