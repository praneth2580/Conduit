use conduit_core::ids::{NodeId, PacketSequence};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PacketKey {
  source: NodeId,
  sequence: PacketSequence,
}

/// Tracks recently seen packets to suppress duplicates.
#[derive(Debug)]
pub struct DuplicateCache {
  window_ms: u64,
  max_entries: usize,
  entries: HashMap<PacketKey, u64>,
  order: VecDeque<(PacketKey, u64)>,
}

impl DuplicateCache {
  pub fn new(window_ms: u64, max_entries: usize) -> Self {
    Self {
      window_ms,
      max_entries,
      entries: HashMap::new(),
      order: VecDeque::new(),
    }
  }

  pub fn is_duplicate(&mut self, source: NodeId, sequence: PacketSequence, now_ms: u64) -> bool {
    self.evict_expired(now_ms);
    let key = PacketKey { source, sequence };
    if self.entries.contains_key(&key) {
      return true;
    }
    self.insert(key, now_ms);
    false
  }

  fn insert(&mut self, key: PacketKey, now_ms: u64) {
    if self.entries.len() >= self.max_entries {
      if let Some((old_key, _)) = self.order.pop_front() {
        self.entries.remove(&old_key);
      }
    }
    self.entries.insert(key, now_ms);
    self.order.push_back((key, now_ms));
  }

  fn evict_expired(&mut self, now_ms: u64) {
    while let Some((key, ts)) = self.order.front().copied() {
      if now_ms.saturating_sub(ts) <= self.window_ms {
        break;
      }
      self.order.pop_front();
      self.entries.remove(&key);
    }
  }

  pub fn len(&self) -> usize {
    self.entries.len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_repeated_sequence() {
    let mut cache = DuplicateCache::new(10_000, 100);
    let source = NodeId::random();
    let seq = PacketSequence(1);
    assert!(!cache.is_duplicate(source, seq, 100));
    assert!(cache.is_duplicate(source, seq, 200));
  }
}
