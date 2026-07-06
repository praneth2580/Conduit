use conduit_core::version::ProtocolVersion;

pub fn run() -> crate::CliResult<()> {
  println!("conduit-cli {}", env!("CARGO_PKG_VERSION"));
  println!("protocol {}", ProtocolVersion::CURRENT);
  Ok(())
}
