use crate::cipher::Cipher;
use crate::config::{ENVELOPE_MAGIC, SECURITY_VERSION};
use crate::session::SessionKey;
use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use conduit_core::serialization::{ByteReader, ByteWriter};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Authenticated, encrypted packet wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureEnvelope {
  pub sender: NodeId,
  pub sequence: u64,
  pub timestamp_ms: u64,
  pub ciphertext: Vec<u8>,
  pub signature: [u8; 64],
}

impl SecureEnvelope {
  pub fn signing_bytes(&self) -> Vec<u8> {
    let mut writer = ByteWriter::new();
    writer.write_bytes(&ENVELOPE_MAGIC);
    writer.write_u8(SECURITY_VERSION);
    self.sender.encode(&mut writer);
    writer.write_u64(self.sequence);
    writer.write_u64(self.timestamp_ms);
    writer.write_length_prefixed(&self.ciphertext);
    writer.into_vec()
  }

  pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<()> {
    let signature = Signature::from_bytes(&self.signature);
    verifying_key
      .verify(&self.signing_bytes(), &signature)
      .map_err(|e| ConduitError::Deserialization(format!("signature invalid: {e}")))?;
    Ok(())
  }

  pub fn decrypt(&self, key: &SessionKey) -> Result<Vec<u8>> {
    Cipher::decrypt(key, &self.ciphertext)
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    let mut writer = ByteWriter::new();
    writer.write_bytes(&self.signing_bytes());
    writer.write_bytes(&self.signature);
    writer.into_vec()
  }

  pub fn from_bytes(data: &[u8]) -> Result<Self> {
    if data.len() < 4 + 1 + 16 + 8 + 8 + 2 + 64 {
      return Err(ConduitError::Deserialization(
        "secure envelope too short".into(),
      ));
    }

    let signature_start = data.len() - 64;
    let signed = &data[..signature_start];
    let mut signature = [0u8; 64];
    signature.copy_from_slice(&data[signature_start..]);

    let mut reader = ByteReader::new(signed);
    let magic = reader.read_bytes(4)?;
    if magic != ENVELOPE_MAGIC {
      return Err(ConduitError::Deserialization("invalid envelope magic".into()));
    }

    let version = reader.read_u8()?;
    if version != SECURITY_VERSION {
      return Err(ConduitError::Deserialization(format!(
        "unsupported security version: {version}"
      )));
    }

    let sender = NodeId::decode(&mut reader)?;
    let sequence = reader.read_u64()?;
    let timestamp_ms = reader.read_u64()?;
    let ciphertext = reader.read_length_prefixed()?;

    Ok(Self {
      sender,
      sequence,
      timestamp_ms,
      ciphertext,
      signature,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::identity::Identity;

  #[test]
  fn envelope_round_trip_bytes() {
    let identity = Identity::generate(NodeId::random());
    let envelope = SecureEnvelope {
      sender: identity.node_id(),
      sequence: 1,
      timestamp_ms: 1_000,
      ciphertext: vec![1, 2, 3],
      signature: identity.sign(b"placeholder"),
    };
    let bytes = envelope.to_bytes();
    let restored = SecureEnvelope::from_bytes(&bytes).unwrap();
    assert_eq!(restored.sender, envelope.sender);
    assert_eq!(restored.sequence, envelope.sequence);
  }
}
