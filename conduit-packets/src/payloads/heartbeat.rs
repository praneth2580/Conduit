use crate::payload::PacketPayload;
use conduit_core::error::Result;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// Periodic liveness signal from a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatPayload {
  pub uptime_ms: u64,
  pub neighbor_count: u16,
}

impl PacketPayload for HeartbeatPayload {
  const PACKET_TYPE: PacketType = PacketType::Heartbeat;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u64(self.uptime_ms);
    writer.write_u16(self.neighbor_count);
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    Ok(Self {
      uptime_ms: reader.read_u64()?,
      neighbor_count: reader.read_u16()?,
    })
  }
}
