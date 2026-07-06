use crate::compression::Compression;
use crate::config::TransportConfig;
use crate::encryption::{Encryption, KEY_SIZE};
use crate::fragment::Fragmenter;
use crate::frame::{TransportFlags, TransportFrame};
use crate::reassembly::Reassembler;
use conduit_core::error::{ConduitError, Result};
use conduit_core::Packet;
use conduit_packets::{PacketCodec, TypedPacket};

/// Main transport pipeline: packet objects ↔ on-wire byte streams.
pub struct TransportEngine {
  config: TransportConfig,
  fragmenter: Fragmenter,
  reassembler: Reassembler,
}

impl TransportEngine {
  pub fn new(config: TransportConfig) -> Result<Self> {
    config.validate()?;
    let fragmenter = Fragmenter::new(config.clone());
    Ok(Self {
      config,
      fragmenter,
      reassembler: Reassembler::new(),
    })
  }

  pub fn config(&self) -> &TransportConfig {
    &self.config
  }

  /// Serialize a packet into one or more transport frames.
  pub fn encode_packet(&self, packet: &Packet) -> Result<Vec<Vec<u8>>> {
    let packet_bytes = packet.to_bytes();
    let (inner, flags) = self.process_outbound(&packet_bytes)?;
    let fragment_id = Fragmenter::next_fragment_id();
    let frames = self.fragmenter.fragment(&inner, flags, fragment_id)?;
    Ok(frames.into_iter().map(|f| f.encode()).collect())
  }

  /// Serialize a typed packet into transport frames.
  pub fn encode_typed(&self, packet: &TypedPacket) -> Result<Vec<Vec<u8>>> {
    self.encode_packet(&PacketCodec::encode(packet)?)
  }

  /// Parse a transport frame. Returns a complete packet when reassembly finishes.
  pub fn decode_frame(&mut self, frame_bytes: &[u8]) -> Result<Option<Packet>> {
    let frame = TransportFrame::decode(frame_bytes)?;
    let reassembled = match self.reassembler.ingest(&frame)? {
      Some(blob) => blob,
      None => return Ok(None),
    };
    let packet_bytes = self.process_inbound(&reassembled.data, reassembled.flags)?;
    Ok(Some(Packet::from_bytes(&packet_bytes)?))
  }

  /// Parse a transport frame into a typed packet when complete.
  pub fn decode_typed_frame(&mut self, frame_bytes: &[u8]) -> Result<Option<TypedPacket>> {
    match self.decode_frame(frame_bytes)? {
      Some(packet) => Ok(Some(PacketCodec::decode(&packet)?)),
      None => Ok(None),
    }
  }

  fn process_outbound(&self, packet_bytes: &[u8]) -> Result<(Vec<u8>, TransportFlags)> {
    let mut data = packet_bytes.to_vec();
    let mut flags = TransportFlags::NONE;

    if self.config.enable_compression {
      if let Some(compressed) =
        Compression::compress_if_beneficial(&data, self.config.compression_threshold)?
      {
        data = compressed;
        flags = flags.union(TransportFlags::COMPRESSED);
      }
    }

    if self.config.enable_encryption {
      let key = self
        .config
        .encryption_key
        .as_ref()
        .expect("validated at construction");
      data = Encryption::encrypt(key, &data)?;
      flags = flags.union(TransportFlags::ENCRYPTED);
    }

    Ok((data, flags))
  }

  fn process_inbound(&self, inner: &[u8], flags: TransportFlags) -> Result<Vec<u8>> {
    let mut data = inner.to_vec();

    if flags.contains(TransportFlags::ENCRYPTED) {
      let key = self
        .config
        .encryption_key
        .as_ref()
        .ok_or_else(|| ConduitError::Deserialization("encrypted frame but no key".into()))?;
      data = Encryption::decrypt(key, &data)?;
    }

    if flags.contains(TransportFlags::COMPRESSED) {
      data = Compression::decompress(&data)?;
    }

    Ok(data)
  }
}

impl Default for TransportEngine {
  fn default() -> Self {
    Self::new(TransportConfig::default()).expect("default transport config is valid")
  }
}

/// Generate a random 32-byte encryption key.
pub fn generate_key() -> [u8; KEY_SIZE] {
  let mut key = [0u8; KEY_SIZE];
  rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key);
  key
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_core::NodeId;
  use conduit_packets::{PacketBody, PacketType, MessagingPayload};

  fn sample_packet() -> Packet {
    let typed = TypedPacket::with_body(
      PacketType::Heartbeat,
      NodeId::random(),
      NodeId::random(),
      PacketBody::Heartbeat(conduit_packets::HeartbeatPayload {
        uptime_ms: 500,
        neighbor_count: 1,
      }),
    )
    .unwrap();
    PacketCodec::encode(&typed).unwrap()
  }

  fn large_packet() -> Packet {
    let typed = TypedPacket::with_body(
      PacketType::Messaging,
      NodeId::random(),
      NodeId::random(),
      PacketBody::Messaging(MessagingPayload {
        message_id: 1,
        content: "x".repeat(2_000),
      }),
    )
    .unwrap();
    PacketCodec::encode(&typed).unwrap()
  }

  #[test]
  fn packet_round_trip() {
    let engine = TransportEngine::default();
    let packet = sample_packet();
    let frames = engine.encode_packet(&packet).unwrap();
    assert_eq!(frames.len(), 1);

    let mut decoder = TransportEngine::default();
    let restored = decoder.decode_frame(&frames[0]).unwrap().unwrap();
    assert_eq!(restored.header.packet_type, packet.header.packet_type);
    assert_eq!(restored.payload, packet.payload);
  }

  #[test]
  fn typed_round_trip() {
    let engine = TransportEngine::default();
    let typed = TypedPacket::with_body(
      PacketType::Messaging,
      NodeId::random(),
      NodeId::random(),
      PacketBody::Messaging(MessagingPayload {
        message_id: 42,
        content: "hello transport".into(),
      }),
    )
    .unwrap();

    let frames = engine.encode_typed(&typed).unwrap();
    let mut decoder = TransportEngine::default();
    let restored = decoder.decode_typed_frame(&frames[0]).unwrap().unwrap();
    assert_eq!(restored.body, typed.body);
  }

  #[test]
  fn fragmented_round_trip() {
    let config = TransportConfig::builder().mtu(200).enable_compression(false).build();
    let engine = TransportEngine::new(config).unwrap();
    let packet = large_packet();
    let frames = engine.encode_packet(&packet).unwrap();
    assert!(frames.len() > 1);

    let mut decoder = TransportEngine::new(TransportConfig::builder().mtu(200).enable_compression(false).build()).unwrap();
    let mut restored = None;
    for frame in frames {
      if let Some(pkt) = decoder.decode_frame(&frame).unwrap() {
        restored = Some(pkt);
      }
    }
    let restored = restored.unwrap();
    assert_eq!(restored.payload, packet.payload);
  }

  #[test]
  fn encrypted_round_trip() {
    let key = [0xAA; KEY_SIZE];
    let config = TransportConfig::builder()
      .enable_encryption(key)
      .enable_compression(false)
      .build();
    let engine = TransportEngine::new(config.clone()).unwrap();
    let packet = sample_packet();
    let frames = engine.encode_packet(&packet).unwrap();
    assert!(TransportFrame::decode(&frames[0]).unwrap().flags.contains(TransportFlags::ENCRYPTED));

    let mut decoder = TransportEngine::new(config).unwrap();
    let restored = decoder.decode_frame(&frames[0]).unwrap().unwrap();
    assert_eq!(restored.payload, packet.payload);
  }

  #[test]
  fn compression_round_trip() {
    let config = TransportConfig::builder()
      .compression_threshold(16)
      .enable_compression(true)
      .build();
    let engine = TransportEngine::new(config.clone()).unwrap();
    let packet = large_packet();
    let frames = engine.encode_packet(&packet).unwrap();

    let mut decoder = TransportEngine::new(config).unwrap();
    let mut restored = None;
    for frame in frames {
      if let Some(pkt) = decoder.decode_frame(&frame).unwrap() {
        restored = Some(pkt);
      }
    }
    assert_eq!(restored.unwrap().payload, packet.payload);
  }
}
