mod header;
mod types;

pub use header::{PacketFlags, PacketHeader, PacketPriority};
pub use types::PacketType;

use crate::error::{ConduitError, Result};
use crate::ids::NodeId;
use crate::serialization::{ByteReader, ByteWriter, Decodable, Encodable};
use crate::utils::checksum;

/// A complete Conduit packet: header plus opaque payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
  pub header: PacketHeader,
  pub payload: Vec<u8>,
}

impl Packet {
  pub fn new(header: PacketHeader, payload: Vec<u8>) -> Self {
    Self { header, payload }
  }

  pub fn with_payload(
    packet_type: PacketType,
    source: NodeId,
    destination: NodeId,
    payload: Vec<u8>,
  ) -> Self {
    let mut header = PacketHeader::new(packet_type, source, destination);
    header.payload_length = payload.len() as u32;
    Self { header, payload }
  }

  pub fn packet_type(&self) -> PacketType {
    self.header.packet_type
  }

  pub fn source(&self) -> NodeId {
    self.header.source
  }

  pub fn destination(&self) -> NodeId {
    self.header.destination
  }

  pub fn total_size(&self) -> usize {
    PacketHeader::WIRE_SIZE + self.payload.len()
  }
}

impl Encodable for Packet {
  fn encode(&self, writer: &mut ByteWriter) {
    self.header.encode(writer);
    writer.write_bytes(&self.payload);
  }
}

impl Decodable for Packet {
  fn decode(reader: &mut ByteReader<'_>) -> Result<Self> {
    let header = PacketHeader::decode(reader)?;
    if reader.remaining() < header.payload_length as usize {
      return Err(ConduitError::BufferOverflow {
        needed: header.payload_length as usize,
        available: reader.remaining(),
      });
    }
    let payload = reader.read_bytes(header.payload_length as usize)?;
    let packet = Self { header, payload };
    packet.header.verify_checksum(&packet.payload)?;
    Ok(packet)
  }
}

impl Packet {
  /// Serialize the packet and finalize the header checksum.
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut header = self.header.clone();
    header.payload_length = self.payload.len() as u32;
    header.checksum = checksum(&self.payload);

    let mut writer = ByteWriter::with_capacity(PacketHeader::WIRE_SIZE + self.payload.len());
    header.encode(&mut writer);
    writer.write_bytes(&self.payload);
    writer.into_vec()
  }

  pub fn from_bytes(data: &[u8]) -> Result<Self> {
    let mut reader = ByteReader::new(data);
    Self::decode(&mut reader)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::version::ProtocolVersion;

  #[test]
  fn packet_round_trip() {
    let source = NodeId::random();
    let dest = NodeId::random();
    let payload = b"phase-1-payload".to_vec();
    let packet = Packet::with_payload(PacketType::Heartbeat, source, dest, payload.clone());

    let bytes = packet.to_bytes();
    let restored = Packet::from_bytes(&bytes).unwrap();

    assert_eq!(restored.header.packet_type, PacketType::Heartbeat);
    assert_eq!(restored.header.source, source);
    assert_eq!(restored.header.destination, dest);
    assert_eq!(restored.payload, payload);
    assert_eq!(restored.header.version, ProtocolVersion::CURRENT);
  }
}
