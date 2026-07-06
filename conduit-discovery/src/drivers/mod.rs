mod chain;
mod hotspot;
mod mock;
mod udp;
mod wifi_aware;
mod wifi_direct;

pub use chain::DriverChain;
pub use hotspot::HotspotDriver;
pub use mock::MockDriver;
pub use udp::UdpBroadcastDriver;
pub use wifi_aware::WifiAwareDriver;
pub use wifi_direct::WifiDirectDriver;

use crate::config::DiscoveryConfig;
use conduit_core::error::Result;

/// Build the default fallback driver chain from configuration.
pub fn default_driver_chain(config: &DiscoveryConfig) -> Result<Box<dyn crate::DiscoveryDriver>> {
  let mut drivers: Vec<Box<dyn crate::DiscoveryDriver>> = vec![
    Box::new(WifiAwareDriver::new()),
    Box::new(WifiDirectDriver::new()),
    Box::new(HotspotDriver::new()),
  ];
  if config.enable_udp_broadcast {
    drivers.push(Box::new(UdpBroadcastDriver::new(config.udp_port)));
  }
  Ok(Box::new(DriverChain::new(drivers)))
}
