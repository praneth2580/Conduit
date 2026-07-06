use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use std::collections::HashMap;

/// Sliding-window replay protection per sender.
#[derive(Debug)]
pub struct ReplayGuard {
  window: u64,
  highest: HashMap<NodeId, u64>,
  seen: HashMap<NodeId, Vec<u64>>,
}

impl ReplayGuard {
  pub fn new(window: u64) -> Self {
    Self {
      window,
      highest: HashMap::new(),
      seen: HashMap::new(),
    }
  }

  pub fn check_and_record(&mut self, sender: NodeId, sequence: u64) -> Result<()> {
    let highest = self.highest.entry(sender).or_insert(0);

    if sequence + self.window < *highest {
      return Err(ConduitError::InvalidPacket(
        "sequence outside replay window".into(),
      ));
    }

    let seen = self.seen.entry(sender).or_default();
    if seen.contains(&sequence) {
      return Err(ConduitError::InvalidPacket("replay detected".into()));
    }

    seen.push(sequence);
    if sequence > *highest {
      *highest = sequence;
    }

    seen.retain(|seq| *highest <= seq + self.window);
    Ok(())
  }

  pub fn reset_peer(&mut self, peer: &NodeId) {
    self.highest.remove(peer);
    self.seen.remove(peer);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_duplicates() {
    let mut guard = ReplayGuard::new(64);
    let sender = NodeId::random();
    guard.check_and_record(sender, 1).unwrap();
    assert!(guard.check_and_record(sender, 1).is_err());
  }

  #[test]
  fn accepts_in_window() {
    let mut guard = ReplayGuard::new(64);
    let sender = NodeId::random();
    guard.check_and_record(sender, 10).unwrap();
    guard.check_and_record(sender, 5).unwrap();
  }
}
