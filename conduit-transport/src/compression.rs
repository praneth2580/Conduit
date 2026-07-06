use conduit_core::error::{ConduitError, Result};

/// DEFLATE compression for transport payloads.
pub struct Compression;

impl Compression {
  /// Compress with zlib wrapper. Returns `None` if compression does not shrink the input.
  pub fn compress_if_beneficial(data: &[u8], threshold: usize) -> Result<Option<Vec<u8>>> {
    if data.len() < threshold {
      return Ok(None);
    }

    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(data, 6);
    if compressed.len() >= data.len() {
      return Ok(None);
    }
    Ok(Some(compressed))
  }

  pub fn compress(data: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
  }

  pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    miniz_oxide::inflate::decompress_to_vec_zlib(data)
      .map_err(|e| ConduitError::Deserialization(format!("decompression failed: {e:?}")))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let data = b"conduit transport compression test payload ".repeat(20);
    let compressed = Compression::compress(&data);
    let restored = Compression::decompress(&compressed).unwrap();
    assert_eq!(restored, data);
  }

  #[test]
  fn skips_small_payloads() {
    let data = b"tiny";
    assert!(Compression::compress_if_beneficial(data, 256).unwrap().is_none());
  }
}
