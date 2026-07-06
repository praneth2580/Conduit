use crate::config::DiscoveryConfig;
use crate::driver::{
  DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
};
use crate::peer::DiscoveredPeer;
use conduit_core::error::Result;
use conduit_core::ids::NodeId;
use conduit_core::utils::unix_timestamp_ms;
use std::collections::HashMap;

/// High-level discovery coordinator.
///
/// Owns the active [`DiscoveryDriver`], maintains the peer table, and emits
/// unified events. Callers never interact with driver implementations directly.
pub struct DiscoveryEngine {
  config: DiscoveryConfig,
  driver: Box<dyn DiscoveryDriver>,
  peers: HashMap<NodeId, DiscoveredPeer>,
  last_announce_ms: u64,
  running: bool,
}

impl DiscoveryEngine {
  pub fn new(config: DiscoveryConfig, driver: Box<dyn DiscoveryDriver>) -> Result<Self> {
    config.validate()?;
    Ok(Self {
      config,
      driver,
      peers: HashMap::new(),
      last_announce_ms: 0,
      running: false,
    })
  }

  pub fn from_config(config: DiscoveryConfig) -> Result<Self> {
    let driver = crate::drivers::default_driver_chain(&config)?;
    Self::new(config, driver)
  }

  pub fn config(&self) -> &DiscoveryConfig {
    &self.config
  }

  pub fn active_driver(&self) -> DriverKind {
    self.driver.kind()
  }

  pub fn state(&self) -> DiscoveryState {
    if self.running {
      DiscoveryState::Running
    } else {
      DiscoveryState::Stopped
    }
  }

  pub fn peers(&self) -> impl Iterator<Item = &DiscoveredPeer> {
    self.peers.values()
  }

  pub fn peer_count(&self) -> usize {
    self.peers.len()
  }

  pub fn get_peer(&self, node_id: &NodeId) -> Option<&DiscoveredPeer> {
    self.peers.get(node_id)
  }

  pub fn start(&mut self) -> Result<Vec<DiscoveryEvent>> {
    let announcement = self.config.announcement();
    self.driver.start(&announcement)?;
    self.running = true;
    self.last_announce_ms = 0;
    self.tick()
  }

  pub fn stop(&mut self) -> Result<Vec<DiscoveryEvent>> {
    self.running = false;
  self.peers.clear();
    self.driver.stop()?;
    Ok(vec![DiscoveryEvent::DriverStopped {
      driver: self.driver.kind(),
    }])
  }

  /// Poll the driver, send periodic announcements, and expire stale peers.
  pub fn tick(&mut self) -> Result<Vec<DiscoveryEvent>> {
    if !self.running {
      return Ok(Vec::new());
    }

    let now = unix_timestamp_ms();
    let mut events = self.driver.poll()?;

    if now.saturating_sub(self.last_announce_ms) >= self.config.announce_interval_ms {
      let announcement = self.config.announcement();
      self.driver.announce(&announcement)?;
      self.last_announce_ms = now;
    }

    let mut peer_events = Vec::new();
    for event in events.drain(..) {
      match event {
        DiscoveryEvent::PeerFound(peer) => {
          if peer.node_id == self.config.node_id {
            continue;
          }
          if let Some(existing) = self.peers.get_mut(&peer.node_id) {
            existing.node_name = peer.node_name;
            existing.capabilities = peer.capabilities;
            existing.endpoint = peer.endpoint;
            existing.touch();
          } else {
            self.peers.insert(peer.node_id, peer.clone());
            peer_events.push(DiscoveryEvent::PeerFound(peer));
          }
        }
        other => peer_events.push(other),
      }
    }

    let expired = self.expire_stale_peers(now);
    peer_events.extend(expired);
    Ok(peer_events)
  }

  fn expire_stale_peers(&mut self, now_ms: u64) -> Vec<DiscoveryEvent> {
    let timeout = self.config.peer_timeout_ms;
    let stale_ids: Vec<NodeId> = self
      .peers
      .iter()
      .filter(|(_, peer)| peer.is_stale(timeout, now_ms))
      .map(|(id, _)| *id)
      .collect();

    let mut events = Vec::new();
    for node_id in stale_ids {
      self.peers.remove(&node_id);
      events.push(DiscoveryEvent::PeerLost {
        node_id,
        driver: self.driver.kind(),
      });
    }
    events
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::drivers::MockDriver;
  use crate::DiscoveryAnnouncement;
  use conduit_core::NodeId;

  #[test]
  fn engine_tracks_discovered_peers() {
    let local_id = NodeId::random();
    let peer_id = NodeId::random();

    let mut driver = MockDriver::new();
    driver.inject_peer(DiscoveryAnnouncement {
      node_id: peer_id,
      node_name: "remote".into(),
      capabilities: 1,
    });

    let mut engine = DiscoveryEngine::new(
      DiscoveryConfig::builder()
        .node_id(local_id)
        .node_name("local")
        .build(),
      Box::new(driver),
    )
    .unwrap();
    let events = engine.start().unwrap();
    assert!(
      events
        .iter()
        .any(|e| matches!(e, DiscoveryEvent::PeerFound(p) if p.node_id == peer_id))
    );
    assert_eq!(engine.peer_count(), 1);
  }

  #[test]
  fn engine_expires_stale_peers() {
    let config = DiscoveryConfig::builder()
      .peer_timeout_ms(10)
      .announce_interval_ms(5)
      .build();
    let peer_id = NodeId::random();

    let mut driver = MockDriver::new();
    driver
      .start(&config.announcement())
      .unwrap();
    driver.inject_peer(DiscoveryAnnouncement {
      node_id: peer_id,
      node_name: "remote".into(),
      capabilities: 1,
    });

    let mut engine = DiscoveryEngine::new(config, Box::new(driver)).unwrap();
    engine.start().unwrap();
    engine.tick().unwrap();
    assert_eq!(engine.peer_count(), 1);

    std::thread::sleep(std::time::Duration::from_millis(20));
    let events = engine.tick().unwrap();
    assert!(events.iter().any(|e| matches!(e, DiscoveryEvent::PeerLost { node_id, .. } if *node_id == peer_id)));
    assert_eq!(engine.peer_count(), 0);
  }
}
