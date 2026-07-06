use conduit_core::ConduitError;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
  Conduit(ConduitError),
  Message(String),
}

pub type CliResult<T> = Result<T, CliError>;

impl From<ConduitError> for CliError {
  fn from(value: ConduitError) -> Self {
    Self::Conduit(value)
  }
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Conduit(err) => write!(f, "{err}"),
      Self::Message(msg) => write!(f, "{msg}"),
    }
  }
}

impl std::error::Error for CliError {}
