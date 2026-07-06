use crate::args::ConfigCommands;
use conduit_core::ConduitConfig;

pub fn run(command: ConfigCommands) -> crate::CliResult<()> {
  match command {
    ConfigCommands::Init { output } => init(&output),
    ConfigCommands::Validate { file } => validate(&file),
  }
}

fn init(output: &std::path::Path) -> crate::CliResult<()> {
  let config = ConduitConfig::default();
  config.save_to_file(output)?;
  println!("wrote {}", output.display());
  Ok(())
}

fn validate(file: &std::path::Path) -> crate::CliResult<()> {
  let config = ConduitConfig::load_from_file(file)?;
  config.validate()?;
  println!("valid: {}", file.display());
  println!("node_id: {}", config.node_id);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init_and_validate_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("conduit.json");
    init(&path).unwrap();
    validate(&path).unwrap();
  }
}
