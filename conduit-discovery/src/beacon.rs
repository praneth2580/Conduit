use crate::config::{BEACON_MAGIC, BEACON_VERSION};
use crate::DiscoveryAnnouncement;
use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use conduit_core::serialization::{ByteReader, ByteWriter};

/// Encode a discovery beacon for broadcast on the wire.
pub fn encode_beacon(announcement: &DiscoveryAnnouncement) -> Result<Vec<u8>> {
  let name_bytes = announcement.node_name.as_bytes();
  if name_bytes.len() > u16::MAX as usize {
    return Err(ConduitError::Serialization(
      "node name too long for beacon".into(),
    ));
  }

  let mut writer = ByteWriter::new();
  writer.write_bytes(&BEACON_MAGIC);
  writer.write_u8(BEACON_VERSION);
  announcement.node_id.encode(&mut writer);
  writer.write_length_prefixed(name_bytes);
  writer.write_u32(announcement.capabilities);
  Ok(writer.into_vec())
}

/// Decode a discovery beacon from raw bytes.
pub fn decode_beacon(data: &[u8]) -> Result<DiscoveryAnnouncement> {
  if data.len() < 4 {
    return Err(ConduitError::Deserialization("beacon too short".into()));
  }
  if &data[..4] != BEACON_MAGIC {
    return Err(ConduitError::Deserialization("invalid beacon magic".into()));
  }

  let mut reader = ByteReader::new(data);
  reader.read_bytes(4)?;
  let version = reader.read_u8()?;
  if version != BEACON_VERSION {
    return Err(ConduitError::Deserialization(format!(
      "unsupported beacon version: {version}"
    )));
  }

  let node_id = NodeId::decode(&mut reader)?;
  let name_bytes = reader.read_length_prefixed()?;
  let node_name = String::from_utf8(name_bytes)
    .map_err(|e| ConduitError::Deserialization(format!("invalid node name: {e}")))?;
  let capabilities = reader.read_u32()?;

  Ok(DiscoveryAnnouncement {
    node_id,
    node_name,
    capabilities,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_core::NodeId;

  #[test]
  fn beacon_round_trip() {
    let announcement = DiscoveryAnnouncement {
      node_id: NodeId::random(),
      node_name: "rider-alpha".into(),
      capabilities: 0b101,
    };
    let bytes = encode_beacon(&announcement).unwrap();
    assert_eq!(decode_beacon(&bytes).unwrap(), announcement);
  }
}
