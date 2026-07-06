use crate::error::{ConduitError, Result};

/// Growable byte buffer for encoding wire-format data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ByteWriter {
  buf: Vec<u8>,
}

impl ByteWriter {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      buf: Vec::with_capacity(capacity),
    }
  }

  pub fn len(&self) -> usize {
    self.buf.len()
  }

  pub fn is_empty(&self) -> bool {
    self.buf.is_empty()
  }

  pub fn as_slice(&self) -> &[u8] {
    &self.buf
  }

  pub fn into_vec(self) -> Vec<u8> {
    self.buf
  }

  pub fn write_u8(&mut self, value: u8) {
    self.buf.push(value);
  }

  pub fn write_u16(&mut self, value: u16) {
    self.buf.extend_from_slice(&value.to_be_bytes());
  }

  pub fn write_u32(&mut self, value: u32) {
    self.buf.extend_from_slice(&value.to_be_bytes());
  }

  pub fn write_u64(&mut self, value: u64) {
    self.buf.extend_from_slice(&value.to_be_bytes());
  }

  pub fn write_bytes(&mut self, bytes: &[u8]) {
    self.buf.extend_from_slice(bytes);
  }

  pub fn write_length_prefixed(&mut self, bytes: &[u8]) {
    self.write_u16(bytes.len() as u16);
    self.write_bytes(bytes);
  }
}

/// Sequential reader over a byte slice.
#[derive(Debug, Clone)]
pub struct ByteReader<'a> {
  data: &'a [u8],
  pos: usize,
}

impl<'a> ByteReader<'a> {
  pub fn new(data: &'a [u8]) -> Self {
    Self { data, pos: 0 }
  }

  pub fn remaining(&self) -> usize {
    self.data.len().saturating_sub(self.pos)
  }

  pub fn position(&self) -> usize {
    self.pos
  }

  fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
    if self.remaining() < len {
      return Err(ConduitError::BufferOverflow {
        needed: len,
        available: self.remaining(),
      });
    }
    let slice = &self.data[self.pos..self.pos + len];
    self.pos += len;
    Ok(slice)
  }

  pub fn read_u8(&mut self) -> Result<u8> {
    Ok(self.read_exact(1)?[0])
  }

  pub fn read_u16(&mut self) -> Result<u16> {
    let bytes = self.read_exact(2)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
  }

  pub fn read_u32(&mut self) -> Result<u32> {
    let bytes = self.read_exact(4)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
  }

  pub fn read_u64(&mut self) -> Result<u64> {
    let bytes = self.read_exact(8)?;
    Ok(u64::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
  }

  pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
    Ok(self.read_exact(len)?.to_vec())
  }

  pub fn read_length_prefixed(&mut self) -> Result<Vec<u8>> {
    let len = self.read_u16()? as usize;
    self.read_bytes(len)
  }
}

/// Trait for types that can be serialized to bytes.
pub trait Encodable {
  fn encode(&self, writer: &mut ByteWriter);
}

/// Trait for types that can be deserialized from bytes.
pub trait Decodable: Sized {
  fn decode(reader: &mut ByteReader<'_>) -> Result<Self>;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn primitive_round_trip() {
    let mut writer = ByteWriter::new();
    writer.write_u8(1);
    writer.write_u16(0xABCD);
    writer.write_u32(0x12345678);
    writer.write_u64(0xDEADBEEF_CAFEBABE);

    let mut reader = ByteReader::new(writer.as_slice());
    assert_eq!(reader.read_u8().unwrap(), 1);
    assert_eq!(reader.read_u16().unwrap(), 0xABCD);
    assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    assert_eq!(reader.read_u64().unwrap(), 0xDEADBEEF_CAFEBABE);
    assert_eq!(reader.remaining(), 0);
  }

  #[test]
  fn length_prefixed_round_trip() {
    let payload = b"hello conduit";
    let mut writer = ByteWriter::new();
    writer.write_length_prefixed(payload);

    let mut reader = ByteReader::new(writer.as_slice());
    assert_eq!(reader.read_length_prefixed().unwrap(), payload);
  }

  #[test]
  fn reader_errors_on_underflow() {
    let mut reader = ByteReader::new(&[0u8; 1]);
    assert!(reader.read_u16().is_err());
  }
}
