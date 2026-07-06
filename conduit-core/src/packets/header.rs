use crate::ids::NodeId;
use crate::packets::PacketType;
use crate::serialization::{ByteReader, ByteWriter, Decodable, Encodable};
use crate::utils::{checksum, unix_timestamp_ms};
use crate::version::ProtocolVersion;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Optional flags carried in the packet header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketFlags {
  bits: u8,
}

impl PacketFlags {
  pub const NONE: Self = Self { bits: 0 };
  pub const FRAGMENT: Self = Self { bits: 1 << 0 };
  pub const LAST_FRAGMENT: Self = Self { bits: 1 << 1 };
  pub const ENCRYPTED: Self = Self { bits: 1 << 2 };
  pub const COMPRESSED: Self = Self { bits: 1 << 3 };
  pub const URGENT: Self = Self { bits: 1 << 4 };

  pub const fn bits(self) -> u8 {
    self.bits
  }

  pub fn from_bits(bits: u8) -> Option<Self> {
    if bits & !0b0001_1111 == 0 {
      Some(Self { bits })
    } else {
      None
    }
  }

  pub const fn contains(self, other: Self) -> bool {
    self.bits & other.bits == other.bits
  }

  pub const fn union(self, other: Self) -> Self {
    Self {
      bits: self.bits | other.bits,
    }
  }
}

impl Serialize for PacketFlags {
  fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_u8(self.bits)
  }
}

impl<'de> Deserialize<'de> for PacketFlags {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
    let bits = u8::deserialize(deserializer)?;
    PacketFlags::from_bits(bits).ok_or_else(|| serde::de::Error::custom("invalid flags"))
  }
}

/// Delivery priority for outbound packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketPriority {
  Low = 0,
  Normal = 1,
  High = 2,
  Critical = 3,
}

impl PacketPriority {
  pub fn from_u8(value: u8) -> Option<Self> {
    match value {
      0 => Some(Self::Low),
      1 => Some(Self::Normal),
      2 => Some(Self::High),
      3 => Some(Self::Critical),
      _ => None,
    }
  }
}

/// Common header shared by every Conduit packet.
///
/// Field layout matches the Phase 2 specification; Phase 1 establishes the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
  pub version: ProtocolVersion,
  pub packet_type: PacketType,
  pub priority: PacketPriority,
  pub flags: PacketFlags,
  pub ttl: u8,
  pub sequence: crate::ids::PacketSequence,
  pub timestamp_ms: u64,
  pub source: NodeId,
  pub destination: NodeId,
  pub payload_length: u32,
  pub checksum: u32,
}

impl PacketHeader {
  pub const WIRE_SIZE: usize = crate::constants::PACKET_HEADER_SIZE;

  pub fn new(packet_type: PacketType, source: NodeId, destination: NodeId) -> Self {
    Self {
      version: crate::constants::PROTOCOL_VERSION,
      packet_type,
      priority: PacketPriority::Normal,
      flags: PacketFlags::NONE,
      ttl: crate::constants::DEFAULT_TTL,
      sequence: crate::ids::PacketSequence::ZERO,
      timestamp_ms: unix_timestamp_ms(),
      source,
      destination,
      payload_length: 0,
      checksum: 0,
    }
  }

  pub fn verify_checksum(&self, payload: &[u8]) -> Result<(), crate::error::ConduitError> {
    let actual = checksum(payload);
    if self.checksum != 0 && self.checksum != actual {
      return Err(crate::error::ConduitError::ChecksumMismatch {
        expected: self.checksum,
        actual,
      });
    }
    Ok(())
  }
}

impl Encodable for PacketHeader {
  fn encode(&self, writer: &mut ByteWriter) {
    self.version.encode(writer);
    writer.write_u8(self.packet_type as u8);
    writer.write_u8(self.priority as u8);
    writer.write_u8(self.flags.bits());
    writer.write_u8(self.ttl);
    self.sequence.encode(writer);
    writer.write_u64(self.timestamp_ms);
    self.source.encode(writer);
    self.destination.encode(writer);
    writer.write_u32(self.payload_length);
    writer.write_u32(self.checksum);
  }
}

impl Decodable for PacketHeader {
  fn decode(reader: &mut ByteReader<'_>) -> crate::error::Result<Self> {
    let version = ProtocolVersion::decode(reader)?;
    let packet_type = PacketType::from_u8(reader.read_u8()?)
      .ok_or_else(|| crate::error::ConduitError::InvalidPacket("unknown packet type".into()))?;
    let priority = PacketPriority::from_u8(reader.read_u8()?)
      .ok_or_else(|| crate::error::ConduitError::InvalidPacket("unknown priority".into()))?;
    let flags = PacketFlags::from_bits(reader.read_u8()?)
      .ok_or_else(|| crate::error::ConduitError::InvalidPacket("invalid flags".into()))?;
    let ttl = reader.read_u8()?;
    let sequence = crate::ids::PacketSequence::decode(reader)?;
    let timestamp_ms = reader.read_u64()?;
    let source = NodeId::decode(reader)?;
    let destination = NodeId::decode(reader)?;
    let payload_length = reader.read_u32()?;
    let checksum = reader.read_u32()?;

    Ok(Self {
      version,
      packet_type,
      priority,
      flags,
      ttl,
      sequence,
      timestamp_ms,
      source,
      destination,
      payload_length,
      checksum,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ids::PacketSequence;

  #[test]
  fn header_wire_size_is_fixed() {
    let header = PacketHeader::new(PacketType::Voice, NodeId::random(), NodeId::random());
    let mut writer = ByteWriter::new();
    header.encode(&mut writer);
    assert_eq!(writer.len(), PacketHeader::WIRE_SIZE);
  }

  #[test]
  fn header_round_trip() {
    let mut header = PacketHeader::new(PacketType::Discovery, NodeId::random(), NodeId::random());
    header.flags = PacketFlags::ENCRYPTED.union(PacketFlags::URGENT);
    header.priority = PacketPriority::High;
    header.sequence = PacketSequence(42);

    let mut writer = ByteWriter::new();
    header.encode(&mut writer);
    let mut reader = ByteReader::new(writer.as_slice());
    let restored = PacketHeader::decode(&mut reader).unwrap();
    assert_eq!(restored, header);
  }
}
