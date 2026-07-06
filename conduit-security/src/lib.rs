//! Conduit Security — encryption, authentication, session keys, replay protection,
//! and identity verification below the application layer.

mod cipher;
mod config;
mod engine;
mod envelope;
mod identity;
mod replay;
mod session;

pub use config::SecurityConfig;
pub use engine::SecurityEngine;
pub use envelope::SecureEnvelope;
pub use identity::{Identity, IdentityProof, PeerIdentity};
pub use replay::ReplayGuard;
pub use session::{SessionKey, SessionStore};

pub use conduit_core::{ConduitError, NodeId, Packet, Result};
