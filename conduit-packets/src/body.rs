use crate::payload::PacketPayload;
use crate::payloads::{
  ControlPayload, DiscoveryPayload, EmergencyPayload, HeartbeatPayload, LocationPayload,
  MessagingPayload, RoutingPayload, TelemetryPayload, VoicePayload,
};
use conduit_core::error::{ConduitError, Result};
use conduit_core::serialization::{ByteReader, ByteWriter};
use conduit_core::PacketType;

/// Typed payload variants for every supported packet type.
///
/// Add a new variant here when introducing a packet type — routing is unaffected
/// because it only reads the header.
#[derive(Debug, Clone, PartialEq)]
pub enum PacketBody {
  Voice(VoicePayload),
  Location(LocationPayload),
  Heartbeat(HeartbeatPayload),
  Emergency(EmergencyPayload),
  Messaging(MessagingPayload),
  Discovery(DiscoveryPayload),
  Routing(RoutingPayload),
  Telemetry(TelemetryPayload),
  Control(ControlPayload),
}

impl PacketBody {
  pub fn packet_type(&self) -> PacketType {
    match self {
      Self::Voice(_) => PacketType::Voice,
      Self::Location(_) => PacketType::Location,
      Self::Heartbeat(_) => PacketType::Heartbeat,
      Self::Emergency(_) => PacketType::Emergency,
      Self::Messaging(_) => PacketType::Messaging,
      Self::Discovery(_) => PacketType::Discovery,
      Self::Routing(_) => PacketType::Routing,
      Self::Telemetry(_) => PacketType::Telemetry,
      Self::Control(_) => PacketType::Control,
    }
  }

  pub fn encode(&self, writer: &mut ByteWriter) {
    match self {
      Self::Voice(p) => p.encode(writer),
      Self::Location(p) => p.encode(writer),
      Self::Heartbeat(p) => p.encode(writer),
      Self::Emergency(p) => p.encode(writer),
      Self::Messaging(p) => p.encode(writer),
      Self::Discovery(p) => p.encode(writer),
      Self::Routing(p) => p.encode(writer),
      Self::Telemetry(p) => p.encode(writer),
      Self::Control(p) => p.encode(writer),
    }
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    let mut writer = ByteWriter::new();
    self.encode(&mut writer);
    writer.into_vec()
  }

  pub fn decode(packet_type: PacketType, reader: &mut ByteReader<'_>) -> Result<Self> {
    let body = match packet_type {
      PacketType::Voice => Self::Voice(VoicePayload::decode(reader)?),
      PacketType::Location => Self::Location(LocationPayload::decode(reader)?),
      PacketType::Heartbeat => Self::Heartbeat(HeartbeatPayload::decode(reader)?),
      PacketType::Emergency => Self::Emergency(EmergencyPayload::decode(reader)?),
      PacketType::Messaging => Self::Messaging(MessagingPayload::decode(reader)?),
      PacketType::Discovery => Self::Discovery(DiscoveryPayload::decode(reader)?),
      PacketType::Routing => Self::Routing(RoutingPayload::decode(reader)?),
      PacketType::Telemetry => Self::Telemetry(TelemetryPayload::decode(reader)?),
      PacketType::Control => Self::Control(ControlPayload::decode(reader)?),
    };
    Ok(body)
  }

  pub fn from_bytes(packet_type: PacketType, data: &[u8]) -> Result<Self> {
    let mut reader = ByteReader::new(data);
    Self::decode(packet_type, &mut reader)
  }

  pub fn matches_type(&self, packet_type: PacketType) -> bool {
    self.packet_type() == packet_type
  }

  pub fn ensure_matches(packet_type: PacketType, body: &Self) -> Result<()> {
    if !body.matches_type(packet_type) {
      return Err(ConduitError::InvalidPacket(format!(
        "header type {:?} does not match body type {:?}",
        packet_type,
        body.packet_type()
      )));
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_core::NodeId;

  #[test]
  fn all_variants_round_trip() {
    let bodies = vec![
      PacketBody::Voice(VoicePayload {
        frame_index: 1,
        duration_ms: 20,
        opus_data: vec![0xDE, 0xAD],
      }),
      PacketBody::Location(LocationPayload {
        latitude_microdeg: 28_613_900,
        longitude_microdeg: 77_209_000,
        altitude_m: 200,
        accuracy_m: 5,
      }),
      PacketBody::Heartbeat(HeartbeatPayload {
        uptime_ms: 60_000,
        neighbor_count: 3,
      }),
      PacketBody::Emergency(EmergencyPayload {
        kind: crate::payloads::EmergencyKind::Medical,
        message: "need help".into(),
      }),
      PacketBody::Messaging(MessagingPayload {
        message_id: 99,
        content: "hello".into(),
      }),
      PacketBody::Discovery(DiscoveryPayload {
        node_name: "rider-1".into(),
        capabilities: 0b101,
      }),
      PacketBody::Routing(RoutingPayload {
        hop_count: 2,
        path: vec![NodeId::random(), NodeId::random()],
      }),
      PacketBody::Telemetry(TelemetryPayload {
        metric: "battery".into(),
        value: 0.87,
      }),
      PacketBody::Control(ControlPayload {
        command: crate::payloads::ControlCommand::Join,
        argument: 1,
      }),
    ];

    for body in bodies {
      let packet_type = body.packet_type();
      let bytes = body.to_bytes();
      let restored = PacketBody::from_bytes(packet_type, &bytes).unwrap();
      assert_eq!(restored, body);
    }
  }
}
