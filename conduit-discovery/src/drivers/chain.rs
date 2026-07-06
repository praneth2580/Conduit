use crate::driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
};
use conduit_core::error::Result;

/// Tries discovery drivers in order until one starts successfully.
///
/// Matches the README fallback chain: Wi-Fi Aware → Wi-Fi Direct → Hotspot → …
pub struct DriverChain {
  drivers: Vec<Box<dyn DiscoveryDriver>>,
  active: Option<usize>,
}

impl DriverChain {
  pub fn new(drivers: Vec<Box<dyn DiscoveryDriver>>) -> Self {
    Self {
      drivers,
      active: None,
    }
  }

  pub fn active_kind(&self) -> Option<DriverKind> {
    self.active.map(|idx| self.drivers[idx].kind())
  }

  pub fn active_driver(&self) -> Option<&dyn DiscoveryDriver> {
    self.active.map(|idx| self.drivers[idx].as_ref())
  }
}

impl DiscoveryDriver for DriverChain {
  fn kind(&self) -> DriverKind {
    self
      .active_kind()
      .unwrap_or(self.drivers.first().map(|d| d.kind()).unwrap_or(DriverKind::Mock))
  }

  fn state(&self) -> DiscoveryState {
    match self.active {
      Some(idx) => self.drivers[idx].state(),
      None => DiscoveryState::Stopped,
    }
  }

  fn start(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()> {
    if self.active.is_some() {
      return Ok(());
    }

    let mut failures = Vec::new();
    for (idx, driver) in self.drivers.iter_mut().enumerate() {
      match driver.start(announcement) {
        Ok(()) => {
          self.active = Some(idx);
          return Ok(());
        }
        Err(e) => failures.push((driver.kind(), e.to_string())),
      }
    }

    Err(conduit_core::ConduitError::Configuration(format!(
      "no discovery driver available: {failures:?}"
    )))
  }

  fn stop(&mut self) -> Result<()> {
    if let Some(idx) = self.active.take() {
      self.drivers[idx].stop()?;
    }
    Ok(())
  }

  fn announce(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()> {
    let idx = self
      .active
      .ok_or_else(|| conduit_core::ConduitError::Configuration("no active driver".into()))?;
    self.drivers[idx].announce(announcement)
  }

  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>> {
    let idx = match self.active {
      Some(idx) => idx,
      None => return Ok(Vec::new()),
    };
    self.drivers[idx].poll()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::drivers::{HotspotDriver, UdpBroadcastDriver, WifiAwareDriver};

  #[test]
  fn falls_back_to_udp_when_platform_stubs_unavailable() {
    let port = {
      let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
      socket.local_addr().unwrap().port()
    };
    let chain = DriverChain::new(vec![
      Box::new(WifiAwareDriver::new()),
      Box::new(HotspotDriver::new()),
      Box::new(UdpBroadcastDriver::new(port)),
    ]);

    let mut chain = chain;
    let ann = DiscoveryAnnouncement {
      node_id: conduit_core::NodeId::random(),
      node_name: "chain-node".into(),
      capabilities: 1,
    };

    chain.start(&ann).unwrap();
    assert_eq!(chain.active_kind(), Some(DriverKind::UdpBroadcast));
  }

  #[test]
  fn uses_first_available_driver() {
    let chain = DriverChain::new(vec![
      Box::new(WifiAwareDriver::new().force_available()),
      Box::new(HotspotDriver::new()),
    ]);
    let mut chain = chain;
    let ann = DiscoveryAnnouncement {
      node_id: conduit_core::NodeId::random(),
      node_name: "aware-node".into(),
      capabilities: 1,
    };
    chain.start(&ann).unwrap();
    assert_eq!(chain.active_kind(), Some(DriverKind::WifiAware));
  }
}
