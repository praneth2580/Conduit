use conduit_packets::VoicePayload;
use std::collections::BTreeMap;

/// Reorders and buffers incoming voice frames before decode/playback.
#[derive(Debug)]
pub struct JitterBuffer {
  target_frames: usize,
  frames: BTreeMap<u16, VoicePayload>,
  next_play_index: u16,
  primed: bool,
}

impl JitterBuffer {
  pub fn new(target_delay_ms: u32, frame_duration_ms: u16) -> Self {
    let target_frames = (target_delay_ms / frame_duration_ms as u32).max(1) as usize;
    Self {
      target_frames,
      frames: BTreeMap::new(),
      next_play_index: 0,
      primed: false,
    }
  }

  pub fn insert(&mut self, payload: VoicePayload) {
    self.frames.insert(payload.frame_index, payload);
    self.trim();
  }

  pub fn ready(&self) -> bool {
    self.frames.len() >= self.target_frames
  }

  pub fn pop(&mut self) -> Option<VoicePayload> {
    if !self.primed {
      if !self.ready() {
        return None;
      }
      self.primed = true;
    }

    let key = self.next_play_index;
    if let Some(payload) = self.frames.remove(&key) {
      self.next_play_index = self.next_play_index.wrapping_add(1);
      return Some(payload);
    }

    // Skip gap — advance to next available frame.
    if let Some((&next, _)) = self.frames.iter().next() {
      self.next_play_index = next;
      return self.frames.remove(&next);
    }
    None
  }

  pub fn len(&self) -> usize {
    self.frames.len()
  }

  pub fn clear(&mut self) {
    self.frames.clear();
    self.next_play_index = 0;
    self.primed = false;
  }

  fn trim(&mut self) {
    while self.frames.len() > self.target_frames * 3 {
      if let Some(key) = self.frames.keys().next().copied() {
        self.frames.remove(&key);
      } else {
        break;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn frame(index: u16) -> VoicePayload {
    VoicePayload {
      frame_index: index,
      duration_ms: 20,
      opus_data: vec![index as u8],
    }
  }

  #[test]
  fn reorders_out_of_sequence_frames() {
    let mut buf = JitterBuffer::new(60, 20);
    buf.insert(frame(1));
    buf.insert(frame(0));
    assert!(!buf.ready());
    buf.insert(frame(2));
    assert!(buf.ready());
    assert_eq!(buf.pop().unwrap().frame_index, 0);
    assert_eq!(buf.pop().unwrap().frame_index, 1);
  }
}
