use crate::config::RoutingConfig;
use crate::congestion::CongestionTracker;
use crate::decision::{RoutingAction, RoutingDropReason};
use crate::duplicate::DuplicateCache;
use crate::routes::RouteTable;
use crate::scoring::NeighborScorer;
use conduit_core::error::Result;
use conduit_core::ids::NodeId;
use conduit_core::utils::unix_timestamp_ms;
use conduit_core::{Packet, PacketType};
use conduit_mesh::{Neighbor, BROADCAST_NODE_ID};
use conduit_packets::{PacketBody, RoutingPayload};

/// Core routing engine — header-only packet processing.
pub struct RoutingEngine {
  config: RoutingConfig,
  duplicates: DuplicateCache,
  routes: RouteTable,
  congestion: CongestionTracker,
}

impl RoutingEngine {
  pub fn new(config: RoutingConfig) -> Result<Self> {
    config.validate()?;
    Ok(Self {
      duplicates: DuplicateCache::new(config.duplicate_window_ms, config.duplicate_cache_size),
      congestion: CongestionTracker::new(config.congestion_limit),
      routes: RouteTable::new(),
      config,
    })
  }

  pub fn config(&self) -> &RoutingConfig {
    &self.config
  }

  pub fn routes(&self) -> &RouteTable {
    &self.routes
  }

  pub fn routes_mut(&mut self) -> &mut RouteTable {
    &mut self.routes
  }

  pub fn congestion(&self) -> &CongestionTracker {
    &self.congestion
  }

  pub fn congestion_mut(&mut self) -> &mut CongestionTracker {
    &mut self.congestion
  }

  /// Route a locally originated packet.
  pub fn process_outbound(&mut self, packet: Packet, neighbors: &[Neighbor]) -> RoutingAction {
    let now = unix_timestamp_ms();
    self.routes.remove_stale(self.config.route_stale_ms, now);
    self.forward_packet(packet, None, neighbors, now, false)
  }

  /// Route a packet received from a neighbor.
  pub fn process_inbound(
    &mut self,
    packet: Packet,
    received_from: NodeId,
    neighbors: &[Neighbor],
  ) -> RoutingAction {
    let now = unix_timestamp_ms();
    self.routes.remove_stale(self.config.route_stale_ms, now);

    if packet.header.source == self.config.local_node_id {
      return RoutingAction::Drop(RoutingDropReason::LoopDetected);
    }

    if self.duplicates.is_duplicate(
      packet.header.source,
      packet.header.sequence,
      now,
    ) {
      return RoutingAction::Drop(RoutingDropReason::Duplicate);
    }

    self.learn_from_packet(&packet, received_from, now);
    self.forward_packet(packet, Some(received_from), neighbors, now, true)
  }

  /// Notify the router that a neighbor is no longer reachable.
  pub fn on_neighbor_lost(&mut self, neighbor: &NodeId) {
    self.routes.remove_neighbor_routes(neighbor);
  }

  /// Mark a forward to a neighbor as completed (congestion tracking).
  pub fn on_forward_complete(&mut self, neighbor: NodeId) {
    self.congestion.record_complete(neighbor);
  }

  fn learn_from_packet(&mut self, packet: &Packet, via: NodeId, now_ms: u64) {
    self.routes.learn(packet.header.source, via, 1, now_ms);

    if packet.packet_type() == PacketType::Routing {
      if let Ok(body) = PacketBody::from_bytes(PacketType::Routing, &packet.payload) {
        if let PacketBody::Routing(payload) = body {
          self.ingest_routing_payload(&payload, via, now_ms);
        }
      }
    }
  }

  fn ingest_routing_payload(&mut self, payload: &RoutingPayload, via: NodeId, now_ms: u64) {
    if payload
      .path
      .iter()
      .any(|id| *id == self.config.local_node_id)
    {
      return;
    }
    if let Some(&destination) = payload.path.last() {
      let hops = payload.hop_count.saturating_add(1);
      self.routes.learn(destination, via, hops, now_ms);
    }
  }

  fn forward_packet(
    &mut self,
    mut packet: Packet,
    received_from: Option<NodeId>,
    neighbors: &[Neighbor],
    now_ms: u64,
    from_network: bool,
  ) -> RoutingAction {
    let destination = packet.header.destination;
    let is_broadcast = destination == BROADCAST_NODE_ID;
    let is_local = destination == self.config.local_node_id;

    if is_local {
      return RoutingAction::Deliver(packet);
    }

    if !is_broadcast && packet.header.ttl == 0 {
      return RoutingAction::Drop(RoutingDropReason::TtlExpired);
    }

    let exclude: Vec<NodeId> = received_from.into_iter().collect();
    let next_hops = self.select_next_hops(&packet, neighbors, &exclude);

    if is_broadcast {
      if packet.header.ttl == 0 || next_hops.is_empty() {
        return RoutingAction::Deliver(packet);
      }
      let mut forward = packet.clone();
      forward.header.ttl -= 1;
      for hop in &next_hops {
        self.congestion.record_send(*hop);
      }
      let _ = (now_ms, from_network);
      return RoutingAction::DeliverAndForward {
        deliver: packet,
        forward,
        next_hops,
      };
    }

    if next_hops.is_empty() {
      return RoutingAction::Drop(RoutingDropReason::NoRoute);
    }

    packet.header.ttl -= 1;
    for hop in &next_hops {
      self.congestion.record_send(*hop);
    }

    let _ = now_ms;
    RoutingAction::Forward {
      packet,
      next_hops,
    }
  }

  fn select_next_hops(
    &self,
    packet: &Packet,
    neighbors: &[Neighbor],
    exclude: &[NodeId],
  ) -> Vec<NodeId> {
    let destination = packet.header.destination;

    if neighbors.iter().any(|n| n.node_id == destination) {
      return vec![destination];
    }

    if let Some(route) = self.routes.get(&destination) {
      if !exclude.contains(&route.next_hop) && !self.congestion.is_congested(&route.next_hop) {
        return vec![route.next_hop];
      }
    }

    if !self.config.enable_flood_fallback {
      return Vec::new();
    }

    NeighborScorer::rank(neighbors, exclude)
      .into_iter()
      .filter(|(n, _)| !self.congestion.is_congested(&n.node_id))
      .take(2)
      .map(|(n, _)| n.node_id)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_discovery::{DiscoveredPeer, DriverKind, PeerEndpoint};
  use conduit_mesh::Neighbor;

  fn node(name: u8) -> NodeId {
    let mut bytes = [0u8; 16];
    bytes[0] = name;
    NodeId::from_bytes(bytes)
  }

  fn neighbor(id: NodeId, link: f32) -> Neighbor {
    let mut n = Neighbor::from_discovered(&DiscoveredPeer::new(
      id,
      format!("node-{id}"),
      conduit_mesh::capabilities::RELAY,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: id.as_bytes()[0] as u64 },
    ));
    n.link_quality = conduit_mesh::LinkQuality::new(link);
    n.state = conduit_mesh::NeighborState::Active;
    n
  }

  fn packet(from: NodeId, to: NodeId, seq: u32) -> Packet {
    let mut p = Packet::with_payload(PacketType::Messaging, from, to, b"hi".to_vec());
    p.header.sequence = conduit_core::PacketSequence(seq);
    p.header.ttl = 8;
    p
  }

  #[test]
  fn delivers_local_packet() {
    let local = node(b'A');
    let mut router = RoutingEngine::new(RoutingConfig::builder().local_node_id(local).build()).unwrap();
    let pkt = packet(node(b'B'), local, 1);
    match router.process_inbound(pkt, node(b'B'), &[]) {
      RoutingAction::Deliver(_) => {}
      other => panic!("expected deliver, got {other:?}"),
    }
  }

  #[test]
  fn drops_duplicate_packets() {
    let local = node(b'A');
    let from = node(b'B');
    let mut router = RoutingEngine::new(RoutingConfig::builder().local_node_id(local).build()).unwrap();
    let pkt = packet(from, local, 42);
    router.process_inbound(pkt.clone(), from, &[]);
    match router.process_inbound(pkt, from, &[]) {
      RoutingAction::Drop(RoutingDropReason::Duplicate) => {}
      other => panic!("expected duplicate drop, got {other:?}"),
    }
  }

  #[test]
  fn forwards_using_route_table() {
    let a = node(b'A');
    let b = node(b'B');
    let d = node(b'D');
    let mut router = RoutingEngine::new(RoutingConfig::builder().local_node_id(a).build()).unwrap();
    router.routes_mut().learn(d, b, 2, 100);

    let neighbors = vec![neighbor(b, 0.9)];
    let pkt = packet(node(b'X'), d, 1);
    match router.process_outbound(pkt, &neighbors) {
      RoutingAction::Forward { next_hops, .. } => assert_eq!(next_hops, vec![b]),
      other => panic!("expected forward, got {other:?}"),
    }
  }

  #[test]
  fn drops_when_ttl_expired() {
    let a = node(b'A');
    let mut router = RoutingEngine::new(RoutingConfig::builder().local_node_id(a).build()).unwrap();
    let mut pkt = packet(a, node(b'D'), 1);
    pkt.header.ttl = 0;
    match router.process_outbound(pkt, &[neighbor(node(b'B'), 0.9)]) {
      RoutingAction::Drop(RoutingDropReason::TtlExpired) => {}
      other => panic!("expected ttl drop, got {other:?}"),
    }
  }

  #[test]
  fn recovers_when_intermediate_node_lost() {
    let a = node(b'A');
    let b = node(b'B');
    let e = node(b'E');
    let d = node(b'D');

    let mut router = RoutingEngine::new(RoutingConfig::builder().local_node_id(a).build()).unwrap();
    router.routes_mut().learn(d, b, 2, 100);

    router.on_neighbor_lost(&b);
    router.routes_mut().learn(d, e, 2, 200);

    let neighbors = vec![neighbor(e, 0.95)];
    let pkt = packet(a, d, 5);
    match router.process_outbound(pkt, &neighbors) {
      RoutingAction::Forward { next_hops, .. } => assert_eq!(next_hops, vec![e]),
      other => panic!("expected forward via E, got {other:?}"),
    }
  }

  #[test]
  fn learns_route_from_routing_payload() {
    let a = node(b'A');
    let c = node(b'C');
    let d = node(b'D');
    let mut router = RoutingEngine::new(RoutingConfig::builder().local_node_id(a).build()).unwrap();

    let payload = RoutingPayload {
      hop_count: 1,
      path: vec![c, d],
    };
    let body = PacketBody::Routing(payload);
    let bytes = body.to_bytes();
    let pkt = Packet::with_payload(PacketType::Routing, c, BROADCAST_NODE_ID, bytes);

    router.process_inbound(pkt, c, &[]);
    assert_eq!(router.routes().get(&d).unwrap().next_hop, c);
  }
}
