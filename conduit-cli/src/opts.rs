use crate::CliError;
use conduit_core::LogLevel;
use conduit_sdk::SdkConfig;

pub fn parse_log_level(value: &str) -> Result<LogLevel, CliError> {
  match value.to_ascii_lowercase().as_str() {
    "trace" => Ok(LogLevel::Trace),
    "debug" => Ok(LogLevel::Debug),
    "info" => Ok(LogLevel::Info),
    "warn" | "warning" => Ok(LogLevel::Warn),
    "error" => Ok(LogLevel::Error),
    other => Err(CliError::Message(format!("unknown log level: {other}"))),
  }
}

pub fn sdk_config(name: &str, simulation: bool, log_level: &str) -> Result<SdkConfig, CliError> {
  let level = parse_log_level(log_level)?;
  let mut config = SdkConfig::builder()
    .node_name(name)
    .log_level(level)
    .simulation(simulation)
    .build();
  if simulation {
    config.simulation = true;
  }
  Ok(config)
}

pub fn sdk_config_seeded(name: &str, seed: u8) -> SdkConfig {
  let mut id_bytes = [0u8; 16];
  id_bytes[0] = seed;
  let mut seed_bytes = [0u8; 32];
  seed_bytes[0] = seed;
  SdkConfig::builder()
    .node_id(conduit_sdk::NodeId::from_bytes(id_bytes))
    .node_name(name)
    .simulation(true)
    .identity_seed(seed_bytes)
    .build()
}
