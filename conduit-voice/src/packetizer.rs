use conduit_core::ids::NodeId;
use conduit_packets::{PacketBody, PacketType, TypedPacket, VoicePayload};
use conduit_core::error::Result;

/// Builds voice [`TypedPacket`]s from encoded audio frames.
#[derive(Debug)]
pub struct VoicePacketizer {
  local_node_id: NodeId,
  frame_duration_ms: u16,
  next_frame_index: u16,
}

impl VoicePacketizer {
  pub fn new(local_node_id: NodeId, frame_duration_ms: u16) -> Self {
    Self {
      local_node_id,
      frame_duration_ms,
      next_frame_index: 0,
    }
  }

  pub fn packetize(
    &mut self,
    destination: NodeId,
    opus_data: Vec<u8>,
  ) -> Result<TypedPacket> {
    let frame_index = self.next_frame_index;
    self.next_frame_index = self.next_frame_index.wrapping_add(1);

    TypedPacket::with_body(
      PacketType::Voice,
      self.local_node_id,
      destination,
      PacketBody::Voice(VoicePayload {
        frame_index,
        duration_ms: self.frame_duration_ms,
        opus_data,
      }),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_mesh::BROADCAST_NODE_ID;

  #[test]
  fn increments_frame_index() {
    let mut p = VoicePacketizer::new(NodeId::random(), 20);
    let a = p.packetize(BROADCAST_NODE_ID, vec![1]).unwrap();
    let b = p.packetize(BROADCAST_NODE_ID, vec![2]).unwrap();
    if let PacketBody::Voice(va) = a.body {
      if let PacketBody::Voice(vb) = b.body {
        assert_eq!(vb.frame_index, va.frame_index.wrapping_add(1));
      }
    }
  }
}
