use crate::body::PacketBody;
use crate::codec::PacketCodec;
use crate::typed::TypedPacket;
use conduit_core::{NodeId, PacketFlags, PacketHeader, PacketPriority, PacketSequence, PacketType};

/// Fluent builder for constructing typed packets.
#[derive(Debug, Clone)]
pub struct PacketBuilder {
  packet_type: PacketType,
  source: NodeId,
  destination: NodeId,
  priority: PacketPriority,
  flags: PacketFlags,
  ttl: u8,
  sequence: PacketSequence,
  body: Option<PacketBody>,
}

impl PacketBuilder {
  pub fn new(packet_type: PacketType, source: NodeId, destination: NodeId) -> Self {
    Self {
      packet_type,
      source,
      destination,
      priority: PacketPriority::Normal,
      flags: PacketFlags::NONE,
      ttl: conduit_core::constants::DEFAULT_TTL,
      sequence: PacketSequence::ZERO,
      body: None,
    }
  }

  pub fn priority(mut self, priority: PacketPriority) -> Self {
    self.priority = priority;
    self
  }

  pub fn flags(mut self, flags: PacketFlags) -> Self {
    self.flags = flags;
    self
  }

  pub fn ttl(mut self, ttl: u8) -> Self {
    self.ttl = ttl;
    self
  }

  pub fn sequence(mut self, sequence: PacketSequence) -> Self {
    self.sequence = sequence;
    self
  }

  pub fn body(mut self, body: PacketBody) -> Self {
    self.body = Some(body);
    self
  }

  pub fn build(self) -> crate::Result<TypedPacket> {
    let body = self
      .body
      .ok_or_else(|| conduit_core::ConduitError::InvalidPacket("missing packet body".into()))?;
    let mut header = PacketHeader::new(self.packet_type, self.source, self.destination);
    header.priority = self.priority;
    header.flags = self.flags;
    header.ttl = self.ttl;
    header.sequence = self.sequence;
    TypedPacket::new(header, body)
  }

  pub fn build_wire(self) -> crate::Result<conduit_core::Packet> {
    PacketCodec::encode(&self.build()?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::payloads::HeartbeatPayload;

  #[test]
  fn builder_produces_valid_packet() {
    let packet = PacketBuilder::new(PacketType::Heartbeat, NodeId::random(), NodeId::random())
      .priority(PacketPriority::Low)
      .body(PacketBody::Heartbeat(HeartbeatPayload {
        uptime_ms: 1_000,
        neighbor_count: 2,
      }))
      .build()
      .unwrap();

    assert_eq!(packet.packet_type(), PacketType::Heartbeat);
  }
}
