use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use conduit_core::error::{ConduitError, Result};
use rand::RngCore;

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

/// ChaCha20-Poly1305 encryption for transport blobs.
pub struct Encryption;

impl Encryption {
  pub fn encrypt(key: &[u8; KEY_SIZE], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
      .encrypt(nonce, plaintext)
      .map_err(|e| ConduitError::Serialization(format!("encryption failed: {e}")))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
  }

  pub fn decrypt(key: &[u8; KEY_SIZE], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < NONCE_SIZE {
      return Err(ConduitError::Deserialization(
        "encrypted blob too short for nonce".into(),
      ));
    }

    let cipher = ChaCha20Poly1305::new(key.into());
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
      .decrypt(nonce, ciphertext)
      .map_err(|e| ConduitError::Deserialization(format!("decryption failed: {e}")))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_key() -> [u8; KEY_SIZE] {
    [0x42; KEY_SIZE]
  }

  #[test]
  fn round_trip() {
    let key = test_key();
    let plaintext = b"sensitive packet bytes";
    let encrypted = Encryption::encrypt(&key, plaintext).unwrap();
    let restored = Encryption::decrypt(&key, &encrypted).unwrap();
    assert_eq!(restored, plaintext);
  }
}
