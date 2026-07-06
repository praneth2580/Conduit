use crate::config::MeshConfig;
use crate::events::{MeshEvent, MeshTick, NeighborRemovalReason};
use crate::neighbor::{Neighbor, BROADCAST_NODE_ID};
use crate::table::NeighborTable;
use conduit_core::error::Result;
use conduit_core::ids::NodeId;
use conduit_core::utils::unix_timestamp_ms;
use conduit_discovery::DiscoveryEvent;
use conduit_packets::{HeartbeatPayload, PacketBody, PacketType, TypedPacket};

/// Maintains the neighbor table and heartbeat state.
///
/// Does not route or forward packets — it only tracks who is nearby.
pub struct MeshEngine {
  config: MeshConfig,
  neighbors: NeighborTable,
  started_at_ms: u64,
  last_heartbeat_ms: u64,
  running: bool,
}

impl MeshEngine {
  pub fn new(config: MeshConfig) -> Result<Self> {
    config.validate()?;
    Ok(Self {
      config,
      neighbors: NeighborTable::new(),
      started_at_ms: 0,
      last_heartbeat_ms: 0,
      running: false,
    })
  }

  pub fn config(&self) -> &MeshConfig {
    &self.config
  }

  pub fn local_node_id(&self) -> NodeId {
    self.config.local_node_id
  }

  pub fn is_running(&self) -> bool {
    self.running
  }

  pub fn neighbor_count(&self) -> usize {
    self.neighbors.len()
  }

  pub fn neighbors(&self) -> impl Iterator<Item = &Neighbor> {
    self.neighbors.values()
  }

  pub fn get_neighbor(&self, node_id: &NodeId) -> Option<&Neighbor> {
    self.neighbors.get(node_id)
  }

  pub fn is_neighbor(&self, node_id: &NodeId) -> bool {
    self.neighbors.contains(node_id)
  }

  pub fn start(&mut self) -> MeshTick {
    let now = unix_timestamp_ms();
    self.running = true;
    self.started_at_ms = now;
    self.last_heartbeat_ms = 0;
    MeshTick::default()
  }

  pub fn stop(&mut self) -> MeshTick {
    self.running = false;
    let mut events = Vec::new();
    let ids: Vec<NodeId> = self.neighbors.values().map(|n| n.node_id).collect();
    for node_id in ids {
      self.neighbors.remove(&node_id);
      events.push(MeshEvent::NeighborRemoved {
        node_id,
        reason: NeighborRemovalReason::Stopped,
      });
    }
    MeshTick { events }
  }

  /// Apply a discovery-layer event to the neighbor table.
  pub fn ingest_discovery(&mut self, event: &DiscoveryEvent) -> MeshTick {
    let mut events = Vec::new();
    match event {
      DiscoveryEvent::PeerFound(peer) => {
        if peer.node_id == self.config.local_node_id {
          return MeshTick { events };
        }
        if let Some(existing) = self.neighbors.get_mut(&peer.node_id) {
          existing.node_name = peer.node_name.clone();
          existing.capabilities = peer.capabilities;
          existing.endpoint = peer.endpoint.clone();
          existing.touch(peer.last_seen_ms);
          if let Some(rssi) = peer.signal_strength {
            existing.update_signal(rssi);
          }
          events.push(MeshEvent::NeighborUpdated {
            node_id: peer.node_id,
            link_quality: existing.link_quality.value(),
            signal_quality: existing.signal_quality.value(),
            state: existing.state,
          });
        } else {
          let neighbor = Neighbor::from_discovered(peer);
          let node_id = neighbor.node_id;
          self.neighbors.insert(neighbor.clone());
          events.push(MeshEvent::NeighborAdded(neighbor));
          let _ = node_id;
        }
      }
      DiscoveryEvent::PeerLost { node_id, .. } => {
        if self.neighbors.remove(node_id).is_some() {
          events.push(MeshEvent::NeighborRemoved {
            node_id: *node_id,
            reason: NeighborRemovalReason::DiscoveryLost,
          });
        }
      }
      _ => {}
    }
    MeshTick { events }
  }

  /// Process an inbound heartbeat packet from a neighbor.
  pub fn on_heartbeat_received(&mut self, packet: &TypedPacket) -> Result<MeshTick> {
    if packet.packet_type() != PacketType::Heartbeat {
      return Err(conduit_core::ConduitError::InvalidPacket(
        "expected heartbeat packet".into(),
      ));
    }
    let source = packet.header.source;
    if source == self.config.local_node_id {
      return Ok(MeshTick::default());
    }

    let now = unix_timestamp_ms();
    let mut events = Vec::new();
    let alpha = self.config.link_quality_alpha;

    if let Some(neighbor) = self.neighbors.get_mut(&source) {
      neighbor.record_heartbeat(now, alpha);
      events.push(MeshEvent::NeighborUpdated {
        node_id: source,
        link_quality: neighbor.link_quality.value(),
        signal_quality: neighbor.signal_quality.value(),
        state: neighbor.state,
      });
    } else {
      let mut neighbor = Neighbor {
        node_id: source,
        node_name: format!("node-{}", source),
        capabilities: 0,
        endpoint: conduit_discovery::PeerEndpoint::Simulated { id: 0 },
        state: crate::neighbor::NeighborState::Discovered,
        signal_strength: None,
        signal_quality: crate::quality::SignalQuality::UNKNOWN,
        link_quality: crate::quality::LinkQuality::UNKNOWN,
        discovered_at_ms: now,
        last_seen_ms: now,
        last_heartbeat_received_ms: Some(now),
        heartbeats_received: 1,
        missed_heartbeats: 0,
      };
      neighbor.record_heartbeat(now, alpha);
      let node_id = neighbor.node_id;
      self.neighbors.insert(neighbor.clone());
      events.push(MeshEvent::NeighborAdded(neighbor));
      let _ = node_id;
    }

    Ok(MeshTick { events })
  }

  /// Build a heartbeat packet for the transport layer to send.
  pub fn build_heartbeat(&self) -> Result<TypedPacket> {
    let uptime = unix_timestamp_ms().saturating_sub(self.started_at_ms);
    TypedPacket::with_body(
      PacketType::Heartbeat,
      self.config.local_node_id,
      BROADCAST_NODE_ID,
      PacketBody::Heartbeat(HeartbeatPayload {
        uptime_ms: uptime,
        neighbor_count: self.neighbors.len() as u16,
      }),
    )
  }

  /// Periodic maintenance: emit heartbeats, expire stale neighbors.
  pub fn tick(&mut self) -> Result<MeshTick> {
    if !self.running {
      return Ok(MeshTick::default());
    }

    let now = unix_timestamp_ms();
    let mut events = Vec::new();

    if now.saturating_sub(self.last_heartbeat_ms) >= self.config.heartbeat_interval_ms {
      let heartbeat = self.build_heartbeat()?;
      self.last_heartbeat_ms = now;
      events.push(MeshEvent::HeartbeatReady(heartbeat));
    }

    let timeout = self.config.neighbor_timeout_ms;
    let min_quality = self.config.min_link_quality;
    let alpha = self.config.link_quality_alpha;

    let stale_ids: Vec<NodeId> = self
      .neighbors
      .values()
      .filter(|n| n.is_stale(timeout, now))
      .map(|n| n.node_id)
      .collect();

    for node_id in stale_ids {
      if let Some(neighbor) = self.neighbors.get_mut(&node_id) {
        neighbor.record_missed_heartbeat(alpha);
        if neighbor.link_quality.value() < min_quality {
          self.neighbors.remove(&node_id);
          events.push(MeshEvent::NeighborRemoved {
            node_id,
            reason: NeighborRemovalReason::LowLinkQuality,
          });
        } else {
          events.push(MeshEvent::NeighborUpdated {
            node_id,
            link_quality: neighbor.link_quality.value(),
            signal_quality: neighbor.signal_quality.value(),
            state: neighbor.state,
          });
        }
      }
    }

    let expired: Vec<NodeId> = self
      .neighbors
      .values()
      .filter(|n| n.is_stale(timeout * 2, now))
      .map(|n| n.node_id)
      .collect();

    for node_id in expired {
      self.neighbors.remove(&node_id);
      events.push(MeshEvent::NeighborRemoved {
        node_id,
        reason: NeighborRemovalReason::TimedOut,
      });
    }

    Ok(MeshTick { events })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_discovery::{DiscoveredPeer, DriverKind, PeerEndpoint};
  use conduit_packets::PacketBuilder;

  fn test_config() -> MeshConfig {
    MeshConfig::builder()
      .heartbeat_interval_ms(100)
      .neighbor_timeout_ms(500)
      .build()
  }

  #[test]
  fn discovery_adds_neighbor() {
    let mut mesh = MeshEngine::new(test_config()).unwrap();
    mesh.start();
    let peer_id = NodeId::random();
    let tick = mesh.ingest_discovery(&DiscoveryEvent::PeerFound(DiscoveredPeer::new(
      peer_id,
      "peer".into(),
      1,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 1 },
    )));
    assert_eq!(mesh.neighbor_count(), 1);
    assert!(tick.events.iter().any(|e| matches!(e, MeshEvent::NeighborAdded(_))));
  }

  #[test]
  fn heartbeat_promotes_neighbor_to_active() {
    let local_id = NodeId::random();
    let remote_id = NodeId::random();
    let config = MeshConfig::builder()
      .local_node_id(local_id)
      .build();
    let mut mesh = MeshEngine::new(config).unwrap();
    mesh.start();

    let heartbeat = PacketBuilder::new(PacketType::Heartbeat, remote_id, BROADCAST_NODE_ID)
      .body(PacketBody::Heartbeat(HeartbeatPayload {
        uptime_ms: 1_000,
        neighbor_count: 0,
      }))
      .build()
      .unwrap();

    mesh.on_heartbeat_received(&heartbeat).unwrap();
    let neighbor = mesh.get_neighbor(&remote_id).unwrap();
    assert_eq!(neighbor.state, crate::neighbor::NeighborState::Active);
  }

  #[test]
  fn tick_emits_heartbeat_when_due() {
    let mut mesh = MeshEngine::new(test_config()).unwrap();
    mesh.start();
    let tick = mesh.tick().unwrap();
    assert!(tick
      .events
      .iter()
      .any(|e| matches!(e, MeshEvent::HeartbeatReady(_))));
  }

  #[test]
  fn stale_neighbor_removed() {
    let mut mesh = MeshEngine::new(MeshConfig::builder()
      .neighbor_timeout_ms(10)
      .heartbeat_interval_ms(5)
      .min_link_quality(0.0)
      .build())
      .unwrap();
    mesh.start();
    let peer_id = NodeId::random();
    mesh.ingest_discovery(&DiscoveryEvent::PeerFound(DiscoveredPeer::new(
      peer_id,
      "peer".into(),
      1,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: 1 },
    )));
    std::thread::sleep(std::time::Duration::from_millis(25));
    let tick = mesh.tick().unwrap();
    assert!(tick.events.iter().any(|e| matches!(
      e,
      MeshEvent::NeighborRemoved { node_id, .. } if *node_id == peer_id
    )));
  }
}
