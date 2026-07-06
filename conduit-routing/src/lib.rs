//! Conduit Routing Engine — forwards packets through the mesh.
//!
//! Inspects packet headers only; payload content is opaque. Responsibilities:
//! forwarding, route selection, duplicate detection, TTL, congestion control,
//! neighbor scoring, and loop prevention.

mod config;
mod congestion;
mod decision;
mod duplicate;
mod engine;
mod routes;
mod scoring;

pub use config::RoutingConfig;
pub use congestion::CongestionTracker;
pub use decision::{RoutingAction, RoutingDropReason};
pub use duplicate::DuplicateCache;
pub use engine::RoutingEngine;
pub use routes::{RouteEntry, RouteTable};
pub use scoring::NeighborScorer;

pub use conduit_core::{NodeId, Packet, PacketType, Result};
pub use conduit_mesh::{Neighbor, BROADCAST_NODE_ID};
pub use conduit_packets::{PacketBody, RoutingPayload, TypedPacket};
