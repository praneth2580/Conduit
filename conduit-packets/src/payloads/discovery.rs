use crate::payload::{read_string, write_string, PacketPayload};
use conduit_core::error::Result;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// Announcement broadcast during peer discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryPayload {
  pub node_name: String,
  pub capabilities: u32,
}

impl PacketPayload for DiscoveryPayload {
  const PACKET_TYPE: PacketType = PacketType::Discovery;

  fn encode(&self, writer: &mut ByteWriter) {
    write_string(writer, &self.node_name).expect("validated on build");
    writer.write_u32(self.capabilities);
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    Ok(Self {
      node_name: read_string(reader)?,
      capabilities: reader.read_u32()?,
    })
  }
}
