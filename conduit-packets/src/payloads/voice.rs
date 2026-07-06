use crate::payload::PacketPayload;
use conduit_core::error::Result;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// Encoded Opus audio frame carried inside a voice packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoicePayload {
  pub frame_index: u16,
  pub duration_ms: u16,
  pub opus_data: Vec<u8>,
}

impl PacketPayload for VoicePayload {
  const PACKET_TYPE: PacketType = PacketType::Voice;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u16(self.frame_index);
    writer.write_u16(self.duration_ms);
    writer.write_length_prefixed(&self.opus_data);
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    Ok(Self {
      frame_index: reader.read_u16()?,
      duration_ms: reader.read_u16()?,
      opus_data: reader.read_length_prefixed()?,
    })
  }
}
