use conduit_sdk::SdkConfig;

/// Default interval between automatic location broadcasts (milliseconds).
pub const DEFAULT_LOCATION_SHARE_INTERVAL_MS: u64 = 5_000;

/// Configuration for a ride intercom session.
#[derive(Debug, Clone)]
pub struct RideIntercomConfig {
  pub sdk: SdkConfig,
  pub rider_name: String,
  pub group_name: String,
  pub location_share_interval_ms: u64,
  pub push_to_talk: bool,
}

impl RideIntercomConfig {
  pub fn builder() -> RideIntercomConfigBuilder {
    RideIntercomConfigBuilder::default()
  }

  pub fn simulation(rider_name: impl Into<String>, group_name: impl Into<String>) -> Self {
    Self::builder()
      .rider_name(rider_name)
      .group_name(group_name)
      .simulation(true)
      .build()
  }

  pub fn validate(&self) -> conduit_sdk::Result<()> {
    self.sdk.validate()?;
    if self.rider_name.is_empty() {
      return Err(conduit_sdk::ConduitError::Configuration(
        "rider_name must not be empty".into(),
      ));
    }
    if self.group_name.is_empty() {
      return Err(conduit_sdk::ConduitError::Configuration(
        "group_name must not be empty".into(),
      ));
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct RideIntercomConfigBuilder {
  sdk: Option<SdkConfig>,
  rider_name: String,
  group_name: String,
  location_share_interval_ms: u64,
  push_to_talk: bool,
  simulation: bool,
}

impl Default for RideIntercomConfigBuilder {
  fn default() -> Self {
    Self {
      sdk: None,
      rider_name: "rider".into(),
      group_name: "default-group".into(),
      location_share_interval_ms: DEFAULT_LOCATION_SHARE_INTERVAL_MS,
      push_to_talk: true,
      simulation: false,
    }
  }
}

impl RideIntercomConfigBuilder {
  pub fn sdk(mut self, sdk: SdkConfig) -> Self {
    self.sdk = Some(sdk);
    self
  }

  pub fn rider_name(mut self, name: impl Into<String>) -> Self {
    self.rider_name = name.into();
    self
  }

  pub fn group_name(mut self, name: impl Into<String>) -> Self {
    self.group_name = name.into();
    self
  }

  pub fn location_share_interval_ms(mut self, ms: u64) -> Self {
    self.location_share_interval_ms = ms;
    self
  }

  pub fn push_to_talk(mut self, enabled: bool) -> Self {
    self.push_to_talk = enabled;
    self
  }

  pub fn simulation(mut self, enabled: bool) -> Self {
    self.simulation = enabled;
    self
  }

  pub fn build(self) -> RideIntercomConfig {
    let mut sdk = self.sdk.unwrap_or_else(|| {
      SdkConfig::builder()
        .node_name(self.rider_name.clone())
        .simulation(self.simulation)
        .build()
    });
    if self.simulation {
      sdk.simulation = true;
    }
    sdk.discovery.node_name = self.rider_name.clone();
    sdk.voice.push_to_talk = self.push_to_talk;

    RideIntercomConfig {
      sdk,
      rider_name: self.rider_name,
      group_name: self.group_name,
      location_share_interval_ms: self.location_share_interval_ms,
      push_to_talk: self.push_to_talk,
    }
  }
}