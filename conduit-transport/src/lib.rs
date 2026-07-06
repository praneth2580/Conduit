//! Conduit Transport Layer — turns packet objects into wire byte streams.
//!
//! Responsibilities: serialization, parsing, fragmentation, reassembly,
//! compression, and encryption. This layer knows nothing about applications.

mod compression;
mod config;
mod encryption;
mod engine;
mod frame;
mod fragment;
mod reassembly;

pub use compression::Compression;
pub use config::TransportConfig;
pub use encryption::Encryption;
pub use engine::{generate_key, TransportEngine};
pub use frame::{TransportFlags, TransportFrame};
pub use fragment::Fragmenter;
pub use reassembly::Reassembler;

pub use conduit_core::{ConduitError, Packet, Result};
pub use conduit_packets::{PacketCodec, TypedPacket};
