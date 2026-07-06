use conduit_core::error::Result;

/// Encode/decode PCM audio frames.
pub trait AudioCodec: Send {
  fn encode(&mut self, pcm: &[i16]) -> Result<Vec<u8>>;
  fn decode(&mut self, data: &[u8]) -> Result<Vec<i16>>;
}

/// Simple codec for tests — stores PCM as little-endian bytes.
#[derive(Debug, Default)]
pub struct LinearCodec;

impl AudioCodec for LinearCodec {
  fn encode(&mut self, pcm: &[i16]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(pcm.len() * 2);
    for sample in pcm {
      out.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(out)
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<i16>> {
    if data.len() % 2 != 0 {
      return Err(conduit_core::ConduitError::Deserialization(
        "linear codec data length must be even".into(),
      ));
    }
    Ok(data
      .chunks_exact(2)
      .map(|c| i16::from_le_bytes([c[0], c[1]]))
      .collect())
  }
}

#[cfg(feature = "opus")]
mod opus_impl {
  use super::*;
  use audiopus::{Application, Bitrate, Channels, Encoder, SampleRate, Decoder};

  pub struct OpusCodec {
    encoder: Encoder,
    decoder: Decoder,
    frame_samples: usize,
  }

  impl OpusCodec {
    pub fn new(sample_rate: u32, bitrate: i32, frame_samples: usize) -> Result<Self> {
      let rate = match sample_rate {
        8000 => SampleRate::Hz8000,
        12000 => SampleRate::Hz12000,
        16000 => SampleRate::Hz16000,
        24000 => SampleRate::Hz24000,
        48000 => SampleRate::Hz48000,
        _ => {
          return Err(conduit_core::ConduitError::Configuration(format!(
            "unsupported sample rate: {sample_rate}"
          )));
        }
      };

      let mut encoder = Encoder::new(rate, Channels::Mono, Application::Voip)
        .map_err(|e| conduit_core::ConduitError::Serialization(e.to_string()))?;
      encoder
        .set_bitrate(Bitrate::Bits(bitrate))
        .map_err(|e| conduit_core::ConduitError::Serialization(e.to_string()))?;

      let decoder = Decoder::new(rate, Channels::Mono)
        .map_err(|e| conduit_core::ConduitError::Deserialization(e.to_string()))?;

      Ok(Self {
        encoder,
        decoder,
        frame_samples,
      })
    }
  }

  impl AudioCodec for OpusCodec {
    fn encode(&mut self, pcm: &[i16]) -> Result<Vec<u8>> {
      let mut out = vec![0u8; 4000];
      let len = self
        .encoder
        .encode(pcm, &mut out)
        .map_err(|e| conduit_core::ConduitError::Serialization(e.to_string()))?;
      out.truncate(len);
      Ok(out)
    }

    fn decode(&mut self, data: &[u8]) -> Result<Vec<i16>> {
      let mut pcm = vec![0i16; self.frame_samples];
      self
        .decoder
        .decode(Some(data), &mut pcm, false)
        .map_err(|e| conduit_core::ConduitError::Deserialization(e.to_string()))?;
      Ok(pcm)
    }
  }
}

#[cfg(feature = "opus")]
pub use opus_impl::OpusCodec;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn linear_codec_round_trip() {
    let pcm: Vec<i16> = (0..960).map(|i| (i % 300) as i16).collect();
    let mut codec = LinearCodec;
    let encoded = codec.encode(&pcm).unwrap();
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, pcm);
  }
}
