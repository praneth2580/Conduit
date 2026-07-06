use crate::frame::TransportFrame;
use conduit_core::error::{ConduitError, Result};
use std::collections::HashMap;

#[derive(Debug)]
struct PartialAssembly {
  flags: crate::frame::TransportFlags,
  fragment_total: u16,
  parts: HashMap<u16, Vec<u8>>,
}

/// Fully reassembled inner blob plus transport processing flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReassembledBlob {
  pub data: Vec<u8>,
  pub flags: crate::frame::TransportFlags,
}

/// Collects fragmented transport frames and reassembles inner blobs.
#[derive(Debug, Default)]
pub struct Reassembler {
  pending: HashMap<u32, PartialAssembly>,
}

impl Reassembler {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn ingest(&mut self, frame: &TransportFrame) -> Result<Option<ReassembledBlob>> {
    if frame.fragment_total == 0 {
      return Err(ConduitError::InvalidPacket(
        "fragment_total must be greater than zero".into(),
      ));
    }

    if !frame.is_fragmented() && frame.fragment_total == 1 {
      return Ok(Some(ReassembledBlob {
        data: frame.payload.clone(),
        flags: frame.flags,
      }));
    }

    let entry = self.pending.entry(frame.fragment_id).or_insert_with(|| PartialAssembly {
      flags: frame.flags,
      fragment_total: frame.fragment_total,
      parts: HashMap::new(),
    });

    if entry.fragment_total != frame.fragment_total {
      return Err(ConduitError::InvalidPacket(format!(
        "fragment_total mismatch for id {}",
        frame.fragment_id
      )));
    }

    if frame.fragment_index >= frame.fragment_total {
      return Err(ConduitError::InvalidPacket(format!(
        "fragment index {} out of range (total {})",
        frame.fragment_index, frame.fragment_total
      )));
    }

    entry.parts.insert(frame.fragment_index, frame.payload.clone());

    if entry.parts.len() < frame.fragment_total as usize {
      return Ok(None);
    }

    let assembly = self
      .pending
      .remove(&frame.fragment_id)
      .expect("entry was just inserted");

    let mut blob = Vec::new();
    for index in 0..assembly.fragment_total {
      let part = assembly.parts.get(&index).ok_or_else(|| {
        ConduitError::InvalidPacket(format!(
          "missing fragment {index} for id {}",
          frame.fragment_id
        ))
      })?;
      blob.extend_from_slice(part);
    }

    Ok(Some(ReassembledBlob {
      data: blob,
      flags: assembly.flags,
    }))
  }

  pub fn pending_count(&self) -> usize {
    self.pending.len()
  }

  pub fn clear(&mut self) {
    self.pending.clear();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::frame::TransportFlags;

  fn make_fragments(data: &[u8], chunk_size: usize, id: u32) -> Vec<TransportFrame> {
    let chunks: Vec<_> = data.chunks(chunk_size).map(|c| c.to_vec()).collect();
    let total = chunks.len() as u16;
    chunks
      .into_iter()
      .enumerate()
      .map(|(index, payload)| {
        let mut flags = TransportFlags::FRAGMENT;
        if index + 1 == total as usize {
          flags = flags.union(TransportFlags::LAST_FRAGMENT);
        }
        TransportFrame {
          flags,
          fragment_id: id,
          fragment_index: index as u16,
          fragment_total: total,
          payload,
        }
      })
      .collect()
  }

  #[test]
  fn reassembles_out_of_order() {
    let data = b"conduit-fragment-reassembly-test-data".repeat(5);
    let frames = make_fragments(&data, 20, 99);

    let mut reassembler = Reassembler::new();
    let mut result = None;
    for frame in frames.iter().rev() {
      if let Some(blob) = reassembler.ingest(frame).unwrap() {
        result = Some(blob);
      }
    }
    assert_eq!(result.unwrap().data, data);
  }
}
