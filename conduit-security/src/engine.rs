use crate::cipher::Cipher;
use crate::config::SecurityConfig;
use crate::envelope::SecureEnvelope;
use crate::identity::{Identity, IdentityProof, PeerIdentity};
use crate::replay::ReplayGuard;
use crate::session::{SessionKey, SessionStore};
use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use conduit_core::utils::unix_timestamp_ms;
use conduit_core::Packet;
use ed25519_dalek::VerifyingKey;
use std::collections::HashMap;

/// Application-facing security layer for protecting Conduit packets.
pub struct SecurityEngine {
  config: SecurityConfig,
  identity: Identity,
  sessions: SessionStore,
  replay: ReplayGuard,
  known_peers: HashMap<NodeId, PeerIdentity>,
  outbound_sequence: u64,
}

impl SecurityEngine {
  pub fn new(config: SecurityConfig, identity: Identity) -> Result<Self> {
    config.validate()?;
    if identity.node_id() != config.local_node_id {
      return Err(ConduitError::Configuration(
        "identity node id does not match security config".into(),
      ));
    }
    Ok(Self {
      replay: ReplayGuard::new(config.replay_window),
      sessions: SessionStore::new(),
      known_peers: HashMap::new(),
      outbound_sequence: 0,
      config,
      identity,
    })
  }

  pub fn identity(&self) -> &Identity {
    &self.identity
  }

  pub fn local_proof(&self) -> IdentityProof {
    self.identity.create_proof()
  }

  pub fn register_peer(&mut self, proof: IdentityProof) -> Result<()> {
    Identity::verify_peer(&proof)?;
    if proof.peer.node_id == self.config.local_node_id {
      return Err(ConduitError::Configuration(
        "cannot register local node as peer".into(),
      ));
    }
    self.known_peers.insert(proof.peer.node_id, proof.peer.clone());
    self.establish_session(proof.peer.node_id, &proof.peer.dh_public_key);
    Ok(())
  }

  pub fn establish_session(&mut self, peer: NodeId, peer_dh_public: &[u8; 32]) {
    let shared = self.identity.diffie_hellman(peer_dh_public);
    let key = SessionKey::derive(&shared, self.config.local_node_id, peer);
    self.sessions.insert(peer, key);
  }

  pub fn has_session(&self, peer: &NodeId) -> bool {
    self.sessions.contains(peer)
  }

  /// Encrypt, authenticate, and wrap a packet for a specific peer.
  pub fn protect(&mut self, packet: &Packet, peer: NodeId) -> Result<Vec<u8>> {
    let session = self
      .sessions
      .get(&peer)
      .ok_or_else(|| ConduitError::Configuration(format!("no session for peer {peer}")))?;

    self.outbound_sequence += 1;
    let sequence = self.outbound_sequence;
    let timestamp_ms = unix_timestamp_ms();
    let plaintext = packet.to_bytes();
    let ciphertext = Cipher::encrypt(&session, &plaintext)?;

    let mut envelope = SecureEnvelope {
      sender: self.config.local_node_id,
      sequence,
      timestamp_ms,
      ciphertext,
      signature: [0u8; 64],
    };
    envelope.signature = self.identity.sign(&envelope.signing_bytes());

    Ok(envelope.to_bytes())
  }

  /// Verify, decrypt, and parse a secure envelope into a packet.
  pub fn open(&mut self, data: &[u8]) -> Result<Packet> {
    let envelope = SecureEnvelope::from_bytes(data)?;
    self.validate_timestamp(envelope.timestamp_ms)?;

    if envelope.sender == self.config.local_node_id {
      return Err(ConduitError::InvalidPacket(
        "refusing packet from self".into(),
      ));
    }

    let peer = self
      .known_peers
      .get(&envelope.sender)
      .ok_or_else(|| ConduitError::Deserialization("unknown sender".into()))?;

    if self.config.require_known_peers && !self.sessions.contains(&envelope.sender) {
      return Err(ConduitError::Deserialization(
        "no session for known peer".into(),
      ));
    }

    let verifying_key = VerifyingKey::from_bytes(&peer.signing_public_key)
      .map_err(|e| ConduitError::Deserialization(e.to_string()))?;
    envelope.verify(&verifying_key)?;

    self
      .replay
      .check_and_record(envelope.sender, envelope.sequence)?;

    let session = self
      .sessions
      .get(&envelope.sender)
      .ok_or_else(|| ConduitError::Deserialization("no session for sender".into()))?;

    let plaintext = envelope.decrypt(&session)?;
    Packet::from_bytes(&plaintext)
  }

  pub fn on_peer_lost(&mut self, peer: &NodeId) {
    self.sessions.remove(peer);
    self.known_peers.remove(peer);
    self.replay.reset_peer(peer);
  }

  fn validate_timestamp(&self, timestamp_ms: u64) -> Result<()> {
    let now = unix_timestamp_ms();
    let skew = now.abs_diff(timestamp_ms);
    if skew > self.config.max_clock_skew_ms {
      return Err(ConduitError::InvalidPacket(format!(
        "timestamp skew too large: {skew}ms"
      )));
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_core::PacketType;

  fn pair() -> (SecurityEngine, SecurityEngine) {
    let a_id = NodeId::random();
    let b_id = NodeId::random();
    let a_identity = Identity::generate(a_id);
    let b_identity = Identity::generate(b_id);

    let mut a = SecurityEngine::new(
      SecurityConfig::builder().local_node_id(a_id).build(),
      a_identity,
    )
    .unwrap();
    let mut b = SecurityEngine::new(
      SecurityConfig::builder().local_node_id(b_id).build(),
      b_identity,
    )
    .unwrap();

    a.register_peer(b.local_proof()).unwrap();
    b.register_peer(a.local_proof()).unwrap();
    (a, b)
  }

  #[test]
  fn protect_and_open_round_trip() {
    let (mut a, mut b) = pair();
    let packet = Packet::with_payload(
      PacketType::Messaging,
      a.config.local_node_id,
      b.config.local_node_id,
      b"secret".to_vec(),
    );

    let secured = a.protect(&packet, b.config.local_node_id).unwrap();
    let opened = b.open(&secured).unwrap();
    assert_eq!(opened.payload, packet.payload);
  }

  #[test]
  fn rejects_replay() {
    let (mut a, mut b) = pair();
    let packet = Packet::with_payload(
      PacketType::Heartbeat,
      a.config.local_node_id,
      b.config.local_node_id,
      vec![],
    );
    let secured = a.protect(&packet, b.config.local_node_id).unwrap();
    b.open(&secured).unwrap();
    assert!(b.open(&secured).is_err());
  }

  #[test]
  fn rejects_unknown_sender() {
    let a_id = NodeId::random();
    let b_id = NodeId::random();
    let mut a = SecurityEngine::new(
      SecurityConfig::builder().local_node_id(a_id).build(),
      Identity::generate(a_id),
    )
    .unwrap();
    let _b = SecurityEngine::new(
      SecurityConfig::builder().local_node_id(b_id).build(),
      Identity::generate(b_id),
    )
    .unwrap();

    let packet = Packet::with_payload(PacketType::Control, a_id, b_id, vec![]);
    let secured = a.protect(&packet, b_id);
    assert!(secured.is_err());
  }

  #[test]
  fn identity_registration_establishes_session() {
    let (a, b) = pair();
    assert!(a.has_session(&b.config.local_node_id));
    assert!(b.has_session(&a.config.local_node_id));
  }
}
