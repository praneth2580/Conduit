use crate::driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
};
use conduit_core::error::{ConduitError, Result};

/// Hotspot-based discovery backend (platform stub).
#[derive(Debug, Default)]
pub struct HotspotDriver {
  available: bool,
  state: DiscoveryState,
}

impl HotspotDriver {
  pub fn new() -> Self {
    Self {
      available: true,
      state: DiscoveryState::Stopped,
    }
  }
}

impl DiscoveryDriver for HotspotDriver {
  fn kind(&self) -> DriverKind {
    DriverKind::Hotspot
  }

  fn state(&self) -> DiscoveryState {
    self.state.clone()
  }

  fn start(&mut self, _announcement: &DiscoveryAnnouncement) -> Result<()> {
    if !self.available {
      return Err(ConduitError::Configuration(
        "hotspot discovery is not available".into(),
      ));
    }
    self.state = DiscoveryState::Running;
    Ok(())
  }

  fn stop(&mut self) -> Result<()> {
    self.state = DiscoveryState::Stopped;
    Ok(())
  }

  fn announce(&mut self, _announcement: &DiscoveryAnnouncement) -> Result<()> {
    if !matches!(self.state, DiscoveryState::Running) {
      return Err(ConduitError::Configuration(
        "hotspot driver is not running".into(),
      ));
    }
    Ok(())
  }

  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>> {
    Ok(Vec::new())
  }
}
