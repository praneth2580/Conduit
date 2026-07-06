//! Conduit Core — foundation layer for the Conduit communication platform.
//!
//! Phase 1 responsibilities: configuration, logging, utilities, IDs, versioning,
//! serialization helpers, packet definitions, constants, and the event system.
//!
//! Nothing in this crate communicates over a network.

pub mod config;
pub mod constants;
pub mod error;
pub mod events;
pub mod ids;
pub mod logging;
pub mod packets;
pub mod serialization;
pub mod utils;
pub mod version;

pub use config::ConduitConfig;
pub use error::{ConduitError, Result};
pub use events::{Event, EventBus, EventKind};
pub use ids::{NodeId, PacketSequence};
pub use logging::{init_logging, LogLevel};
pub use packets::{Packet, PacketFlags, PacketHeader, PacketPriority, PacketType};
pub use version::ProtocolVersion;
