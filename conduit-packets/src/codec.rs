use crate::body::PacketBody;
use crate::typed::TypedPacket;
use conduit_core::error::{ConduitError, Result};
use conduit_core::serialization::ByteReader;
use conduit_core::Packet;

/// Encodes and decodes between wire-format [`Packet`] and typed [`TypedPacket`].
pub struct PacketCodec;

impl PacketCodec {
  /// Convert a typed packet into a wire-ready opaque packet.
  pub fn encode(packet: &TypedPacket) -> Result<Packet> {
    PacketBody::ensure_matches(packet.header.packet_type, &packet.body)?;
    let payload = packet.body.to_bytes();
    let mut header = packet.header.clone();
    header.packet_type = packet.body.packet_type();
    header.payload_length = payload.len() as u32;
    Ok(Packet::new(header, payload))
  }

  /// Parse a wire packet into its typed form.
  pub fn decode(packet: &Packet) -> Result<TypedPacket> {
    let mut reader = ByteReader::new(&packet.payload);
    let body = PacketBody::decode(packet.header.packet_type, &mut reader)?;
    if reader.remaining() > 0 {
      return Err(ConduitError::InvalidPacket(format!(
        "trailing {} bytes in payload",
        reader.remaining()
      )));
    }
    TypedPacket::new(packet.header.clone(), body)
  }

  /// Serialize a typed packet directly to bytes (header + payload + checksum).
  pub fn to_bytes(packet: &TypedPacket) -> Result<Vec<u8>> {
    Ok(Self::encode(packet)?.to_bytes())
  }

  /// Deserialize bytes into a typed packet.
  pub fn from_bytes(data: &[u8]) -> Result<TypedPacket> {
    let packet = Packet::from_bytes(data)?;
    Self::decode(&packet)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::payloads::{MessagingPayload, VoicePayload};
  use conduit_core::{NodeId, PacketType};

  #[test]
  fn full_wire_round_trip() {
    let source = NodeId::random();
    let dest = NodeId::random();
    let typed = TypedPacket::with_body(
      PacketType::Messaging,
      source,
      dest,
      PacketBody::Messaging(MessagingPayload {
        message_id: 7,
        content: "ride safe".into(),
      }),
    )
    .unwrap();

    let bytes = PacketCodec::to_bytes(&typed).unwrap();
    let restored = PacketCodec::from_bytes(&bytes).unwrap();
    assert_eq!(restored.body, typed.body);
    assert_eq!(restored.header.source, typed.header.source);
    assert_eq!(restored.header.destination, typed.header.destination);
    assert_eq!(restored.packet_type(), typed.packet_type());
    assert!(restored.header.checksum != 0);
  }

  #[test]
  fn opaque_packet_round_trip() {
    let typed = TypedPacket::with_body(
      PacketType::Voice,
      NodeId::random(),
      NodeId::random(),
      PacketBody::Voice(VoicePayload {
        frame_index: 3,
        duration_ms: 20,
        opus_data: vec![1, 2, 3],
      }),
    )
    .unwrap();

    let wire = PacketCodec::encode(&typed).unwrap();
    let restored = PacketCodec::decode(&wire).unwrap();
    assert_eq!(restored.body, typed.body);
    assert_eq!(restored.header.payload_length, typed.header.payload_length);
  }
}
