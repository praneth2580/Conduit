use conduit_core::{ConduitConfig, LogLevel, NodeId};
use conduit_discovery::DiscoveryConfig;
use conduit_mesh::MeshConfig;
use conduit_routing::RoutingConfig;
use conduit_security::SecurityConfig;
use conduit_transport::TransportConfig;
use crate::network::SimBusHandle;
use conduit_voice::VoiceConfig;

/// Application-facing configuration for [`crate::Conduit`].
#[derive(Debug, Clone)]
pub struct SdkConfig {
  pub core: ConduitConfig,
  pub node_name: String,
  pub capabilities: u32,
  pub discovery: DiscoveryConfig,
  pub mesh: MeshConfig,
  pub routing: RoutingConfig,
  pub transport: TransportConfig,
  pub security: SecurityConfig,
  pub voice: VoiceConfig,
  /// Use mock discovery and an in-memory network (for tests and simulations).
  pub simulation: bool,
  /// Deterministic identity seed for reproducible tests.
  pub identity_seed: Option<[u8; 32]>,
  /// Shared simulation bus (created automatically when unset).
  pub sim_bus: Option<SimBusHandle>,
}

impl SdkConfig {
  pub fn builder() -> SdkConfigBuilder {
    SdkConfigBuilder::default()
  }

  pub fn from_core(core: ConduitConfig) -> Self {
    Self::builder().core(core).build()
  }

  /// Preset for local multi-node simulations.
  pub fn simulation(node_name: impl Into<String>) -> Self {
    Self::builder()
      .node_name(node_name)
      .simulation(true)
      .build()
  }

  pub fn validate(&self) -> conduit_core::Result<()> {
    self.core.validate()?;
    self.discovery.validate()?;
    self.mesh.validate()?;
    self.routing.validate()?;
    self.transport.validate()?;
    self.security.validate()?;
    self.voice.validate()?;
    if self.node_name.is_empty() {
      return Err(conduit_core::ConduitError::Configuration(
        "node_name must not be empty".into(),
      ));
    }
    Ok(())
  }

  fn sync_layer_ids(&mut self) {
    let id = self.core.node_id;
    self.discovery.node_id = id;
    self.mesh.local_node_id = id;
    self.routing.local_node_id = id;
    self.security.local_node_id = id;
    self.voice.local_node_id = id;
  }
}

#[derive(Debug, Clone)]
pub struct SdkConfigBuilder {
  core: Option<ConduitConfig>,
  node_name: String,
  capabilities: u32,
  transport: Option<TransportConfig>,
  simulation: bool,
  identity_seed: Option<[u8; 32]>,
  sim_bus: Option<SimBusHandle>,
}

impl Default for SdkConfigBuilder {
  fn default() -> Self {
    Self {
      core: None,
      node_name: "conduit-node".into(),
      capabilities: conduit_mesh::capabilities::VOICE
        | conduit_mesh::capabilities::RELAY
        | conduit_mesh::capabilities::LOCATION,
      transport: None,
      simulation: false,
      identity_seed: None,
      sim_bus: None,
    }
  }
}

impl SdkConfigBuilder {
  pub fn core(mut self, core: ConduitConfig) -> Self {
    self.core = Some(core);
    self
  }

  pub fn node_id(mut self, id: NodeId) -> Self {
    let mut core = self.core.unwrap_or_default();
    core.node_id = id;
    self.core = Some(core);
    self
  }

  pub fn node_name(mut self, name: impl Into<String>) -> Self {
    self.node_name = name.into();
    self
  }

  pub fn capabilities(mut self, caps: u32) -> Self {
    self.capabilities = caps;
    self
  }

  pub fn log_level(mut self, level: LogLevel) -> Self {
    let mut core = self.core.unwrap_or_default();
    core.log_level = level;
    self.core = Some(core);
    self
  }

  pub fn transport(mut self, transport: TransportConfig) -> Self {
    self.transport = Some(transport);
    self
  }

  pub fn simulation(mut self, enabled: bool) -> Self {
    self.simulation = enabled;
    self
  }

  pub fn identity_seed(mut self, seed: [u8; 32]) -> Self {
    self.identity_seed = Some(seed);
    self
  }

  pub fn sim_bus(mut self, bus: SimBusHandle) -> Self {
    self.sim_bus = Some(bus);
    self
  }

  pub fn build(self) -> SdkConfig {
    let core = self.core.unwrap_or_default();
    let node_id = core.node_id;

    let discovery = DiscoveryConfig::builder()
      .node_id(node_id)
      .node_name(self.node_name.clone())
      .capabilities(self.capabilities)
      .enable_udp_broadcast(!self.simulation)
      .build();

    let mesh = MeshConfig::builder()
      .local_node_id(node_id)
      .heartbeat_interval_ms(core.heartbeat_interval_ms)
      .neighbor_timeout_ms(core.neighbor_timeout_ms)
      .build();

    let routing = RoutingConfig::builder().local_node_id(node_id).build();

    let transport = self.transport.unwrap_or_else(|| {
      TransportConfig::builder()
        .enable_compression(false)
        .build()
    });

    let security = SecurityConfig::builder()
      .local_node_id(node_id)
      .require_known_peers(false)
      .build();

    let voice = VoiceConfig::builder()
      .local_node_id(node_id)
      .push_to_talk(false)
      .build();

    let mut config = SdkConfig {
      core,
      node_name: self.node_name,
      capabilities: self.capabilities,
      discovery,
      mesh,
      routing,
      transport,
      security,
      voice,
      simulation: self.simulation,
      identity_seed: self.identity_seed,
      sim_bus: self.sim_bus,
    };
    config.sync_layer_ids();
    config
  }
}
