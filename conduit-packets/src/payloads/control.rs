use crate::payload::PacketPayload;
use conduit_core::error::{ConduitError, Result};
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ControlCommand {
  Join = 1,
  Leave = 2,
  Mute = 3,
  Unmute = 4,
  Sync = 5,
}

impl ControlCommand {
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      1 => Some(Self::Join),
      2 => Some(Self::Leave),
      3 => Some(Self::Mute),
      4 => Some(Self::Unmute),
      5 => Some(Self::Sync),
      _ => None,
    }
  }
}

/// Session or channel control signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPayload {
  pub command: ControlCommand,
  pub argument: u32,
}

impl PacketPayload for ControlPayload {
  const PACKET_TYPE: PacketType = PacketType::Control;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u8(self.command as u8);
    writer.write_u32(self.argument);
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    let command = ControlCommand::from_u8(reader.read_u8()?)
      .ok_or_else(|| ConduitError::InvalidPacket("unknown control command".into()))?;
    Ok(Self {
      command,
      argument: reader.read_u32()?,
    })
  }
}
