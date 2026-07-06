use crate::payload::{read_string, write_string, PacketPayload};
use conduit_core::error::Result;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// Single metric sample from a node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPayload {
  pub metric: String,
  pub value: f32,
}

impl PacketPayload for TelemetryPayload {
  const PACKET_TYPE: PacketType = PacketType::Telemetry;

  fn encode(&self, writer: &mut ByteWriter) {
    write_string(writer, &self.metric).expect("validated on build");
    writer.write_u32(self.value.to_bits());
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    Ok(Self {
      metric: read_string(reader)?,
      value: f32::from_bits(reader.read_u32()?),
    })
  }
}
