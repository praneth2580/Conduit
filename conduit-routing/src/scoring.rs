use conduit_core::ids::NodeId;
use conduit_mesh::Neighbor;

/// Scores neighbors for route selection using mesh link/signal quality.
#[derive(Debug, Default)]
pub struct NeighborScorer;

impl NeighborScorer {
  pub fn score(neighbor: &Neighbor) -> f32 {
    let link = neighbor.link_quality.value();
    let signal = neighbor.signal_quality.as_fraction();
    let relay_bonus = if neighbor.capabilities & conduit_mesh::capabilities::RELAY != 0 {
      0.1
    } else {
      0.0
    };
    (link * 0.6 + signal * 0.4 + relay_bonus).clamp(0.0, 1.0)
  }

  /// Rank neighbors by score (highest first), excluding blocked nodes.
  pub fn rank<'a>(
    neighbors: &'a [Neighbor],
    exclude: &[NodeId],
  ) -> Vec<(&'a Neighbor, f32)> {
    let mut ranked: Vec<_> = neighbors
      .iter()
      .filter(|n| !exclude.contains(&n.node_id))
      .map(|n| (n, Self::score(n)))
      .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_discovery::{DriverKind, PeerEndpoint};

  #[test]
  fn relay_neighbors_score_higher() {
    let mut relay = Neighbor::from_discovered(&conduit_discovery::DiscoveredPeer::new(
      NodeId::random(),
      "relay".into(),
      conduit_mesh::capabilities::RELAY,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 1 },
    ));
    relay.link_quality = conduit_mesh::LinkQuality::new(0.8);
    let plain = Neighbor::from_discovered(&conduit_discovery::DiscoveredPeer::new(
      NodeId::random(),
      "plain".into(),
      0,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 2 },
    ));
    assert!(NeighborScorer::score(&relay) > NeighborScorer::score(&plain));
  }
}
