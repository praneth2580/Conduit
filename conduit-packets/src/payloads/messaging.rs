use crate::payload::{read_string, write_string, PacketPayload};
use conduit_core::error::Result;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// Text message between nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessagingPayload {
  pub message_id: u64,
  pub content: String,
}

impl PacketPayload for MessagingPayload {
  const PACKET_TYPE: PacketType = PacketType::Messaging;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u64(self.message_id);
    write_string(writer, &self.content).expect("validated on build");
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    Ok(Self {
      message_id: reader.read_u64()?,
      content: read_string(reader)?,
    })
  }
}
