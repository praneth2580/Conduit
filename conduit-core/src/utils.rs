use crate::constants::{FRAME_MAGIC, FRAME_MAGIC_SIZE};
use crate::error::{ConduitError, Result};
use crate::serialization::ByteWriter;
use std::time::{SystemTime, UNIX_EPOCH};

/// CRC-32 checksum (IEEE polynomial) over arbitrary bytes.
pub fn checksum(data: &[u8]) -> u32 {
  const TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
      let mut crc = i as u32;
      let mut j = 0;
      while j < 8 {
        if crc & 1 == 1 {
          crc = (crc >> 1) ^ 0xEDB8_8320;
        } else {
          crc >>= 1;
        }
        j += 1;
      }
      table[i] = crc;
      i += 1;
    }
    table
  };

  let mut crc = 0xFFFF_FFFF;
  for byte in data {
    let index = ((crc ^ u32::from(*byte)) & 0xFF) as usize;
    crc = (crc >> 8) ^ TABLE[index];
  }
  !crc
}

/// Current Unix timestamp in milliseconds.
pub fn unix_timestamp_ms() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() as u64
}

/// Wrap raw packet bytes with the Conduit frame magic prefix.
pub fn frame_packet(packet_bytes: &[u8]) -> Vec<u8> {
  let mut writer = ByteWriter::with_capacity(FRAME_MAGIC_SIZE + packet_bytes.len());
  writer.write_bytes(&FRAME_MAGIC);
  writer.write_bytes(packet_bytes);
  writer.into_vec()
}

/// Strip the frame magic and return the inner packet bytes.
pub fn unframe_packet(data: &[u8]) -> Result<&[u8]> {
  if data.len() < FRAME_MAGIC_SIZE {
    return Err(ConduitError::Deserialization(
      "frame too short for magic prefix".into(),
    ));
  }
  if &data[..FRAME_MAGIC_SIZE] != FRAME_MAGIC {
    return Err(ConduitError::Deserialization(
      "invalid frame magic".into(),
    ));
  }
  Ok(&data[FRAME_MAGIC_SIZE..])
}

/// Clamp a value between min and max (inclusive).
pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
  if value < min {
    min
  } else if value > max {
    max
  } else {
    value
  }
}

/// Format a byte count for human-readable logging.
pub fn format_bytes(bytes: usize) -> String {
  const KIB: f64 = 1024.0;
  if bytes < 1024 {
    format!("{bytes} B")
  } else if bytes < 1024 * 1024 {
    format!("{:.1} KiB", bytes as f64 / KIB)
  } else {
    format!("{:.1} MiB", bytes as f64 / (KIB * KIB))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn checksum_is_deterministic() {
    let data = b"conduit";
    assert_eq!(checksum(data), checksum(data));
    assert_ne!(checksum(data), checksum(b"different"));
  }

  #[test]
  fn frame_round_trip() {
    let inner = b"packet-data";
    let framed = frame_packet(inner);
    assert_eq!(unframe_packet(&framed).unwrap(), inner);
  }

  #[test]
  fn clamp_limits_value() {
    assert_eq!(clamp(5, 0, 10), 5);
    assert_eq!(clamp(-1, 0, 10), 0);
    assert_eq!(clamp(99, 0, 10), 10);
  }
}
