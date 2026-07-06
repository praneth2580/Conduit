//! Conduit Discovery — find nearby peers without central infrastructure.
//!
//! Higher layers interact only with [`DiscoveryEngine`] and [`DiscoveryDriver`].
//! The active driver (Wi-Fi Aware, Wi-Fi Direct, hotspot, UDP broadcast, etc.)
//! is an implementation detail.

mod beacon;
mod config;
mod driver;
mod drivers;
mod engine;
mod peer;

pub use config::{DiscoveryConfig, DEFAULT_DATA_PORT, DEFAULT_DISCOVERY_PORT};
pub use driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
  PeerEndpoint,
};
pub use drivers::{
  DriverChain, HotspotDriver, MockDriver, UdpBroadcastDriver, WifiAwareDriver, WifiDirectDriver,
};
pub use engine::DiscoveryEngine;
pub use peer::{capabilities, DiscoveredPeer};

pub use conduit_core::{ConduitError, NodeId, Result};
