use crate::driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
};
use conduit_core::error::{ConduitError, Result};

/// Wi-Fi Aware discovery backend (platform stub).
///
/// On Android this will bind to the Wi-Fi Aware API. On other platforms it
/// reports unavailability so the driver chain can fall through.
#[derive(Debug, Default)]
pub struct WifiAwareDriver {
  available: bool,
  state: DiscoveryState,
}

impl WifiAwareDriver {
  pub fn new() -> Self {
    Self {
      available: cfg!(target_os = "android"),
      state: DiscoveryState::Stopped,
    }
  }

  /// Mark driver available for testing fallback logic.
  pub fn force_available(mut self) -> Self {
    self.available = true;
    self
  }
}

impl DiscoveryDriver for WifiAwareDriver {
  fn kind(&self) -> DriverKind {
    DriverKind::WifiAware
  }

  fn state(&self) -> DiscoveryState {
    self.state.clone()
  }

  fn start(&mut self, _announcement: &DiscoveryAnnouncement) -> Result<()> {
    if !self.available {
      return Err(ConduitError::Configuration(
        "Wi-Fi Aware is not available on this platform".into(),
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
        "Wi-Fi Aware driver is not running".into(),
      ));
    }
    Ok(())
  }

  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>> {
    Ok(Vec::new())
  }
}
