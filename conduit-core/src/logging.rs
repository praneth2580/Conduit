use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

/// Log verbosity for a Conduit node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}

impl LogLevel {
  pub fn to_tracing_level(self) -> Level {
    match self {
      Self::Trace => Level::TRACE,
      Self::Debug => Level::DEBUG,
      Self::Info => Level::INFO,
      Self::Warn => Level::WARN,
      Self::Error => Level::ERROR,
    }
  }
}

impl Default for LogLevel {
  fn default() -> Self {
    Self::Info
  }
}

/// Initialize the global tracing subscriber.
///
/// Respects `CONDUIT_LOG` if set; otherwise uses the provided level.
pub fn init_logging(level: LogLevel) {
  let filter = EnvFilter::try_from_env("CONDUIT_LOG")
    .unwrap_or_else(|_| EnvFilter::new(level.to_tracing_level().as_str()));

  let _ = fmt()
    .with_env_filter(filter)
    .with_target(true)
    .with_thread_ids(false)
    .with_file(false)
    .with_line_number(false)
    .try_init();
}

/// Structured log macros re-exported for convenience.
pub use tracing::{debug, error, info, trace, warn};

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn log_level_maps_to_tracing() {
    assert_eq!(LogLevel::Debug.to_tracing_level(), Level::DEBUG);
  }
}
