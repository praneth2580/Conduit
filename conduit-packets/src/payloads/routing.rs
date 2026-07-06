use crate::payload::PacketPayload;
use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// Route advertisement or path update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingPayload {
  pub hop_count: u8,
  pub path: Vec<NodeId>,
}

impl PacketPayload for RoutingPayload {
  const PACKET_TYPE: PacketType = PacketType::Routing;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u8(self.hop_count);
    writer.write_u8(self.path.len() as u8);
    for node in &self.path {
      node.encode(writer);
    }
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    let hop_count = reader.read_u8()?;
    let path_len = reader.read_u8()? as usize;
    if path_len > 32 {
      return Err(ConduitError::InvalidPacket(format!(
        "routing path too long: {path_len}"
      )));
    }
    let mut path = Vec::with_capacity(path_len);
    for _ in 0..path_len {
      path.push(NodeId::decode(reader)?);
    }
    Ok(Self { hop_count, path })
  }
}
