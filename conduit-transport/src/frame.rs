use crate::config::TRANSPORT_FRAME_HEADER_SIZE;
use conduit_core::constants::FRAME_MAGIC;
use conduit_core::error::{ConduitError, Result};
use conduit_core::serialization::{ByteReader, ByteWriter};

/// Per-frame transport flags describing how the inner blob was processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportFlags {
  bits: u8,
}

impl TransportFlags {
  pub const NONE: Self = Self { bits: 0 };
  pub const COMPRESSED: Self = Self { bits: 1 << 0 };
  pub const ENCRYPTED: Self = Self { bits: 1 << 1 };
  pub const FRAGMENT: Self = Self { bits: 1 << 2 };
  pub const LAST_FRAGMENT: Self = Self { bits: 1 << 3 };

  pub const fn bits(self) -> u8 {
    self.bits
  }

  pub fn from_bits(bits: u8) -> Option<Self> {
    if bits & !0b0000_1111 == 0 {
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

/// A single on-wire transport frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportFrame {
  pub flags: TransportFlags,
  pub fragment_id: u32,
  pub fragment_index: u16,
  pub fragment_total: u16,
  pub payload: Vec<u8>,
}

impl TransportFrame {
  pub fn encode(&self) -> Vec<u8> {
    let mut writer = ByteWriter::with_capacity(TRANSPORT_FRAME_HEADER_SIZE + self.payload.len());
    writer.write_bytes(&FRAME_MAGIC);
    writer.write_u8(crate::config::TRANSPORT_VERSION);
    writer.write_u8(self.flags.bits());
    writer.write_u32(self.fragment_id);
    writer.write_u16(self.fragment_index);
    writer.write_u16(self.fragment_total);
    writer.write_u16(self.payload.len() as u16);
    writer.write_bytes(&self.payload);
    writer.into_vec()
  }

  pub fn decode(data: &[u8]) -> Result<Self> {
    if data.len() < TRANSPORT_FRAME_HEADER_SIZE {
      return Err(ConduitError::Deserialization(
        "transport frame too short".into(),
      ));
    }
    if &data[..4] != FRAME_MAGIC {
      return Err(ConduitError::Deserialization(
        "invalid transport frame magic".into(),
      ));
    }

    let mut reader = ByteReader::new(data);
    reader.read_bytes(4)?; // magic
    let version = reader.read_u8()?;
    if version != crate::config::TRANSPORT_VERSION {
      return Err(ConduitError::Deserialization(format!(
        "unsupported transport version: {version}"
      )));
    }

    let flags = TransportFlags::from_bits(reader.read_u8()?)
      .ok_or_else(|| ConduitError::Deserialization("invalid transport flags".into()))?;
    let fragment_id = reader.read_u32()?;
    let fragment_index = reader.read_u16()?;
    let fragment_total = reader.read_u16()?;
    let payload_len = reader.read_u16()? as usize;

    if reader.remaining() < payload_len {
      return Err(ConduitError::BufferOverflow {
        needed: payload_len,
        available: reader.remaining(),
      });
    }
    let payload = reader.read_bytes(payload_len)?;

    Ok(Self {
      flags,
      fragment_id,
      fragment_index,
      fragment_total,
      payload,
    })
  }

  pub fn is_fragmented(&self) -> bool {
    self.flags.contains(TransportFlags::FRAGMENT)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn frame_round_trip() {
    let frame = TransportFrame {
      flags: TransportFlags::COMPRESSED,
      fragment_id: 42,
      fragment_index: 0,
      fragment_total: 1,
      payload: b"inner-blob".to_vec(),
    };
    let encoded = frame.encode();
    let decoded = TransportFrame::decode(&encoded).unwrap();
    assert_eq!(decoded, frame);
  }
}
