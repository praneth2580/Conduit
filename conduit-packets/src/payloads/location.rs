use crate::payload::PacketPayload;
use conduit_core::error::Result;
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;
use serde::{Deserialize, Serialize};

/// GPS coordinates in microdegrees (degrees × 1_000_000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationPayload {
  pub latitude_microdeg: i32,
  pub longitude_microdeg: i32,
  pub altitude_m: i16,
  pub accuracy_m: u16,
}

impl PacketPayload for LocationPayload {
  const PACKET_TYPE: PacketType = PacketType::Location;

  fn encode(&self, writer: &mut ByteWriter) {
    writer.write_u32(self.latitude_microdeg as u32);
    writer.write_u32(self.longitude_microdeg as u32);
    writer.write_u16(self.altitude_m as u16);
    writer.write_u16(self.accuracy_m);
  }

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    Ok(Self {
      latitude_microdeg: reader.read_u32()? as i32,
      longitude_microdeg: reader.read_u32()? as i32,
      altitude_m: reader.read_u16()? as i16,
      accuracy_m: reader.read_u16()?,
    })
  }
}
