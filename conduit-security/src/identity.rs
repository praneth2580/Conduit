use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};

/// Local cryptographic identity for a Conduit node.
#[derive(Clone)]
pub struct Identity {
  node_id: NodeId,
  signing_key: SigningKey,
  dh_secret: StaticSecret,
}

/// Public identity material for a remote peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerIdentity {
  pub node_id: NodeId,
  pub signing_public_key: [u8; 32],
  pub dh_public_key: [u8; 32],
}

/// Signed proof tying a node ID to its public keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityProof {
  pub peer: PeerIdentity,
  pub signature: [u8; 64],
}

impl Identity {
  pub fn generate(node_id: NodeId) -> Self {
    Self {
      node_id,
      signing_key: SigningKey::generate(&mut OsRng),
      dh_secret: StaticSecret::random_from_rng(OsRng),
    }
  }

  pub fn from_seed(node_id: NodeId, seed: [u8; 32]) -> Self {
    let signing_key = SigningKey::from_bytes(&seed);
    let dh_secret = StaticSecret::from(seed);
    Self {
      node_id,
      signing_key,
      dh_secret,
    }
  }

  pub fn node_id(&self) -> NodeId {
    self.node_id
  }

  pub fn signing_public_key(&self) -> VerifyingKey {
    self.signing_key.verifying_key()
  }

  pub fn dh_public_key(&self) -> PublicKey {
    PublicKey::from(&self.dh_secret)
  }

  pub fn sign(&self, message: &[u8]) -> [u8; 64] {
    self.signing_key.sign(message).to_bytes()
  }

  pub fn create_proof(&self) -> IdentityProof {
    let peer = PeerIdentity {
      node_id: self.node_id,
      signing_public_key: self.signing_public_key().to_bytes(),
      dh_public_key: *self.dh_public_key().as_bytes(),
    };
    let message = peer.signing_message();
    IdentityProof {
      peer,
      signature: self.sign(&message),
    }
  }

  pub fn verify_peer(proof: &IdentityProof) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(&proof.peer.signing_public_key)
      .map_err(|e| ConduitError::Deserialization(format!("invalid signing key: {e}")))?;
    verifying_key
      .verify(&proof.peer.signing_message(), &proof.signature.into())
      .map_err(|e| ConduitError::Deserialization(format!("identity verification failed: {e}")))?;
    Ok(())
  }

  pub fn diffie_hellman(&self, peer_public: &[u8; 32]) -> [u8; 32] {
    let public = PublicKey::from(*peer_public);
    *self.dh_secret.diffie_hellman(&public).as_bytes()
  }
}

impl PeerIdentity {
  pub fn signing_message(&self) -> Vec<u8> {
    let mut msg = Vec::with_capacity(16 + 32 + 32);
    msg.extend_from_slice(self.node_id.as_bytes());
    msg.extend_from_slice(&self.signing_public_key);
    msg.extend_from_slice(&self.dh_public_key);
    msg
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn identity_proof_round_trip() {
    let id = Identity::generate(NodeId::random());
    let proof = id.create_proof();
    Identity::verify_peer(&proof).unwrap();
  }
}
