use conduit_core::ids::NodeId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Well-known discovery driver implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverKind {
  WifiAware,
  WifiDirect,
  Hotspot,
  UdpBroadcast,
  Mock,
}

/// Opaque connectable address reported by a driver.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PeerEndpoint {
  Udp { addr: SocketAddr },
  /// Platform-specific address blob (Wi-Fi Aware / Direct handles).
  Native { bytes: Vec<u8> },
  /// In-memory simulation endpoint.
  Simulated { id: u64 },
}

/// Local node advertisement broadcast to nearby peers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryAnnouncement {
  pub node_id: NodeId,
  pub node_name: String,
  pub capabilities: u32,
}

/// Lifecycle state of a discovery driver.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DiscoveryState {
  #[default]
  Stopped,
  Running,
}

/// Events emitted by discovery drivers and the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryEvent {
  PeerFound(crate::DiscoveredPeer),
  PeerLost { node_id: NodeId, driver: DriverKind },
  DriverStarted { driver: DriverKind },
  DriverStopped { driver: DriverKind },
  DriverFailed { driver: DriverKind, reason: String },
}

/// Replaceable discovery backend.
///
/// The mesh and routing layers depend on this trait — never on a concrete driver.
pub trait DiscoveryDriver: Send {
  fn kind(&self) -> DriverKind;

  fn state(&self) -> DiscoveryState;

  fn start(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()>;

  fn stop(&mut self) -> Result<()>;

  /// Send an immediate announcement (e.g. UDP broadcast).
  fn announce(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()>;

  /// Non-blocking poll for driver events since the last call.
  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>>;
}

use conduit_core::error::Result;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::drivers::MockDriver;

  #[test]
  fn mock_driver_lifecycle() {
    let mut driver = MockDriver::new();
    let announcement = DiscoveryAnnouncement {
      node_id: NodeId::random(),
      node_name: "test".into(),
      capabilities: 1,
    };
    assert_eq!(driver.state(), DiscoveryState::Stopped);
    driver.start(&announcement).unwrap();
    assert_eq!(driver.state(), DiscoveryState::Running);
    driver.stop().unwrap();
    assert_eq!(driver.state(), DiscoveryState::Stopped);
  }
}
