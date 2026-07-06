use crate::driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
  PeerEndpoint,
};
use crate::peer::DiscoveredPeer;
use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use std::collections::{HashMap, VecDeque};

/// In-memory discovery driver for tests and simulations.
#[derive(Debug, Default)]
pub struct MockDriver {
  state: DiscoveryState,
  local: Option<DiscoveryAnnouncement>,
  peers: HashMap<NodeId, DiscoveredPeer>,
  pending: VecDeque<DiscoveryEvent>,
}

impl MockDriver {
  pub fn new() -> Self {
    Self::default()
  }

  /// Inject a remote peer as if it were discovered on the network.
  pub fn inject_peer(&mut self, announcement: DiscoveryAnnouncement) {
    let peer = DiscoveredPeer::new(
      announcement.node_id,
      announcement.node_name,
      announcement.capabilities,
      DriverKind::Mock,
      PeerEndpoint::Simulated {
        id: u64::from_be_bytes({
          let b = *announcement.node_id.as_bytes();
          [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
        }),
      },
    );
    self.peers.insert(announcement.node_id, peer.clone());
    self
      .pending
      .push_back(DiscoveryEvent::PeerFound(peer));
  }

  /// Connect two mock drivers for bidirectional discovery in tests.
  pub fn link_to(other: &mut MockDriver, local: &DiscoveryAnnouncement) {
    other.inject_peer(local.clone());
  }
}

impl DiscoveryDriver for MockDriver {
  fn kind(&self) -> DriverKind {
    DriverKind::Mock
  }

  fn state(&self) -> DiscoveryState {
    self.state.clone()
  }

  fn start(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()> {
    if matches!(self.state, DiscoveryState::Running) {
      return Ok(());
    }
    self.local = Some(announcement.clone());
    self.state = DiscoveryState::Running;
    self
      .pending
      .push_back(DiscoveryEvent::DriverStarted {
        driver: DriverKind::Mock,
      });
    Ok(())
  }

  fn stop(&mut self) -> Result<()> {
    if matches!(self.state, DiscoveryState::Stopped) {
      return Ok(());
    }
    self.state = DiscoveryState::Stopped;
    self.local = None;
    self.peers.clear();
    self
      .pending
      .push_back(DiscoveryEvent::DriverStopped {
        driver: DriverKind::Mock,
      });
    Ok(())
  }

  fn announce(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()> {
    if !matches!(self.state, DiscoveryState::Running) {
      return Err(ConduitError::Configuration(
        "mock driver is not running".into(),
      ));
    }
    self.local = Some(announcement.clone());
    Ok(())
  }

  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>> {
    Ok(self.pending.drain(..).collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inject_peer_queues_event() {
    let mut driver = MockDriver::new();
    let announcement = DiscoveryAnnouncement {
      node_id: NodeId::random(),
      node_name: "peer".into(),
      capabilities: 1,
    };
    driver.inject_peer(announcement);
    let events = driver.poll().unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], DiscoveryEvent::PeerFound(_)));
  }
}
