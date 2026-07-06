//! Conduit Mesh Engine — tracks nearby neighbors.
//!
//! The mesh layer maintains the neighbor table, link quality, signal quality,
//! and heartbeats. It never routes packets.

pub mod capabilities;
mod config;
mod engine;
mod events;
mod neighbor;
mod quality;
mod table;

pub use config::MeshConfig;
pub use engine::MeshEngine;
pub use events::{MeshEvent, MeshTick, NeighborRemovalReason};
pub use neighbor::{Neighbor, NeighborState, BROADCAST_NODE_ID};
pub use quality::{LinkQuality, SignalQuality};
pub use table::NeighborTable;

pub use conduit_core::{ConduitError, NodeId, Result};
pub use conduit_discovery::DiscoveryEvent;
pub use conduit_packets::{HeartbeatPayload, TypedPacket};
