use crate::neighbor::Neighbor;
use conduit_core::ids::NodeId;
use std::collections::HashMap;

/// In-memory neighbor table keyed by node ID.
#[derive(Debug, Default)]
pub struct NeighborTable {
  entries: HashMap<NodeId, Neighbor>,
}

impl NeighborTable {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  pub fn contains(&self, node_id: &NodeId) -> bool {
    self.entries.contains_key(node_id)
  }

  pub fn get(&self, node_id: &NodeId) -> Option<&Neighbor> {
    self.entries.get(node_id)
  }

  pub fn get_mut(&mut self, node_id: &NodeId) -> Option<&mut Neighbor> {
    self.entries.get_mut(node_id)
  }

  pub fn insert(&mut self, neighbor: Neighbor) -> Option<Neighbor> {
    self.entries.insert(neighbor.node_id, neighbor)
  }

  pub fn remove(&mut self, node_id: &NodeId) -> Option<Neighbor> {
    self.entries.remove(node_id)
  }

  pub fn values(&self) -> impl Iterator<Item = &Neighbor> {
    self.entries.values()
  }

  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Neighbor> {
    self.entries.values_mut()
  }

  pub fn clear(&mut self) {
    self.entries.clear();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::neighbor::{NeighborState, BROADCAST_NODE_ID};
  use conduit_discovery::{DiscoveredPeer, DriverKind, PeerEndpoint};

  #[test]
  fn insert_and_lookup() {
    let mut table = NeighborTable::new();
    let peer = DiscoveredPeer::new(
      NodeId::random(),
      "n".into(),
      0,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 1 },
    );
    let neighbor = Neighbor::from_discovered(&peer);
    let id = neighbor.node_id;
    table.insert(neighbor);
    assert_eq!(table.get(&id).unwrap().state, NeighborState::Discovered);
    assert_ne!(id, BROADCAST_NODE_ID);
  }
}
