use crate::driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
};
use conduit_core::error::{ConduitError, Result};

/// Wi-Fi Direct discovery backend (platform stub).
#[derive(Debug, Default)]
pub struct WifiDirectDriver {
  available: bool,
  state: DiscoveryState,
}

impl WifiDirectDriver {
  pub fn new() -> Self {
    Self {
      available: cfg!(any(target_os = "android", target_os = "linux")),
      state: DiscoveryState::Stopped,
    }
  }

  pub fn force_available(mut self) -> Self {
    self.available = true;
    self
  }
}

impl DiscoveryDriver for WifiDirectDriver {
  fn kind(&self) -> DriverKind {
    DriverKind::WifiDirect
  }

  fn state(&self) -> DiscoveryState {
    self.state.clone()
  }

  fn start(&mut self, _announcement: &DiscoveryAnnouncement) -> Result<()> {
    if !self.available {
      return Err(ConduitError::Configuration(
        "Wi-Fi Direct is not available on this platform".into(),
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
        "Wi-Fi Direct driver is not running".into(),
      ));
    }
    Ok(())
  }

  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>> {
    Ok(Vec::new())
  }
}
