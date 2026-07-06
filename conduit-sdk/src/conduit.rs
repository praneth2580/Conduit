use crate::config::SdkConfig;
use crate::events::SdkEvent;
use crate::network::{NetworkBackend, NullNetwork, SimBusHandle, SimNetwork};
use conduit_core::error::{ConduitError, Result};
use conduit_core::logging::init_logging;
use conduit_core::{NodeId, Packet, PacketPriority, PacketSequence, PacketType};
use conduit_discovery::{
  DiscoveryAnnouncement, DiscoveryEngine, DiscoveryEvent, DiscoveredPeer, MockDriver,
};
use conduit_mesh::{MeshEngine, MeshEvent, MeshTick, NeighborRemovalReason, BROADCAST_NODE_ID};
use conduit_packets::{
  EmergencyPayload, LocationPayload, MessagingPayload, PacketBody, PacketBuilder, PacketCodec,
  TypedPacket,
};
use conduit_routing::{RoutingAction, RoutingDropReason, RoutingEngine};
use conduit_security::{Identity, SecurityEngine};
use conduit_transport::TransportEngine;
use conduit_voice::{LinearCodec, VoiceEngine};

/// Lifecycle state of a [`Conduit`] instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConduitState {
  Initialized,
  Joined,
}

/// Application-facing entry point for the Conduit mesh platform.
///
/// Applications use this type exclusively — they never touch routing or
/// transport internals directly.
pub struct Conduit {
  config: SdkConfig,
  state: ConduitState,
  discovery: DiscoveryEngine,
  mesh: MeshEngine,
  routing: RoutingEngine,
  transport: TransportEngine,
  #[allow(dead_code)]
  security: SecurityEngine,
  voice: VoiceEngine<LinearCodec>,
  network: Box<dyn NetworkBackend>,
  sim_bus: Option<SimBusHandle>,
  outbound_sequence: PacketSequence,
  message_id: u64,
  pending_events: Vec<SdkEvent>,
}

impl Conduit {
  /// Create and initialize a Conduit node from configuration.
  ///
  /// This mirrors `Conduit.initialize()` from the platform specification.
  pub fn initialize(config: SdkConfig) -> Result<Self> {
    config.validate()?;
    init_logging(config.core.log_level);

    let node_id = config.core.node_id;
    let identity = match config.identity_seed {
      Some(seed) => Identity::from_seed(node_id, seed),
      None => Identity::generate(node_id),
    };

    let discovery = if config.simulation {
      DiscoveryEngine::new(config.discovery.clone(), Box::new(MockDriver::new()))?
    } else {
      DiscoveryEngine::from_config(config.discovery.clone())?
    };

    let mesh = MeshEngine::new(config.mesh.clone())?;
    let routing = RoutingEngine::new(config.routing.clone())?;
    let transport = TransportEngine::new(config.transport.clone())?;
    let security = SecurityEngine::new(config.security.clone(), identity)?;
    let voice = VoiceEngine::new_linear(config.voice.clone())?;

    let (network, sim_bus): (Box<dyn NetworkBackend>, Option<SimBusHandle>) = if config.simulation
    {
      let bus = config.sim_bus.clone().unwrap_or_else(SimBusHandle::new);
      let net = SimNetwork::new(node_id, bus.clone());
      (Box::new(net), Some(bus))
    } else {
      (Box::new(NullNetwork), None)
    };

    Ok(Self {
      config,
      state: ConduitState::Initialized,
      discovery,
      mesh,
      routing,
      transport,
      security,
      voice,
      network,
      sim_bus,
      outbound_sequence: PacketSequence::ZERO,
      message_id: 0,
      pending_events: Vec::new(),
    })
  }

  pub fn state(&self) -> ConduitState {
    self.state
  }

  pub fn is_joined(&self) -> bool {
    self.state == ConduitState::Joined
  }

  pub fn node_id(&self) -> NodeId {
    self.config.core.node_id
  }

  pub fn config(&self) -> &SdkConfig {
    &self.config
  }

  pub fn voice(&self) -> &VoiceEngine<LinearCodec> {
    &self.voice
  }

  pub fn voice_mut(&mut self) -> &mut VoiceEngine<LinearCodec> {
    &mut self.voice
  }

  pub fn set_push_to_talk(&mut self, active: bool) {
    self.voice.set_push_to_talk(active);
  }

  /// Join the mesh network — starts discovery and mesh maintenance.
  pub fn join_network(&mut self) -> Result<()> {
    if self.state == ConduitState::Joined {
      return Ok(());
    }
    self.discovery.start()?;
    self.mesh.start();
    self.state = ConduitState::Joined;
    self.pending_events.push(SdkEvent::NetworkJoined);
    Ok(())
  }

  /// Leave the mesh network.
  pub fn leave_network(&mut self) -> Result<()> {
    if self.state == ConduitState::Initialized {
      return Ok(());
    }
    let mesh_tick = self.mesh.stop();
    self.collect_mesh_tick(mesh_tick);
    self.discovery.stop()?;
    self.state = ConduitState::Initialized;
    self.pending_events.push(SdkEvent::NetworkLeft);
    Ok(())
  }

  /// Capture microphone samples and send a voice frame when ready.
  pub fn send_voice(&mut self, samples: &[i16]) -> Result<()> {
    self.ensure_joined()?;
    if let Some(packet) = self.voice.capture(samples)? {
      self.dispatch_outbound(packet)?;
    }
    Ok(())
  }

  /// Broadcast a GPS location update.
  pub fn send_location(
    &mut self,
    latitude_microdeg: i32,
    longitude_microdeg: i32,
    altitude_m: i16,
    accuracy_m: u16,
  ) -> Result<()> {
    self.ensure_joined()?;
    let packet = TypedPacket::with_body(
      PacketType::Location,
      self.node_id(),
      BROADCAST_NODE_ID,
      PacketBody::Location(LocationPayload {
        latitude_microdeg,
        longitude_microdeg,
        altitude_m,
        accuracy_m,
      }),
    )?;
    self.dispatch_outbound(packet)
  }

  /// Broadcast a high-priority emergency signal.
  pub fn send_emergency(&mut self, emergency: EmergencyPayload) -> Result<()> {
    self.ensure_joined()?;
    let packet = PacketBuilder::new(PacketType::Emergency, self.node_id(), BROADCAST_NODE_ID)
      .priority(PacketPriority::Critical)
      .body(PacketBody::Emergency(emergency))
      .build()?;
    self.dispatch_outbound(packet)
  }

  /// Send a text message to all nearby nodes.
  pub fn send_broadcast(&mut self, content: impl Into<String>) -> Result<()> {
    self.send_message(BROADCAST_NODE_ID, content)
  }

  /// Send a text message to a specific peer.
  pub fn send_message(&mut self, to: NodeId, content: impl Into<String>) -> Result<()> {
    self.ensure_joined()?;
    self.message_id = self.message_id.wrapping_add(1);
    let packet = PacketBuilder::new(PacketType::Messaging, self.node_id(), to)
      .body(PacketBody::Messaging(MessagingPayload {
        message_id: self.message_id,
        content: content.into(),
      }))
      .build()?;
    self.dispatch_outbound(packet)
  }

  /// Advance discovery, mesh, routing, and inbound I/O. Returns new SDK events.
  pub fn tick(&mut self) -> Result<Vec<SdkEvent>> {
    if self.state != ConduitState::Joined {
      return Ok(self.take_pending_events());
    }

    for event in self.discovery.tick()? {
      self.handle_discovery_event(event);
    }

    let mesh_tick = self.mesh.tick()?;
    for event in &mesh_tick.events {
      if let MeshEvent::HeartbeatReady(packet) = event {
        self.dispatch_outbound(packet.clone())?;
      }
    }
    self.collect_mesh_tick(mesh_tick);

    for frame in self.network.drain_inbound() {
      if let Some(packet) = self.transport.decode_frame(&frame.data)? {
        self.process_inbound(frame.from, packet)?;
      }
    }

    Ok(self.take_pending_events())
  }

  /// Drain events accumulated since the last call.
  pub fn poll_events(&mut self) -> Vec<SdkEvent> {
    self.take_pending_events()
  }

  /// Connect a simulated peer directly (simulation mode only).
  pub fn connect_peer(&mut self, peer: DiscoveredPeer) -> Result<()> {
    if !self.config.simulation {
      return Err(ConduitError::Configuration(
        "connect_peer is only available in simulation mode".into(),
      ));
    }
    let tick = self
      .mesh
      .ingest_discovery(&DiscoveryEvent::PeerFound(peer));
    self.collect_mesh_tick(tick);
    Ok(())
  }

  /// Link two simulation-mode nodes for bidirectional mesh connectivity.
  pub fn link_simulated_peers(a: &mut Conduit, b: &mut Conduit) -> Result<()> {
    if !a.config.simulation || !b.config.simulation {
      return Err(ConduitError::Configuration(
        "link_simulated_peers requires simulation mode".into(),
      ));
    }

    let ann_a = a.discovery.config().announcement();
    let ann_b = b.discovery.config().announcement();

    a.inject_mock_peer(ann_b)?;
    b.inject_mock_peer(ann_a)?;
    Ok(())
  }

  /// Shared simulation bus when running in simulation mode.
  pub fn sim_bus(&self) -> Option<&SimBusHandle> {
    self.sim_bus.as_ref()
  }

  fn inject_mock_peer(&mut self, announcement: DiscoveryAnnouncement) -> Result<()> {
    let peer = DiscoveredPeer::new(
      announcement.node_id,
      announcement.node_name,
      announcement.capabilities,
      conduit_discovery::DriverKind::Mock,
      PeerEndpoint::Simulated {
        id: u64::from_be_bytes({
          let b = *announcement.node_id.as_bytes();
          [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
        }),
      },
    );
    self.connect_peer(peer)
  }

  fn ensure_joined(&self) -> Result<()> {
    if self.state != ConduitState::Joined {
      return Err(ConduitError::Configuration(
        "must join the network before sending".into(),
      ));
    }
    Ok(())
  }

  fn take_pending_events(&mut self) -> Vec<SdkEvent> {
    std::mem::take(&mut self.pending_events)
  }

  fn handle_discovery_event(&mut self, event: DiscoveryEvent) {
    match &event {
      DiscoveryEvent::PeerFound(peer) => {
        self.pending_events.push(SdkEvent::PeerDiscovered {
          node_id: peer.node_id,
          name: peer.node_name.clone(),
        });
      }
      DiscoveryEvent::PeerLost { node_id, .. } => {
        self.pending_events.push(SdkEvent::PeerLost { node_id: *node_id });
      }
      _ => {}
    }
    let tick = self.mesh.ingest_discovery(&event);
    self.collect_mesh_tick(tick);
  }

  fn collect_mesh_tick(&mut self, tick: MeshTick) {
    for event in tick.events {
      match event {
        MeshEvent::NeighborAdded(neighbor) => {
          self.pending_events.push(SdkEvent::NeighborAdded(neighbor));
        }
        MeshEvent::NeighborRemoved { node_id, reason } => {
          if reason != NeighborRemovalReason::Stopped {
            self.routing.on_neighbor_lost(&node_id);
          }
          self.pending_events.push(SdkEvent::NeighborRemoved { node_id });
        }
        _ => {}
      }
    }
  }

  fn dispatch_outbound(&mut self, mut typed: TypedPacket) -> Result<()> {
    typed.header.sequence = self.outbound_sequence;
    self.outbound_sequence = self.outbound_sequence.next();
    typed.header.ttl = self.config.core.default_ttl;

    let packet = PacketCodec::encode(&typed)?;
    let neighbors: Vec<_> = self.mesh.neighbors().cloned().collect();
    match self.routing.process_outbound(packet, &neighbors) {
      RoutingAction::Deliver(_) => Ok(()),
      RoutingAction::Forward { packet, next_hops } => self.transmit_packet(&packet, &next_hops),
      RoutingAction::DeliverAndForward { forward, next_hops, .. } => {
        self.transmit_packet(&forward, &next_hops)
      }
      RoutingAction::Drop(reason) => Err(ConduitError::InvalidPacket(drop_reason_message(reason))),
    }
  }

  fn process_inbound(&mut self, from: NodeId, packet: Packet) -> Result<()> {
    let neighbors: Vec<_> = self.mesh.neighbors().cloned().collect();
    match self.routing.process_inbound(packet, from, &neighbors) {
      RoutingAction::Deliver(packet) => self.deliver_local(packet),
      RoutingAction::DeliverAndForward {
        deliver,
        forward,
        next_hops,
      } => {
        self.deliver_local(deliver)?;
        self.transmit_packet(&forward, &next_hops)
      }
      RoutingAction::Forward { packet, next_hops } => self.transmit_packet(&packet, &next_hops),
      RoutingAction::Drop(_) => Ok(()),
    }
  }

  fn deliver_local(&mut self, packet: Packet) -> Result<()> {
    let typed = PacketCodec::decode(&packet)?;
    let from = typed.header.source;

    match &typed.body {
      PacketBody::Heartbeat(_) => {
        let tick = self.mesh.on_heartbeat_received(&typed)?;
        self.collect_mesh_tick(tick);
      }
      PacketBody::Voice(_) => {
        self.voice.receive(&typed)?;
        self
          .pending_events
          .push(SdkEvent::VoiceFrameReceived { from });
      }
      PacketBody::Location(location) => {
        self.pending_events.push(SdkEvent::LocationReceived {
          from,
          location: location.clone(),
        });
      }
      PacketBody::Emergency(emergency) => {
        self.pending_events.push(SdkEvent::EmergencyReceived {
          from,
          emergency: emergency.clone(),
        });
      }
      PacketBody::Messaging(message) => {
        self.pending_events.push(SdkEvent::MessageReceived {
          from,
          message: message.clone(),
        });
      }
      _ => {}
    }
    Ok(())
  }

  fn transmit_packet(&mut self, packet: &Packet, hops: &[NodeId]) -> Result<()> {
    let frames = self.transport.encode_packet(packet)?;
    for hop in hops {
      self.network.send_frames(*hop, &frames)?;
      self.routing.on_forward_complete(*hop);
    }
    Ok(())
  }
}

fn drop_reason_message(reason: RoutingDropReason) -> String {
  match reason {
    RoutingDropReason::Duplicate => "duplicate packet".into(),
    RoutingDropReason::TtlExpired => "ttl expired".into(),
    RoutingDropReason::NoRoute => "no route to destination".into(),
    RoutingDropReason::LoopDetected => "routing loop detected".into(),
    RoutingDropReason::Congested => "neighbor congested".into(),
    RoutingDropReason::InvalidPacket => "invalid packet".into(),
  }
}

use conduit_discovery::PeerEndpoint;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::network::SimBusHandle;
  use conduit_discovery::DriverKind;
  use conduit_packets::EmergencyKind;

  fn sim_node(name: &str, seed: u8, bus: &SimBusHandle) -> Conduit {
    let mut id_bytes = [0u8; 16];
    id_bytes[0] = seed;
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    let config = SdkConfig::builder()
      .node_id(NodeId::from_bytes(id_bytes))
      .node_name(name)
      .simulation(true)
      .sim_bus(bus.clone())
      .identity_seed(seed_bytes)
      .build();
    Conduit::initialize(config).unwrap()
  }

  fn discovered_peer(other: &Conduit, sim_id: u64) -> DiscoveredPeer {
    DiscoveredPeer::new(
      other.node_id(),
      other.config().node_name.clone(),
      other.config().capabilities,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: sim_id },
    )
  }

  #[test]
  fn initialize_and_join() {
    let bus = SimBusHandle::new();
    let mut node = sim_node("alpha", 1, &bus);
    assert_eq!(node.state(), ConduitState::Initialized);
    node.join_network().unwrap();
    assert!(node.is_joined());
    let events = node.poll_events();
    assert!(events.contains(&SdkEvent::NetworkJoined));
  }

  #[test]
  fn location_reaches_peer() {
    let bus = SimBusHandle::new();
    let mut a = sim_node("alice", 10, &bus);
    let mut b = sim_node("bob", 20, &bus);
    a.join_network().unwrap();
    b.join_network().unwrap();

    a.connect_peer(discovered_peer(&b, 20)).unwrap();
    b.connect_peer(discovered_peer(&a, 10)).unwrap();

    a.send_location(12_345_678, -98_765_432, 100, 5)
      .unwrap();
    a.tick().unwrap();
    let events = b.tick().unwrap();

    assert!(events.iter().any(|e| matches!(
      e,
      SdkEvent::LocationReceived { location, .. }
        if location.latitude_microdeg == 12_345_678
    )));
  }

  #[test]
  fn emergency_broadcast() {
    let bus = SimBusHandle::new();
    let mut a = sim_node("sender", 1, &bus);
    let mut b = sim_node("receiver", 2, &bus);
    a.join_network().unwrap();
    b.join_network().unwrap();
    a.connect_peer(discovered_peer(&b, 2)).unwrap();
    b.connect_peer(discovered_peer(&a, 1)).unwrap();

    a.send_emergency(EmergencyPayload {
      kind: EmergencyKind::Medical,
      message: "need help".into(),
    })
    .unwrap();
    a.tick().unwrap();
    let events = b.tick().unwrap();

    assert!(events.iter().any(|e| matches!(
      e,
      SdkEvent::EmergencyReceived { emergency, .. }
        if emergency.message == "need help"
    )));
  }

  #[test]
  fn direct_message_delivery() {
    let bus = SimBusHandle::new();
    let mut a = sim_node("alice", 3, &bus);
    let mut b = sim_node("bob", 4, &bus);
    a.join_network().unwrap();
    b.join_network().unwrap();
    a.connect_peer(discovered_peer(&b, 4)).unwrap();
    b.connect_peer(discovered_peer(&a, 3)).unwrap();

    a.send_message(b.node_id(), "hello bob").unwrap();
    a.tick().unwrap();
    let events = b.tick().unwrap();

    assert!(events.iter().any(|e| matches!(
      e,
      SdkEvent::MessageReceived { message, .. } if message.content == "hello bob"
    )));
  }

  #[test]
  fn voice_frame_round_trip() {
    let bus = SimBusHandle::new();
    let mut a = sim_node("talker", 5, &bus);
    let mut b = sim_node("listener", 6, &bus);
    a.join_network().unwrap();
    b.join_network().unwrap();
    a.connect_peer(discovered_peer(&b, 6)).unwrap();
    b.connect_peer(discovered_peer(&a, 5)).unwrap();

    let frame_samples = a.voice().config().frame_samples();
    let samples = vec![1000i16; frame_samples];
    for _ in 0..3 {
      a.send_voice(&samples).unwrap();
      a.tick().unwrap();
      b.tick().unwrap();
    }

    assert!(b
      .voice_mut()
      .playback()
      .unwrap()
      .is_some());
  }
}
