use crate::body::PacketBody;
use conduit_core::error::{ConduitError, Result};
use conduit_core::{PacketHeader, PacketType};

/// A fully typed Conduit packet: shared header plus a typed payload body.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedPacket {
  pub header: PacketHeader,
  pub body: PacketBody,
}

impl TypedPacket {
  pub fn new(header: PacketHeader, body: PacketBody) -> Result<Self> {
    PacketBody::ensure_matches(header.packet_type, &body)?;
    let mut packet = Self { header, body };
    packet.sync_header();
    Ok(packet)
  }

  pub fn with_body(
    packet_type: PacketType,
    source: conduit_core::NodeId,
    destination: conduit_core::NodeId,
    body: PacketBody,
  ) -> Result<Self> {
    if body.packet_type() != packet_type {
      return Err(ConduitError::InvalidPacket(format!(
        "expected {:?}, got {:?}",
        packet_type,
        body.packet_type()
      )));
    }
    let header = PacketHeader::new(packet_type, source, destination);
    Self::new(header, body)
  }

  pub fn packet_type(&self) -> PacketType {
    self.header.packet_type
  }

  pub fn into_parts(self) -> (PacketHeader, PacketBody) {
    (self.header, self.body)
  }

  /// Keep header metadata aligned with the typed body.
  fn sync_header(&mut self) {
    self.header.packet_type = self.body.packet_type();
    self.header.payload_length = self.body.to_bytes().len() as u32;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::payloads::HeartbeatPayload;
  use conduit_core::NodeId;

  #[test]
  fn rejects_mismatched_header_and_body() {
    let header = PacketHeader::new(PacketType::Voice, NodeId::random(), NodeId::random());
    let body = PacketBody::Heartbeat(HeartbeatPayload {
      uptime_ms: 0,
      neighbor_count: 0,
    });
    assert!(TypedPacket::new(header, body).is_err());
  }
}
