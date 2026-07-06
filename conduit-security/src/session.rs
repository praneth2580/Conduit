use conduit_core::ids::NodeId;
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;

pub const SESSION_KEY_SIZE: usize = 32;

/// Symmetric session key derived for a peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionKey(pub [u8; SESSION_KEY_SIZE]);

impl SessionKey {
  pub fn derive(shared_secret: &[u8; 32], local: NodeId, peer: NodeId) -> Self {
    let mut local_bytes = *local.as_bytes();
    let mut peer_bytes = *peer.as_bytes();
    if local_bytes > peer_bytes {
      std::mem::swap(&mut local_bytes, &mut peer_bytes);
    }

    let salt = [local_bytes, peer_bytes].concat();
    let mut okm = [0u8; SESSION_KEY_SIZE];
    Hkdf::<Sha256>::new(Some(&salt), shared_secret)
      .expand(b"conduit-session-v1", &mut okm)
      .expect("hkdf expand fits");
    Self(okm)
  }
}

/// Per-peer session key store.
#[derive(Debug, Default)]
pub struct SessionStore {
  keys: HashMap<NodeId, SessionKey>,
}

impl SessionStore {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn insert(&mut self, peer: NodeId, key: SessionKey) {
    self.keys.insert(peer, key);
  }

  pub fn get(&self, peer: &NodeId) -> Option<SessionKey> {
    self.keys.get(peer).copied()
  }

  pub fn remove(&mut self, peer: &NodeId) {
    self.keys.remove(peer);
  }

  pub fn contains(&self, peer: &NodeId) -> bool {
    self.keys.contains_key(peer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn session_key_is_symmetric() {
    let a = NodeId::random();
    let b = NodeId::random();
    let secret = [7u8; 32];
    let key_ab = SessionKey::derive(&secret, a, b);
    let key_ba = SessionKey::derive(&secret, b, a);
    assert_eq!(key_ab, key_ba);
  }
}
