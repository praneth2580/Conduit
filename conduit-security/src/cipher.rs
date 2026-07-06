use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use conduit_core::error::{ConduitError, Result};
use rand::RngCore;

pub const NONCE_SIZE: usize = 12;

/// AEAD encryption with session keys.
pub struct Cipher;

impl Cipher {
  pub fn encrypt(key: &crate::session::SessionKey, plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.0.as_ref().into());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
      .encrypt(nonce, plaintext)
      .map_err(|e| ConduitError::Serialization(format!("encrypt failed: {e}")))?;

    let mut out = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
  }

  pub fn decrypt(key: &crate::session::SessionKey, data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < NONCE_SIZE {
      return Err(ConduitError::Deserialization(
        "ciphertext too short for nonce".into(),
      ));
    }
    let cipher = ChaCha20Poly1305::new(key.0.as_ref().into());
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
      .decrypt(nonce, ciphertext)
      .map_err(|e| ConduitError::Deserialization(format!("decrypt failed: {e}")))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::session::SessionKey;

  #[test]
  fn round_trip() {
    let key = SessionKey([0xAB; 32]);
    let plain = b"secure packet";
    let enc = Cipher::encrypt(&key, plain).unwrap();
    assert_eq!(Cipher::decrypt(&key, &enc).unwrap(), plain);
  }
}
