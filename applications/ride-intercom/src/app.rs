use crate::config::RideIntercomConfig;
use crate::events::RideEvent;
use crate::group::{self, GroupMessage};
use crate::member::{RideMember, RidePosition};
use conduit_sdk::{
  Conduit, DiscoveredPeer, EmergencyKind, EmergencyPayload, MessagingPayload, SdkEvent,
  SimBusHandle,
};
use conduit_sdk::{NodeId, Result};
use std::collections::HashMap;
use std::time::Instant;

/// Motorcycle group intercom — the first Conduit application.
///
/// Wraps the SDK with ride-specific group management, push-to-talk voice,
/// GPS sharing, and SOS handling. Never touches routing or transport directly.
pub struct RideIntercom {
  config: RideIntercomConfig,
  conduit: Conduit,
  members: HashMap<NodeId, RideMember>,
  running: bool,
  position: Option<RidePosition>,
  last_location_share: Option<Instant>,
  push_to_talk_active: bool,
}

impl RideIntercom {
  /// Create a new intercom session (initializes the underlying Conduit node).
  pub fn new(config: RideIntercomConfig) -> Result<Self> {
    config.validate()?;
    let push_to_talk = config.push_to_talk;
    let mut conduit = Conduit::initialize(config.sdk.clone())?;
    conduit.set_push_to_talk(push_to_talk);

    let mut members = HashMap::new();
    let self_member = RideMember {
      node_id: conduit.node_id(),
      name: config.rider_name.clone(),
      last_location: None,
    };
    members.insert(conduit.node_id(), self_member);

    Ok(Self {
      config,
      conduit,
      members,
      running: false,
      position: None,
      last_location_share: None,
      push_to_talk_active: false,
    })
  }

  pub fn config(&self) -> &RideIntercomConfig {
    &self.config
  }

  pub fn node_id(&self) -> NodeId {
    self.conduit.node_id()
  }

  pub fn is_running(&self) -> bool {
    self.running
  }

  pub fn members(&self) -> impl Iterator<Item = &RideMember> {
    self.members.values()
  }

  pub fn member(&self, node_id: &NodeId) -> Option<&RideMember> {
    self.members.get(node_id)
  }

  pub fn group_name(&self) -> &str {
    &self.config.group_name
  }

  pub fn rider_name(&self) -> &str {
    &self.config.rider_name
  }

  /// Join the mesh and announce membership in the ride group.
  pub fn start(&mut self) -> Result<Vec<RideEvent>> {
    if self.running {
      return Ok(Vec::new());
    }
    self.conduit.join_network()?;
    self.running = true;
    self.broadcast_group_join()?;
    let mut events = vec![RideEvent::SessionStarted {
      group: self.config.group_name.clone(),
    }];
    events.extend(self.tick()?);
    Ok(events)
  }

  /// Leave the group and disconnect from the mesh.
  pub fn stop(&mut self) -> Result<Vec<RideEvent>> {
    if !self.running {
      return Ok(Vec::new());
    }
    self.broadcast_group_leave()?;
    self.conduit.tick()?;
    self.conduit.leave_network()?;
    self.running = false;
    self.members
      .retain(|id, _| *id == self.conduit.node_id());
    Ok(vec![RideEvent::SessionStopped])
  }

  /// Update the rider's GPS fix for periodic sharing.
  pub fn set_position(&mut self, position: RidePosition) {
    self.position = Some(position);
    if let Some(member) = self.members.get_mut(&self.conduit.node_id()) {
      member.last_location = Some(position.to_location_payload());
    }
  }

  /// Press or release the push-to-talk button.
  pub fn set_push_to_talk(&mut self, active: bool) {
    self.push_to_talk_active = active;
    self.conduit.set_push_to_talk(active);
  }

  /// Send captured microphone samples when push-to-talk is active.
  pub fn send_audio(&mut self, samples: &[i16]) -> Result<()> {
    if self.config.push_to_talk && !self.push_to_talk_active {
      return Ok(());
    }
    self.conduit.send_voice(samples)?;
    Ok(())
  }

  /// Immediately broadcast the current GPS position.
  pub fn share_location(&mut self) -> Result<()> {
    let position = self
      .position
      .ok_or_else(|| conduit_sdk::ConduitError::Configuration("no position set".into()))?;
    self.conduit.send_location(
      position.latitude_microdeg,
      position.longitude_microdeg,
      position.altitude_m,
      position.accuracy_m,
    )?;
    self.last_location_share = Some(Instant::now());
    Ok(())
  }

  /// Broadcast a distress signal to the group.
  pub fn trigger_sos(&mut self, message: impl Into<String>) -> Result<()> {
    self.conduit.send_emergency(EmergencyPayload {
      kind: EmergencyKind::General,
      message: message.into(),
    })
  }

  /// Send a plain-text message to the group (broadcast).
  pub fn send_text(&mut self, content: impl Into<String>) -> Result<()> {
    self.conduit.send_broadcast(content)
  }

  /// Advance the session: poll mesh I/O, share GPS on interval, decode playback.
  pub fn tick(&mut self) -> Result<Vec<RideEvent>> {
    if !self.running {
      return Ok(Vec::new());
    }

    self.maybe_share_location()?;

    let sdk_events = self.conduit.tick()?;
    let mut events = Vec::new();
    for event in sdk_events {
      events.extend(self.map_sdk_event(event));
    }
    Ok(events)
  }

  /// Number of PCM samples per voice frame.
  pub fn frame_samples(&self) -> usize {
    self.conduit.voice().config().frame_samples()
  }

  /// Decode the next voice frame for speaker output.
  pub fn playback_audio(&mut self) -> Result<Option<Vec<i16>>> {
    self.conduit.voice_mut().playback()
  }

  /// Connect a simulated peer (simulation mode only).
  pub fn connect_peer(&mut self, peer: DiscoveredPeer) -> Result<()> {
    self.conduit.connect_peer(peer)
  }

  /// Link two simulated riders for local testing.
  pub fn link_riders(a: &mut RideIntercom, b: &mut RideIntercom) -> Result<()> {
    Conduit::link_simulated_peers(&mut a.conduit, &mut b.conduit)
  }

  /// Shared simulation bus when running in simulation mode.
  pub fn sim_bus(&self) -> Option<&SimBusHandle> {
    self.conduit.sim_bus()
  }

  fn broadcast_group_join(&mut self) -> Result<()> {
    let msg = group::join_message(&self.config.group_name, &self.config.rider_name);
    self.conduit.send_broadcast(msg)
  }

  fn broadcast_group_leave(&mut self) -> Result<()> {
    let msg = group::leave_message(&self.config.group_name, &self.config.rider_name);
    self.conduit.send_broadcast(msg)
  }

  fn maybe_share_location(&mut self) -> Result<()> {
    if self.position.is_none() || self.config.location_share_interval_ms == 0 {
      return Ok(());
    }
    let due = match self.last_location_share {
      None => true,
      Some(last) => {
        last.elapsed().as_millis() as u64 >= self.config.location_share_interval_ms
      }
    };
    if due {
      self.share_location()?;
    }
    Ok(())
  }

  fn map_sdk_event(&mut self, event: SdkEvent) -> Vec<RideEvent> {
    match event {
      SdkEvent::PeerDiscovered { node_id, name } => {
        vec![RideEvent::MemberJoined(RideMember {
          node_id,
          name,
          last_location: None,
        })]
      }
      SdkEvent::PeerLost { node_id } => {
        let name = self
          .members
          .remove(&node_id)
          .map(|m| m.name)
          .unwrap_or_else(|| format!("node-{node_id}"));
        vec![RideEvent::MemberLeft { node_id, name }]
      }
      SdkEvent::NeighborAdded(neighbor) => {
        if self.members.contains_key(&neighbor.node_id) {
          return Vec::new();
        }
        let member = RideMember {
          node_id: neighbor.node_id,
          name: neighbor.node_name,
          last_location: None,
        };
        self.members.insert(neighbor.node_id, member.clone());
        vec![RideEvent::MemberJoined(member)]
      }
      SdkEvent::NeighborRemoved { node_id } => {
        if node_id == self.conduit.node_id() {
          return Vec::new();
        }
        let name = self
          .members
          .remove(&node_id)
          .map(|m| m.name)
          .unwrap_or_else(|| format!("node-{node_id}"));
        vec![RideEvent::MemberLeft { node_id, name }]
      }
      SdkEvent::LocationReceived { from, location } => {
        if let Some(member) = self.members.get_mut(&from) {
          member.last_location = Some(location.clone());
        } else {
          self.members.insert(
            from,
            RideMember {
              node_id: from,
              name: format!("node-{from}"),
              last_location: Some(location.clone()),
            },
          );
        }
        vec![RideEvent::MemberLocationUpdated { node_id: from, location }]
      }
      SdkEvent::EmergencyReceived { from, emergency } => {
        vec![RideEvent::SosReceived { from, emergency }]
      }
      SdkEvent::VoiceFrameReceived { from } => {
        vec![RideEvent::VoiceReceived { from }]
      }
      SdkEvent::MessageReceived { from, message } => self.handle_message(from, message),
      SdkEvent::NetworkJoined | SdkEvent::NetworkLeft => Vec::new(),
    }
  }

  fn handle_message(&mut self, from: NodeId, message: MessagingPayload) -> Vec<RideEvent> {
    if let Some(group_msg) = group::parse(&message.content) {
      return self.handle_group_message(from, group_msg);
    }
  vec![RideEvent::TextMessage {
      from,
      content: message.content,
    }]
  }

  fn handle_group_message(&mut self, from: NodeId, message: GroupMessage) -> Vec<RideEvent> {
    match message {
      GroupMessage::Join { group, rider } if group == self.config.group_name => {
        if self.members.contains_key(&from) {
          return Vec::new();
        }
        let member = RideMember {
          node_id: from,
          name: rider,
          last_location: None,
        };
        self.members.insert(from, member.clone());
        vec![RideEvent::MemberJoined(member)]
      }
      GroupMessage::Leave { group, rider } if group == self.config.group_name => {
        self.members.remove(&from);
        vec![RideEvent::MemberLeft { node_id: from, name: rider }]
      }
      _ => Vec::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_sdk::{DriverKind, PeerEndpoint, SimBusHandle};

  fn rider(name: &str, seed: u8, group: &str, bus: &SimBusHandle) -> RideIntercom {
    let mut id_bytes = [0u8; 16];
    id_bytes[0] = seed;
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    let sdk = conduit_sdk::SdkConfig::builder()
      .node_id(NodeId::from_bytes(id_bytes))
      .node_name(name)
      .simulation(true)
      .sim_bus(bus.clone())
      .identity_seed(seed_bytes)
      .build();
    let config = RideIntercomConfig::builder()
      .sdk(sdk)
      .rider_name(name)
      .group_name(group)
      .push_to_talk(false)
      .location_share_interval_ms(0)
      .build();
    RideIntercom::new(config).unwrap()
  }

  fn peer_for(other: &RideIntercom, sim_id: u64) -> DiscoveredPeer {
    DiscoveredPeer::new(
      other.node_id(),
      other.rider_name().into(),
      other.config().sdk.capabilities,
      DriverKind::Mock,
      PeerEndpoint::Simulated { id: sim_id },
    )
  }

  fn link_pair(a: &mut RideIntercom, b: &mut RideIntercom) {
    a.connect_peer(peer_for(b, b.node_id().as_bytes()[0] as u64))
      .unwrap();
    b.connect_peer(peer_for(a, a.node_id().as_bytes()[0] as u64))
      .unwrap();
  }

  #[test]
  fn session_start_and_group_join() {
    let bus = SimBusHandle::new();
    let mut ride = rider("alex", 1, "sunday-ride", &bus);
    let events = ride.start().unwrap();
    assert!(ride.is_running());
    assert!(events
      .iter()
      .any(|e| matches!(e, RideEvent::SessionStarted { group } if group == "sunday-ride")));
  }

  #[test]
  fn sos_reaches_group_member() {
    let bus = SimBusHandle::new();
    let mut a = rider("alex", 2, "pack-ride", &bus);
    let mut b = rider("blake", 3, "pack-ride", &bus);
    a.start().unwrap();
    b.start().unwrap();
    link_pair(&mut a, &mut b);

    a.trigger_sos("bike down").unwrap();
    a.tick().unwrap();
    let events = b.tick().unwrap();

    assert!(events.iter().any(|e| matches!(
      e,
      RideEvent::SosReceived { emergency, .. } if emergency.message == "bike down"
    )));
  }

  #[test]
  fn location_sharing_between_riders() {
    let bus = SimBusHandle::new();
    let mut a = rider("alex", 4, "gps-ride", &bus);
    let mut b = rider("blake", 5, "gps-ride", &bus);
    a.start().unwrap();
    b.start().unwrap();
    link_pair(&mut a, &mut b);

    a.set_position(RidePosition::new(51_507_400, -0_127_800));
    a.share_location().unwrap();
    a.tick().unwrap();
    let events = b.tick().unwrap();

    assert!(events.iter().any(|e| matches!(
      e,
      RideEvent::MemberLocationUpdated { location, .. }
        if location.latitude_microdeg == 51_507_400
    )));
  }

  #[test]
  fn voice_round_trip_through_app() {
    let bus = SimBusHandle::new();
    let mut a = rider("talker", 6, "voice-ride", &bus);
    let mut b = rider("listener", 7, "voice-ride", &bus);
    a.start().unwrap();
    b.start().unwrap();
    link_pair(&mut a, &mut b);

    let samples = vec![800i16; a.frame_samples()];
    for _ in 0..3 {
      a.send_audio(&samples).unwrap();
      a.tick().unwrap();
      b.tick().unwrap();
    }

    assert!(b.playback_audio().unwrap().is_some());
  }

  #[test]
  fn group_join_message_adds_member() {
    let bus = SimBusHandle::new();
    let mut a = rider("alex", 8, "group-a", &bus);
    let mut b = rider("blake", 9, "group-a", &bus);
    a.start().unwrap();
    b.start().unwrap();
    link_pair(&mut a, &mut b);

    a.send_text(crate::group::join_message("group-a", "alex"))
      .unwrap();
    a.tick().unwrap();
    let events = b.tick().unwrap();

    assert!(events.iter().any(|e| matches!(
      e,
      RideEvent::MemberJoined(m) if m.name == "alex"
    )));
    assert!(b.members().any(|m| m.name == "alex"));
  }
}
