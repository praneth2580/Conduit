use conduit_core::ids::NodeId;
use std::collections::HashMap;

/// A known route toward a destination node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteEntry {
  pub destination: NodeId,
  pub next_hop: NodeId,
  pub hop_count: u8,
  pub last_updated_ms: u64,
}

/// Distance-vector style routing table.
#[derive(Debug, Default)]
pub struct RouteTable {
  routes: HashMap<NodeId, RouteEntry>,
}

impl RouteTable {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn learn(
    &mut self,
    destination: NodeId,
    next_hop: NodeId,
    hop_count: u8,
    now_ms: u64,
  ) -> bool {
    if destination == next_hop && hop_count > 1 {
      return false;
    }

    let replace = match self.routes.get(&destination) {
      None => true,
      Some(existing) => hop_count < existing.hop_count,
    };

    if replace {
      self.routes.insert(
        destination,
        RouteEntry {
          destination,
          next_hop,
          hop_count,
          last_updated_ms: now_ms,
        },
      );
      true
    } else {
      false
    }
  }

  pub fn get(&self, destination: &NodeId) -> Option<&RouteEntry> {
    self.routes.get(destination)
  }

  pub fn remove_via(&mut self, next_hop: &NodeId) -> Vec<NodeId> {
    let removed: Vec<NodeId> = self
      .routes
      .iter()
      .filter(|(_, entry)| entry.next_hop == *next_hop)
      .map(|(dest, _)| *dest)
      .collect();
    for dest in &removed {
      self.routes.remove(dest);
    }
    removed
  }

  pub fn remove_stale(&mut self, stale_after_ms: u64, now_ms: u64) {
    self.routes.retain(|_, entry| {
      now_ms.saturating_sub(entry.last_updated_ms) <= stale_after_ms
    });
  }

  pub fn remove_neighbor_routes(&mut self, neighbor: &NodeId) {
    self.routes.remove(neighbor);
    self.remove_via(neighbor);
  }

  pub fn len(&self) -> usize {
    self.routes.len()
  }

  pub fn entries(&self) -> impl Iterator<Item = &RouteEntry> {
    self.routes.values()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prefers_shorter_paths() {
    let mut table = RouteTable::new();
    let dest = NodeId::random();
    let hop_a = NodeId::random();
    let hop_b = NodeId::random();
    table.learn(dest, hop_a, 3, 100);
    table.learn(dest, hop_b, 2, 200);
    assert_eq!(table.get(&dest).unwrap().next_hop, hop_b);
  }

  #[test]
  fn removes_routes_via_failed_neighbor() {
    let mut table = RouteTable::new();
    let dest = NodeId::random();
    let via = NodeId::random();
    table.learn(dest, via, 2, 100);
    table.remove_via(&via);
    assert!(table.get(&dest).is_none());
  }
}
