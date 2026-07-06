use conduit_core::error::{ConduitError, Result};
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;

/// A typed packet payload with a known wire format.
///
/// Implement this trait to add new packet types. Routing never inspects payloads —
/// only the header [`PacketType`] field matters for forwarding.
pub trait PacketPayload: Sized {
  const PACKET_TYPE: PacketType;

  fn encode(&self, writer: &mut ByteWriter);

  fn decode(reader: &mut ByteReader<'_>) -> Result<Self>;

  fn to_bytes(&self) -> Vec<u8> {
    let mut writer = ByteWriter::new();
    self.encode(&mut writer);
    writer.into_vec()
  }

  fn from_bytes(data: &[u8]) -> Result<Self> {
    let mut reader = ByteReader::new(data);
    Self::decode(&mut reader)
  }
}

pub(crate) fn write_string(writer: &mut ByteWriter, value: &str) -> Result<()> {
  let bytes = value.as_bytes();
  if bytes.len() > u16::MAX as usize {
    return Err(ConduitError::Serialization(format!(
      "string too long: {} bytes",
      bytes.len()
    )));
  }
  writer.write_length_prefixed(bytes);
  Ok(())
}

pub(crate) fn read_string(reader: &mut ByteReader<'_>) -> Result<String> {
  let bytes = reader.read_length_prefixed()?;
  String::from_utf8(bytes)
    .map_err(|e| ConduitError::Deserialization(format!("invalid utf-8 string: {e}")))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn string_round_trip() {
    let mut writer = ByteWriter::new();
    write_string(&mut writer, "conduit").unwrap();
    let mut reader = ByteReader::new(writer.as_slice());
    assert_eq!(read_string(&mut reader).unwrap(), "conduit");
  }
}
