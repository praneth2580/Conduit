use crate::config::TransportConfig;
use crate::frame::{TransportFlags, TransportFrame};
use conduit_core::error::{ConduitError, Result};
use rand::RngCore;

/// Splits an inner blob into one or more transport frames.
pub struct Fragmenter {
  config: TransportConfig,
}

impl Fragmenter {
  pub fn new(config: TransportConfig) -> Self {
    Self { config }
  }

  pub fn fragment(
    &self,
    inner: &[u8],
    flags: TransportFlags,
    fragment_id: u32,
  ) -> Result<Vec<TransportFrame>> {
    let max_chunk = self.config.max_frame_payload();
    if inner.is_empty() {
      return Err(ConduitError::InvalidPacket(
        "cannot fragment empty blob".into(),
      ));
    }

    if inner.len() <= max_chunk {
      return Ok(vec![TransportFrame {
        flags,
        fragment_id,
        fragment_index: 0,
        fragment_total: 1,
        payload: inner.to_vec(),
      }]);
    }

    let chunks: Vec<_> = inner.chunks(max_chunk).map(|c| c.to_vec()).collect();
    let total = chunks.len() as u16;
    if total > self.config.max_fragments {
      return Err(ConduitError::InvalidPacket(format!(
        "packet requires {total} fragments, max is {}",
        self.config.max_fragments
      )));
    }

    let frames = chunks
      .into_iter()
      .enumerate()
      .map(|(index, payload)| {
        let mut frame_flags = flags.union(TransportFlags::FRAGMENT);
        if index + 1 == total as usize {
          frame_flags = frame_flags.union(TransportFlags::LAST_FRAGMENT);
        }
        TransportFrame {
          flags: frame_flags,
          fragment_id,
          fragment_index: index as u16,
          fragment_total: total,
          payload,
        }
      })
      .collect();

    Ok(frames)
  }

  pub fn next_fragment_id() -> u32 {
    let mut buf = [0u8; 4];
    rand::thread_rng().fill_bytes(&mut buf);
    u32::from_be_bytes(buf)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn single_frame_when_under_mtu() {
    let config = TransportConfig::builder().mtu(512).build();
    let fragmenter = Fragmenter::new(config);
    let frames = fragmenter
      .fragment(b"small", TransportFlags::NONE, 1)
      .unwrap();
    assert_eq!(frames.len(), 1);
    assert!(!frames[0].is_fragmented());
  }

  #[test]
  fn splits_large_blob() {
    let config = TransportConfig::builder().mtu(100).build();
    let fragmenter = Fragmenter::new(config);
    let data = vec![0xAB; 250];
    let frames = fragmenter
      .fragment(&data, TransportFlags::NONE, 7)
      .unwrap();
    assert!(frames.len() > 1);
    assert!(frames.last().unwrap().flags.contains(TransportFlags::LAST_FRAGMENT));
  }
}
