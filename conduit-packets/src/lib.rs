//! Conduit Packet System — typed payloads layered on the shared packet header.
//!
//! Every packet shares one [`PacketHeader`] (from `conduit-core`). The payload
//! format depends on [`PacketType`]. Routing layers inspect only the header;
//! new payload types can be added without routing changes.

mod body;
mod builder;
mod codec;
mod payload;
mod payloads;
mod typed;

pub use body::PacketBody;
pub use builder::PacketBuilder;
pub use codec::PacketCodec;
pub use payload::PacketPayload;
pub use payloads::{
  ControlCommand, ControlPayload, DiscoveryPayload, EmergencyKind, EmergencyPayload,
  HeartbeatPayload, LocationPayload, MessagingPayload, RoutingPayload, TelemetryPayload,
  VoicePayload,
};
pub use typed::TypedPacket;

pub use conduit_core::{
  ConduitError, NodeId, Packet, PacketFlags, PacketHeader, PacketPriority, PacketSequence,
  PacketType, ProtocolVersion, Result,
};
