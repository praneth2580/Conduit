use crate::payload::{read_string, write_string, PacketPayload};
use conduit_core::error::{ConduitError, Result};
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EmergencyKind {
  Medical = 1,
  Mechanical = 2,
  Lost = 3,
  General = 4,
}

impl EmergencyKind {
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      1 => Some(Self::Medical),
      2 => Some(Self::Mechanical),
      3 => Some(Self::Lost),
      4 => Some(Self::General),
      _ => None,
    }
  }
}

/// High-priority distress signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmergencyPayload {
  pub kind: EmergencyKind,
  pub message: String,
}

impl PacketPayload for EmergencyPayload {
  const PACKET_TYPE: PacketType = PacketType::Emergency;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u8(self.kind as u8);
    write_string(writer, &self.message).expect("validated on build");
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    let kind = EmergencyKind::from_u8(reader.read_u8()?)
      .ok_or_else(|| ConduitError::InvalidPacket("unknown emergency kind".into()))?;
    let message = read_string(reader)?;
    Ok(Self { kind, message })
  }
}
