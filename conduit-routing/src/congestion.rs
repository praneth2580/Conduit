use conduit_core::ids::NodeId;
use std::collections::HashMap;

/// Tracks in-flight forwards per neighbor for congestion control.
#[derive(Debug, Default)]
pub struct CongestionTracker {
  limit: u16,
  in_flight: HashMap<NodeId, u16>,
}

impl CongestionTracker {
  pub fn new(limit: u16) -> Self {
    Self {
      limit,
      in_flight: HashMap::new(),
    }
  }

  pub fn is_congested(&self, neighbor: &NodeId) -> bool {
    self.in_flight.get(neighbor).copied().unwrap_or(0) >= self.limit
  }

  pub fn record_send(&mut self, neighbor: NodeId) {
    *self.in_flight.entry(neighbor).or_insert(0) += 1;
  }

  pub fn record_complete(&mut self, neighbor: NodeId) {
    if let Some(count) = self.in_flight.get_mut(&neighbor) {
      *count = count.saturating_sub(1);
      if *count == 0 {
        self.in_flight.remove(&neighbor);
      }
    }
  }

  pub fn in_flight_count(&self, neighbor: &NodeId) -> u16 {
    self.in_flight.get(neighbor).copied().unwrap_or(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn marks_neighbor_congested_at_limit() {
    let mut tracker = CongestionTracker::new(2);
    let n = NodeId::random();
    tracker.record_send(n);
    tracker.record_send(n);
    assert!(tracker.is_congested(&n));
    tracker.record_complete(n);
    assert!(!tracker.is_congested(&n));
  }
}
